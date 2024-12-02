use std::{
    collections::HashMap,
    ffi,
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, LazyLock,
    },
    thread::{self, JoinHandle},
};

use ori::{
    app::{AppRequest, UiBuilder},
    core::{command::CommandWaker, window::WindowUpdate},
    prelude::*,
};
use ori_skia::{SkiaFonts, SkiaRenderer};
use vst3_sys::vst::{IComponentHandler, RestartFlags};
use x11_dl::{
    glx::{
        self, Glx, GLX_ALPHA_SIZE, GLX_BLUE_SIZE, GLX_DOUBLEBUFFER, GLX_GREEN_SIZE, GLX_RED_SIZE,
        GLX_RGBA, GLX_SAMPLES, GLX_SAMPLE_BUFFERS,
    },
    xcursor::Xcursor,
    xlib::{
        self, AllocNone, ButtonPressMask, ButtonReleaseMask, CWColormap, CWEventMask, Display,
        EnterWindowMask, ExposureMask, InputOutput, KeyPressMask, KeyReleaseMask, LeaveWindowMask,
        PointerMotionMask, StructureNotifyMask, XEvent, XKeyEvent, XSetWindowAttributes, Xlib,
    },
};
use xkeysym::Keysym;

use crate::{editor::EditorHandle, PluginState, VstPlugin};

static XLIB: LazyLock<Xlib> = LazyLock::new(|| Xlib::open().unwrap());
static XCURSOR: LazyLock<Xcursor> = LazyLock::new(|| Xcursor::open().unwrap());
static GLX: LazyLock<Glx> = LazyLock::new(|| Glx::open().unwrap());

pub unsafe fn spawn_editor<P: VstPlugin>(
    state: Arc<PluginState<P>>,
    parent: *mut ffi::c_void,
) -> Arc<dyn EditorHandle> {
    let (event_tx, event_rx) = mpsc::channel();

    let window = P::window();

    let handle = Arc::new(X11EditorHandle {
        event_tx,
        width: AtomicU32::new(window.width()),
        height: AtomicU32::new(window.height()),
        resizable: AtomicBool::new(window.resizable),
    });

    spawn_editor_thread(state, parent, event_rx, handle.clone(), window);

    handle
}

struct X11EditorHandle {
    event_tx: Sender<EditorEvent>,
    width: AtomicU32,
    height: AtomicU32,
    resizable: AtomicBool,
}

impl EditorHandle for X11EditorHandle {
    fn quit(&self) {
        let _ = self.event_tx.send(EditorEvent::Quit);
    }

    fn size(&self) -> (u32, u32) {
        let width = self.width.load(Ordering::Relaxed);
        let height = self.height.load(Ordering::Relaxed);

        (width, height)
    }

    fn resize(&self, width: u32, height: u32) {
        let _ = self.event_tx.send(EditorEvent::Resize(width, height));
    }

    fn resizable(&self) -> bool {
        self.resizable.load(Ordering::Relaxed)
    }

    fn rebuild(&self) {
        let _ = self.event_tx.send(EditorEvent::Rebuild);
    }
}

#[allow(dead_code)]
struct X11Editor<P: VstPlugin> {
    parent: *mut ffi::c_void,
    display: *mut Display,
    event_thread: JoinHandle<()>,
    handle: Arc<X11EditorHandle>,
    state: Arc<PluginState<P>>,

    params: Vec<f32>,

    app: App<P>,
    window: Option<X11Window>,

    render: bool,
    running: Arc<AtomicBool>,
}

struct X11Window {
    id: WindowId,
    glx: glx::GLXContext,
    window: xlib::Window,
    cursor: Cursor,
    cursors: HashMap<Cursor, xlib::Cursor>,
    renderer: ManuallyDrop<SkiaRenderer>,
}

unsafe fn handle_app_requests<P: VstPlugin>(editor: &mut X11Editor<P>) {
    for request in editor.app.take_requests() {
        handle_app_request(editor, request);
    }
}

unsafe fn handle_app_request<P: VstPlugin>(editor: &mut X11Editor<P>, request: AppRequest<P>) {
    match request {
        AppRequest::OpenWindow(window, ui) => open_window(editor, window, ui),
        AppRequest::CloseWindow(_) => editor.running.store(false, Ordering::Relaxed),
        AppRequest::DragWindow(_) => {
            warn!("DragWindow is not supported on X11");
        }
        AppRequest::RequestRedraw(_) => editor.render = true,
        AppRequest::UpdateWindow(_, update) => match update {
            WindowUpdate::Title(_) => warn!("Title is not supported in VSTs"),
            WindowUpdate::Icon(_) => warn!("Icon is not supported in VSTs"),
            WindowUpdate::Size(_) => warn!("Size is not supported in VSTs"),
            WindowUpdate::Scale(_) => warn!("Scale is not supported in VSTs"),
            WindowUpdate::Resizable(_) => warn!("Resizable is not supported in VSTs"),
            WindowUpdate::Decorated(_) => warn!("Decorated is not supported in VSTs"),
            WindowUpdate::Maximized(_) => warn!("Maximized is not supported in VSTs"),
            WindowUpdate::Visible(_) => warn!("Visible is not supported in VSTs"),
            WindowUpdate::Color(_) => editor.render = true,
            WindowUpdate::Cursor(cursor) => {
                set_window_cursor(editor, cursor);
            }
            WindowUpdate::Ime(_) => {}
        },
        AppRequest::Quit => editor.running.store(false, Ordering::Relaxed),
    }
}

unsafe fn open_window<P: VstPlugin>(editor: &mut X11Editor<P>, window: Window, ui: UiBuilder<P>) {
    if editor.window.is_some() {
        panic!("Only one window is supported");
    }

    let mut attrs = [
        GLX_RGBA,
        GLX_RED_SIZE,
        8,
        GLX_BLUE_SIZE,
        8,
        GLX_GREEN_SIZE,
        8,
        GLX_ALPHA_SIZE,
        8,
        GLX_DOUBLEBUFFER,
        GLX_SAMPLE_BUFFERS,
        1,
        GLX_SAMPLES,
        4,
        0,
    ];

    let vi = (GLX.glXChooseVisual)(editor.display, 0, attrs.as_mut_ptr());

    if vi.is_null() {
        panic!("Failed to choose visual");
    }

    let colormap = (XLIB.XCreateColormap)(
        editor.display,
        editor.parent as u64,
        (*vi).visual,
        AllocNone,
    );

    let mut attrs = MaybeUninit::<XSetWindowAttributes>::uninit();

    (*attrs.as_mut_ptr()).colormap = colormap;
    (*attrs.as_mut_ptr()).event_mask = ExposureMask
        | StructureNotifyMask
        | PointerMotionMask
        | EnterWindowMask
        | LeaveWindowMask
        | ButtonPressMask
        | ButtonReleaseMask
        | KeyPressMask
        | KeyReleaseMask;

    let width = window.width();
    let height = window.height();

    editor.handle.width.store(width, Ordering::Relaxed);
    editor.handle.height.store(height, Ordering::Relaxed);

    let x11_window = (XLIB.XCreateWindow)(
        editor.display,
        editor.parent as u64,
        0,
        0,
        width,
        height,
        0,
        (*vi).depth,
        InputOutput as u32,
        (*vi).visual,
        CWColormap | CWEventMask,
        attrs.as_mut_ptr(),
    );

    (XLIB.XMapWindow)(editor.display, x11_window);

    let glx = (GLX.glXCreateContext)(editor.display, vi, ptr::null_mut(), 1);
    (GLX.glXMakeCurrent)(editor.display, x11_window, glx);

    let renderer = SkiaRenderer::new(|s| {
        if s.starts_with("egl") {
            return ptr::null();
        }

        let cstring = ffi::CString::new(s).unwrap();
        (GLX.glXGetProcAddress)(cstring.as_ptr() as *const _).unwrap() as *const _
    });

    (GLX.glXMakeCurrent)(editor.display, 0, ptr::null_mut());

    let x11_window = X11Window {
        id: window.id(),
        glx,
        window: x11_window,
        cursor: Cursor::default(),
        cursors: HashMap::new(),
        renderer: ManuallyDrop::new(renderer),
    };

    let mut plugin = editor.state.plugin.lock();
    editor.app.add_window(&mut plugin, ui, window);

    editor.window = Some(x11_window);
}

unsafe fn render_window<P: VstPlugin>(editor: &mut X11Editor<P>) {
    let Some(ref mut window) = editor.window else {
        return;
    };

    if !editor.render {
        return;
    }

    editor.render = false;

    let draw = {
        // we want to hold the lock for as short as possible
        let mut plugin = editor.state.plugin.lock();
        editor.app.draw_window(&mut plugin, window.id)
    };

    (GLX.glXMakeCurrent)(editor.display, window.window, window.glx);

    if let Some(draw) = draw {
        let width = editor.handle.width.load(Ordering::Relaxed);
        let height = editor.handle.height.load(Ordering::Relaxed);

        let fonts = editor.app.contexts.get_mut::<Box<dyn Fonts>>().unwrap();

        (window.renderer).render(
            fonts.downcast_mut().unwrap(),
            &draw.canvas,
            draw.clear_color,
            width,
            height,
            1.0,
        );
    }

    (GLX.glXSwapBuffers)(editor.display, window.window);
    (GLX.glXMakeCurrent)(editor.display, 0, ptr::null_mut());
}

unsafe fn set_window_cursor<P: VstPlugin>(editor: &mut X11Editor<P>, cursor: Cursor) {
    if let Some(ref mut window) = editor.window {
        let cursor = window.cursors.entry(cursor).or_insert_with(|| {
            let cstring = ffi::CString::new(cursor.name()).unwrap();
            (XCURSOR.XcursorLibraryLoadCursor)(editor.display, cstring.as_ptr())
        });

        (XLIB.XDefineCursor)(editor.display, window.window, *cursor);
    }
}

unsafe fn spawn_editor_thread<P: VstPlugin>(
    state: Arc<PluginState<P>>,
    parent: *mut ffi::c_void,
    event_rx: Receiver<EditorEvent>,
    handle: Arc<X11EditorHandle>,
    window: Window,
) -> JoinHandle<()> {
    let parent = AssertSend(parent);

    thread::spawn(move || {
        let parent = parent;
        let AssertSend(parent) = parent;

        let display = (XLIB.XOpenDisplay)(ptr::null());

        let app = App::build().window(window, |plugin: &mut P| any(plugin.ui()));

        let running = Arc::new(AtomicBool::new(true));
        let event_thread = spawn_event_thread(display, handle.event_tx.clone(), running.clone());
        let waker = CommandWaker::new({
            let event_tx = handle.event_tx.clone();

            move || {
                if let Err(err) = event_tx.send(EditorEvent::Wake) {
                    println!("Error sending wake event: {:?}", err);
                }
            }
        });

        let fonts = Box::new(SkiaFonts::new(Some("Roboto")));

        let app = app.build(waker, fonts);

        let params = state.param_values();

        let mut editor = X11Editor {
            parent,
            display,
            event_thread,
            handle,
            state,

            params,

            app,
            window: None,

            render: true,
            running,
        };

        editor.app.init(&mut editor.state.plugin.lock());

        while editor.running.load(Ordering::Relaxed) {
            (XLIB.XFlush)(display);

            editor.app.idle(&mut editor.state.plugin.lock());
            handle_app_requests(&mut editor);

            render_window(&mut editor);
            handle_app_requests(&mut editor);

            while let Ok(event) = event_rx.try_recv() {
                handle_event(&mut editor, event);
                handle_app_requests(&mut editor);
            }

            let params = editor.state.param_values();

            for (i, (old, new)) in editor.params.iter().zip(params.iter()).enumerate() {
                if old != new {
                    let component = editor.state.component.lock();
                    if let Some(component) = component.as_ref() {
                        let _ = component.begin_edit(i as u32);

                        component.perform_edit(i as u32, *new as f64);

                        let _ = component.end_edit(i as u32);

                        component.restart_component(RestartFlags::kParamValuesChanged as i32);
                    }
                }
            }

            editor.params = params;

            if editor.render {
                continue;
            }

            if let Ok(event) = event_rx.recv() {
                handle_event(&mut editor, event);
                handle_app_requests(&mut editor);
            }
        }
    })
}

unsafe fn handle_event<P: VstPlugin>(editor: &mut X11Editor<P>, event: EditorEvent) {
    match event {
        EditorEvent::Wake => {}
        EditorEvent::XEvent(event) => handle_xevent(editor, event),
        EditorEvent::Quit => editor.running.store(false, Ordering::Relaxed),
        EditorEvent::Resize(width, height) => {
            editor.handle.width.store(width, Ordering::Relaxed);
            editor.handle.height.store(height, Ordering::Relaxed);

            editor.render = true;

            if let Some(ref mut window) = editor.window {
                (XLIB.XResizeWindow)(editor.display, window.window, width, height);

                let mut plugin = editor.state.plugin.lock();
                (editor.app).window_resized(&mut plugin, window.id, width, height);
            }
        }
        EditorEvent::Rebuild => {
            let mut plugin = editor.state.plugin.lock();
            (editor.app).rebuild(&mut plugin);
        }
    }
}

unsafe fn handle_xevent<P: VstPlugin>(editor: &mut X11Editor<P>, mut event: XEvent) {
    match event.type_ {
        xlib::ClientMessage => {}
        xlib::Expose => {
            editor.render = true;
        }
        xlib::ConfigureNotify => {
            let width = event.configure.width as u32;
            let height = event.configure.height as u32;

            editor.handle.width.store(width, Ordering::Relaxed);
            editor.handle.height.store(height, Ordering::Relaxed);

            editor.render = true;
        }
        xlib::MotionNotify => {
            let position = Point::new(event.motion.x as f32, event.motion.y as f32);

            if let Some(ref window) = editor.window {
                let mut plugin = editor.state.plugin.lock();

                (editor.app).pointer_moved(
                    &mut plugin,
                    window.id,
                    PointerId::from_u64(0),
                    position,
                );
            }
        }
        xlib::EnterNotify => {
            let cursor = editor
                .window
                .as_ref()
                .map_or(Cursor::default(), |w| w.cursor);

            set_window_cursor(editor, cursor);
        }
        xlib::LeaveNotify => {
            if let Some(ref window) = editor.window {
                let mut plugin = editor.state.plugin.lock();

                (editor.app).pointer_left(&mut plugin, window.id, PointerId::from_u64(0));
            }
        }
        xlib::ButtonPress => {
            handle_pointer_button(editor, PointerId::from_u64(0), event.button.button, true);
        }
        xlib::ButtonRelease => {
            handle_pointer_button(editor, PointerId::from_u64(0), event.button.button, false);
        }
        xlib::KeyPress => {
            let modifiers = Modifiers {
                shift: event.key.state & xlib::ShiftMask != 0,
                ctrl: event.key.state & xlib::ControlMask != 0,
                alt: event.key.state & xlib::Mod1Mask != 0,
                meta: event.key.state & xlib::Mod4Mask != 0,
            };

            editor.app.modifiers_changed(modifiers);

            if let Some(ref window) = editor.window {
                let (text, _) = get_key_text(&mut event.key);

                event.key.state = 0;
                let (_, key) = get_key_text(&mut event.key);

                let scancode = event.key.keycode as u8;
                let keycode = Code::from_linux_scancode(scancode - 8);

                let mut plugin = editor.state.plugin.lock();

                (editor.app).keyboard_key(&mut plugin, window.id, key, keycode, text, true);
            }
        }
        xlib::KeyRelease => {
            let modifiers = Modifiers {
                shift: event.key.state & xlib::ShiftMask != 0,
                ctrl: event.key.state & xlib::ControlMask != 0,
                alt: event.key.state & xlib::Mod1Mask != 0,
                meta: event.key.state & xlib::Mod4Mask != 0,
            };

            editor.app.modifiers_changed(modifiers);

            if let Some(ref window) = editor.window {
                event.key.state = 0;
                let (_, key) = get_key_text(&mut event.key);

                let scancode = event.key.keycode as u8;
                let keycode = Code::from_linux_scancode(scancode - 8);

                let mut plugin = editor.state.plugin.lock();

                (editor.app).keyboard_key(&mut plugin, window.id, key, keycode, None, false);
            }
        }
        _ => {}
    }
}

unsafe fn get_key_text(key: &mut XKeyEvent) -> (Option<String>, Key) {
    let mut text = [0i8; 32];
    let mut keysym = 0;

    let count = (XLIB.XLookupString)(
        key,
        text.as_mut_ptr(),
        text.len() as i32,
        &mut keysym,
        ptr::null_mut(),
    );

    let text = match count {
        0 => None,
        _ => {
            let text = ffi::CStr::from_ptr(text.as_ptr());
            Some(text.to_string_lossy().into_owned())
        }
    };

    let keysym = Keysym::new(keysym as u32);

    let key = match text {
        Some(ref text) => {
            let mut chars = text.chars();
            let c = chars.next().unwrap();
            debug_assert!(chars.next().is_none());

            if !c.is_control() {
                Key::Character(c)
            } else {
                keysym_to_key(keysym)
            }
        }
        None => keysym_to_key(keysym),
    };

    (text, key)
}

unsafe fn handle_pointer_button<P: VstPlugin>(
    editor: &mut X11Editor<P>,
    pointer_id: PointerId,
    button: u32,
    pressed: bool,
) {
    if let Some(ref window) = editor.window {
        let mut plugin = editor.state.plugin.lock();

        match button {
            code @ 4..8 => {
                let delta = match code {
                    4 => Vector::Y,
                    5 => Vector::NEG_Y,
                    6 => Vector::X,
                    7 => Vector::NEG_X,
                    _ => unreachable!(),
                };

                (editor.app).pointer_scrolled(&mut plugin, window.id, pointer_id, delta);
            }
            _ => {
                let button = PointerButton::from_u16(button as u16);

                (editor.app).pointer_button(&mut plugin, window.id, pointer_id, button, pressed);
            }
        }
    }
}

unsafe fn spawn_event_thread(
    display: *mut Display,
    event_tx: Sender<EditorEvent>,
    running: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn({
        let display = AssertSend(display);

        move || {
            let display = display;
            let AssertSend(display) = display;

            while running.load(Ordering::Relaxed) {
                let mut event = MaybeUninit::uninit();

                if (XLIB.XNextEvent)(display, event.as_mut_ptr()) != 0 {
                    continue;
                }

                let event = event.assume_init();

                if event_tx.send(EditorEvent::XEvent(event)).is_err() {
                    break;
                }
            }
        }
    })
}

impl<P: VstPlugin> Drop for X11Editor<P> {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref mut window) = self.window {
                ptr::drop_in_place(&mut window.renderer);

                (GLX.glXDestroyContext)(self.display, window.glx);
                (XLIB.XDestroyWindow)(self.display, window.window);

                for (_, cursor) in window.cursors.drain() {
                    (XLIB.XFreeCursor)(self.display, cursor);
                }
            }

            // FIXME: for whatever reason, when the display is closed
            // the reaper will at some point in the future just close with
            // exit code 1
            // (XLIB.XCloseDisplay)(self.display);
        }
    }
}

enum EditorEvent {
    Wake,
    XEvent(XEvent),
    Quit,
    Resize(u32, u32),
    Rebuild,
}

struct AssertSend<T>(T);

unsafe impl Send for EditorEvent {}
unsafe impl<T> Send for AssertSend<T> {}

pub fn keysym_to_key(keysym: Keysym) -> Key {
    match keysym {
        /* modifier keys */
        Keysym::Alt_L | Keysym::Alt_R => Key::Alt,
        Keysym::SUN_AltGraph | Keysym::ISO_Level3_Shift => Key::AltGraph,
        Keysym::Control_L | Keysym::Control_R => Key::Control,
        Keysym::Shift_L | Keysym::Shift_R => Key::Shift,
        Keysym::Meta_L | Keysym::Meta_R => Key::Meta,
        Keysym::Super_L | Keysym::Super_R => Key::Super,
        Keysym::Hyper_L | Keysym::Hyper_R => Key::Hyper,
        Keysym::XF86_Fn | Keysym::XF86_Launch1 => Key::Fn,

        /* lock keys */
        Keysym::Num_Lock => Key::NumLock,
        Keysym::Caps_Lock => Key::CapsLock,
        Keysym::Scroll_Lock => Key::ScrollLock,

        /* function keys */
        Keysym::F1 => Key::F1,
        Keysym::F2 => Key::F2,
        Keysym::F3 => Key::F3,
        Keysym::F4 => Key::F4,
        Keysym::F5 => Key::F5,
        Keysym::F6 => Key::F6,
        Keysym::F7 => Key::F7,
        Keysym::F8 => Key::F8,
        Keysym::F9 => Key::F9,
        Keysym::F10 => Key::F10,
        Keysym::F11 => Key::F11,
        Keysym::F12 => Key::F12,
        Keysym::F13 => Key::F13,
        Keysym::F14 => Key::F14,
        Keysym::F15 => Key::F15,
        Keysym::F16 => Key::F16,
        Keysym::F17 => Key::F17,
        Keysym::F18 => Key::F18,
        Keysym::F19 => Key::F19,
        Keysym::F20 => Key::F20,
        Keysym::F21 => Key::F21,
        Keysym::F22 => Key::F22,
        Keysym::F23 => Key::F23,
        Keysym::F24 => Key::F24,

        /* misc keys */
        Keysym::Return | Keysym::KP_Enter => Key::Enter,
        Keysym::Tab => Key::Tab,
        Keysym::Down => Key::Down,
        Keysym::Left => Key::Left,
        Keysym::Right => Key::Right,
        Keysym::Up => Key::Up,
        Keysym::End => Key::End,
        Keysym::Home => Key::Home,
        Keysym::Page_Down => Key::PageDown,
        Keysym::Page_Up => Key::PageUp,
        Keysym::BackSpace => Key::Backspace,
        Keysym::Clear => Key::Clear,
        Keysym::SUN_Copy | Keysym::XF86_Copy => Key::Copy,
        Keysym::SUN_Cut | Keysym::XF86_Cut => Key::Cut,
        Keysym::Delete => Key::Delete,
        Keysym::Insert => Key::Insert,
        Keysym::SUN_Paste | Keysym::OSF_Paste | Keysym::XF86_Paste => Key::Paste,
        Keysym::Cancel => Key::Cancel,
        Keysym::Escape => Key::Escape,
        Keysym::Execute => Key::Execute,
        Keysym::Find => Key::Find,
        Keysym::Help => Key::Help,
        Keysym::Pause => Key::Pause,
        Keysym::Select => Key::Select,
        Keysym::SUN_Print_Screen => Key::PrintScreen,
        Keysym::Codeinput => Key::CodeInput,

        _ => Key::Unidentified,
    }
}
