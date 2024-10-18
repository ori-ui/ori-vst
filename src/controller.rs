use std::{ffi::c_void, slice};

use vst3_sys::{
    base::{kResultOk, tresult, FIDString, IBStream},
    utils::SharedVstPtr,
    vst::{IComponentHandler, IEditController, ParameterInfo, TChar},
};

use crate::{util, RawPlugin, RawView, VstPlugin};

impl<P: VstPlugin> IEditController for RawPlugin<P> {
    unsafe fn set_component_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn set_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        let mut plugin = self.state.plugin.lock();
        plugin.params().count() as i32
    }

    unsafe fn get_parameter_info(&self, index: i32, out_info: *mut ParameterInfo) -> tresult {
        let mut plugin = self.state.plugin.lock();

        if let Some(info) = plugin.params().info(index as usize) {
            let out_info = &mut *out_info;

            out_info.id = index as u32;
            util::u16strcpy(&info.name, &mut out_info.title);
            util::u16strcpy(&info.name, &mut out_info.short_title);
            util::u16strcpy(info.unit.label(), &mut out_info.units);
            out_info.unit_id = info.unit.id();
            out_info.step_count = info.step_count;
            out_info.default_normalized_value = info.default_normalized as f64;
            out_info.flags = info.flags.bits() as i32;
        }

        kResultOk
    }

    unsafe fn get_param_string_by_value(
        &self,
        id: u32,
        normalized: f64,
        string: *mut TChar,
    ) -> tresult {
        let mut plugin = self.state.plugin.lock();
        let param = plugin.params().param(id as usize).unwrap();
        let plain = param.plain(normalized as f32);

        let s = param.to_string(plain);

        let string = slice::from_raw_parts_mut(string, 128);
        string.fill(0);
        util::u16strcpy(&s, string);

        kResultOk
    }

    unsafe fn get_param_value_by_string(
        &self,
        id: u32,
        string: *const TChar,
        normalized: *mut f64,
    ) -> tresult {
        let len = util::u16strlen(string);

        let string = slice::from_raw_parts(string.cast(), len);
        let s = String::from_utf16_lossy(string);

        let mut plugin = self.state.plugin.lock();
        let param = plugin.params().param(id as usize).unwrap();
        let plain = param.from_string(&s);
        *normalized = param.normalize(plain) as f64;

        kResultOk
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, normalized: f64) -> f64 {
        let mut plugin = self.state.plugin.lock();
        let param = plugin.params().param(id as usize).unwrap();
        param.plain(normalized as f32) as f64
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain: f64) -> f64 {
        let mut plugin = self.state.plugin.lock();
        let param = plugin.params().param(id as usize).unwrap();
        param.normalize(plain as f32) as f64
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        let mut plugin = self.state.plugin.lock();
        let param = plugin.params().param(id as usize).unwrap();
        let plain = param.get();
        param.normalize(plain) as f64
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        let mut plugin = self.state.plugin.lock();
        let param = plugin.params().param(id as usize).unwrap();

        let plain = param.plain(value as f32);
        param.set(plain);

        if let Some(editor) = self.state.editor.lock().as_ref() {
            editor.rebuild();
        }

        kResultOk
    }

    unsafe fn set_component_handler(
        &self,
        handler: SharedVstPtr<dyn IComponentHandler>,
    ) -> tresult {
        self.control.lock().replace(handler);

        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        Box::into_raw(RawView::new(self.state.clone())) as *mut c_void
    }
}

impl<P: VstPlugin> RawPlugin<P> {}
