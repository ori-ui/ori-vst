use std::{
    ffi::{c_void, CStr},
    sync::{Arc, Mutex},
};
use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugView, ViewRect},
    VST3,
};

use crate::{editor::EditorHandle, PluginState, VstPlugin};

#[VST3(implements(IPlugView))]
pub struct RawView<P: VstPlugin> {
    state: Arc<PluginState<P>>,
    handle: Mutex<Option<EditorHandle>>,
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
        let mut handle = self.handle.lock().unwrap();

        if handle.is_some() {
            return kResultFalse;
        }

        let c_str = CStr::from_ptr(type_);

        let new_handle = match c_str.to_str() {
            #[cfg(target_os = "linux")]
            Ok(t) if t == "X11EmbedWindowID" => {
                EditorHandle::new_x11(self.state.clone(), parent as u32)
            }
            _ => return kResultFalse,
        };

        *handle = Some(new_handle);

        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        let mut handle = self.handle.lock().unwrap();

        if let Some(handle) = handle.take() {
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
        let handle = self.handle.lock().unwrap();

        if let Some(handle) = handle.as_ref() {
            let (width, height) = handle.size();

            (*size).top = 0;
            (*size).left = 0;
            (*size).bottom = height as i32;
            (*size).right = width as i32;
        }

        kResultOk
    }

    unsafe fn on_size(&self, new_size: *mut ViewRect) -> tresult {
        let handle = self.handle.lock().unwrap();

        if let Some(handle) = handle.as_ref() {
            handle.resize((*new_size).right as u32, (*new_size).bottom as u32);
        }

        kResultOk
    }

    unsafe fn on_focus(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_frame(&self, _frame: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn can_resize(&self) -> tresult {
        let handle = self.handle.lock().unwrap();

        if let Some(handle) = handle.as_ref() {
            if handle.resizable() {
                return kResultOk;
            }
        }

        kResultFalse
    }

    unsafe fn check_size_constraint(&self, _rect: *mut ViewRect) -> tresult {
        kResultOk
    }
}
