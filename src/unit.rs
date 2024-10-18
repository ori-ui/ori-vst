use vst3_sys::{
    base::{tresult, IBStream},
    utils::SharedVstPtr,
    vst::{IUnitInfo, ProgramListInfo, UnitInfo},
};

use crate::{RawPlugin, VstPlugin};

impl<P: VstPlugin> IUnitInfo for RawPlugin<P> {
    unsafe fn get_unit_count(&self) -> i32 {
        todo!()
    }

    unsafe fn get_unit_info(&self, _unit_index: i32, _info: *mut UnitInfo) -> tresult {
        todo!()
    }

    unsafe fn get_program_list_count(&self) -> i32 {
        todo!()
    }

    unsafe fn get_program_list_info(
        &self,
        _list_index: i32,
        _info: *mut ProgramListInfo,
    ) -> tresult {
        todo!()
    }

    unsafe fn get_program_name(
        &self,
        _list_id: i32,
        _program_index: i32,
        _name: *mut u16,
    ) -> tresult {
        todo!()
    }

    unsafe fn get_program_info(
        &self,
        _list_id: i32,
        _program_index: i32,
        _attribute_id: *const u8,
        _attribute_value: *mut u16,
    ) -> tresult {
        todo!()
    }

    unsafe fn has_program_pitch_names(&self, _id: i32, _index: i32) -> tresult {
        todo!()
    }

    unsafe fn get_program_pitch_name(
        &self,
        _id: i32,
        _index: i32,
        _pitch: i16,
        _name: *mut u16,
    ) -> tresult {
        todo!()
    }

    unsafe fn get_selected_unit(&self) -> i32 {
        todo!()
    }

    unsafe fn select_unit(&self, _id: i32) -> tresult {
        todo!()
    }

    unsafe fn get_unit_by_bus(
        &self,
        _type_: i32,
        _dir: i32,
        _bus_index: i32,
        _channel: i32,
        _unit_id: *mut i32,
    ) -> tresult {
        todo!()
    }

    unsafe fn set_unit_program_data(
        &self,
        _list_or_unit: i32,
        _program_idx: i32,
        _data: SharedVstPtr<dyn IBStream>,
    ) -> tresult {
        todo!()
    }
}
