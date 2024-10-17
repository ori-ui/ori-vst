use std::ffi::c_void;

use vst3_sys::{
    base::{kResultOk, tresult, FIDString, IBStream},
    utils::SharedVstPtr,
    vst::{IComponentHandler, IEditController, ParameterInfo, TChar},
};

use crate::{Params, RawPlugin, RawView, VstPlugin};

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
        let mut plugin = self.state.plugin.lock().unwrap();
        let params = plugin.params();
        params.count() as i32
    }

    unsafe fn get_parameter_info(&self, index: i32, info: *mut ParameterInfo) -> tresult {
        kResultOk
    }

    unsafe fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
        string: *mut TChar,
    ) -> tresult {
        kResultOk
    }

    unsafe fn get_param_value_by_string(
        &self,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> tresult {
        kResultOk
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        value_normalized
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        plain_value
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        //let mut plugin = self.state.plugin.lock().unwrap();
        //let mut params = plugin.params();
        //let param = params.param(id as usize).unwrap();
        //param.normalized() as f64
        0.0
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        kResultOk
    }

    unsafe fn set_component_handler(
        &self,
        handler: SharedVstPtr<dyn IComponentHandler>,
    ) -> tresult {
        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        Box::into_raw(RawView::new(self.state.clone())) as *mut c_void
    }
}
