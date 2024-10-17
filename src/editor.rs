use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread,
};

use ori::prelude::*;

use crate::{PluginState, VstPlugin};

#[derive(Clone)]
pub struct EditorHandle {
    raw: Arc<RawEditorHandle>,
}

impl EditorHandle {
    pub(crate) fn new_x11<P: VstPlugin>(state: Arc<PluginState<P>>, parent: u32) -> Self {
        let window = P::window();

        let raw = Arc::new(RawEditorHandle {
            proxy: Mutex::new(None),
            width: AtomicU32::new(window.width()),
            height: AtomicU32::new(window.height()),
            resize: AtomicBool::new(window.resizable),
        });

        thread::spawn({
            let raw = raw.clone();

            move || {
                let options = ori::shell::platform::x11::X11RunOptions::new()
                    .with_window_parent(window.id(), parent);

                let app = App::build()
                    .delegate(EditorHandle { raw: raw.clone() })
                    .window(window, Self::ui);

                if let Err(err) = ori::shell::platform::x11::run(app, &mut state.clone(), options) {
                    println!("Error: {}", err);
                }
            }
        });

        Self { raw }
    }

    pub fn resize(&self, width: u32, height: u32) {
        let proxy = self.raw.proxy.lock().unwrap();

        if let Some(proxy) = &*proxy {
            proxy.cmd(Command::Resize(width, height));
        }
    }

    pub fn size(&self) -> (u32, u32) {
        let width = self.raw.width.load(Ordering::Relaxed);
        let height = self.raw.height.load(Ordering::Relaxed);

        (width, height)
    }

    pub fn quit(&self) {
        let proxy = self.raw.proxy.lock().unwrap();

        if let Some(proxy) = &*proxy {
            proxy.cmd(Command::Quit);
        }
    }

    pub fn resizable(&self) -> bool {
        self.raw.resize.load(Ordering::Relaxed)
    }

    fn ui<P: VstPlugin>(state: &mut Arc<PluginState<P>>) -> impl View<Arc<PluginState<P>>> {
        let view = state.plugin.lock().unwrap().ui();

        let view = focus(any(view), |state: &mut Arc<PluginState<P>>, lens| {
            let mut plugin = state.plugin.lock().unwrap();
            lens(&mut plugin)
        });

        on_event(view, |cx, _, event| {
            let handle = cx.context::<EditorHandle>();

            let resize = cx.window().resizable;
            handle.raw.resize.store(resize, Ordering::Relaxed);

            if let Event::WindowResized(e) = event {
                handle.raw.width.store(e.width, Ordering::Relaxed);
                handle.raw.height.store(e.height, Ordering::Relaxed);
            }

            if let Some(command) = event.cmd::<Command>() {
                match command {
                    Command::Quit => cx.cmd(AppCommand::Quit),
                    Command::Resize(width, height) => {
                        cx.window_mut().size = Size::new(*width as f32, *height as f32);
                        cx.layout();
                    }
                }
            }

            false
        })
    }
}

struct RawEditorHandle {
    proxy: Mutex<Option<CommandProxy>>,
    width: AtomicU32,
    height: AtomicU32,
    resize: AtomicBool,
}

enum Command {
    Quit,
    Resize(u32, u32),
}

impl<P: VstPlugin> AppDelegate<Arc<PluginState<P>>> for EditorHandle {
    fn init(&mut self, cx: &mut DelegateCx<Arc<PluginState<P>>>, _data: &mut Arc<PluginState<P>>) {
        self.raw.proxy.lock().unwrap().replace(cx.proxy());
        cx.insert_context(self.clone());
    }

    fn event(
        &mut self,
        _cx: &mut DelegateCx<Arc<PluginState<P>>>,
        _data: &mut Arc<PluginState<P>>,
        _event: &Event,
    ) -> bool {
        false
    }
}
