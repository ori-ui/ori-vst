use std::{
    ffi::{c_void, CStr},
    mem,
    sync::Arc,
};

use parking_lot::Mutex;
use vst3_com::VstPtr;
use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, kResultTrue, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, ViewRect},
    utils::SharedVstPtr,
    VST3,
};

use crate::{PluginState, VstPlugin};

#[VST3(implements(IPlugView))]
pub struct RawView<P: VstPlugin> {
    state: Arc<PluginState<P>>,
    frame: Mutex<Option<VstPtr<dyn IPlugFrame>>>,
}

impl<P: VstPlugin> RawView<P> {
    pub fn new(state: Arc<PluginState<P>>) -> Box<Self> {
        Self::allocate(state, Mutex::new(None))
    }
}

impl<P: VstPlugin> IPlugView for RawView<P> {
    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        let c_str = CStr::from_ptr(type_);

        match c_str.to_str() {
            #[cfg(target_os = "windows")]
            Ok(t) if t == "HWND" => kResultOk,

            #[cfg(target_os = "macos")]
            Ok(t) if t == "NSView" => kResultOk,

            #[cfg(target_os = "linux")]
            Ok(t) if t == "X11EmbedWindowID" => kResultOk,
            _ => kResultFalse,
        }
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        let mut editor = self.state.editor.lock();

        if editor.is_some() {
            return kResultFalse;
        }

        let c_str = CStr::from_ptr(type_);

        let new_handle = match c_str.to_str() {
            #[cfg(target_os = "linux")]
            Ok(t) if t == "X11EmbedWindowID" => {
                crate::x11::spawn_editor(self.state.clone(), parent)
            }
            _ => return kResultFalse,
        };

        *editor = Some(new_handle);

        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        let mut editor = self.state.editor.lock();

        if let Some(handle) = editor.take() {
            handle.quit();
            return kResultOk;
        }

        kResultFalse
    }

    unsafe fn on_wheel(&self, _distance: f32) -> tresult {
        kResultOk
    }

    unsafe fn on_key_down(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        kResultOk
    }

    unsafe fn on_key_up(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        kResultOk
    }

    unsafe fn get_size(&self, size: *mut ViewRect) -> tresult {
        let editor = self.state.editor.lock();

        if let Some(handle) = editor.as_ref() {
            let (width, height) = handle.size();

            (*size).top = 0;
            (*size).left = 0;
            (*size).bottom = height as i32;
            (*size).right = width as i32;
        }

        kResultOk
    }

    unsafe fn on_size(&self, new_size: *mut ViewRect) -> tresult {
        let editor = self.state.editor.lock();

        if let Some(editor) = editor.as_ref() {
            let width = (*new_size).right - (*new_size).left;
            let height = (*new_size).bottom - (*new_size).top;

            let width = width as u32;
            let height = height as u32;

            if !editor.resizable() && (width, height) != editor.size() {
                return kResultFalse;
            }

            editor.resize(width, height);
        }

        kResultOk
    }

    unsafe fn on_focus(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_frame(&self, frame: *mut c_void) -> tresult {
        let frame: SharedVstPtr<dyn IPlugFrame> = mem::transmute(frame);
        match frame.upgrade() {
            Some(frame) => {
                self.frame.lock().replace(frame);
            }
            None => {
                self.frame.lock().take();
            }
        }

        kResultOk
    }

    unsafe fn can_resize(&self) -> tresult {
        let editor = self.state.editor.lock();

        if let Some(handle) = editor.as_ref() {
            if handle.resizable() {
                return kResultTrue;
            }
        }

        kResultFalse
    }

    unsafe fn check_size_constraint(&self, _rect: *mut ViewRect) -> tresult {
        kResultOk
    }
}
