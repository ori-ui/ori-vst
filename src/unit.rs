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

    unsafe fn get_unit_info(&self, unit_index: i32, info: *mut UnitInfo) -> tresult {
        todo!()
    }

    unsafe fn get_program_list_count(&self) -> i32 {
        todo!()
    }

    unsafe fn get_program_list_info(&self, list_index: i32, info: *mut ProgramListInfo) -> tresult {
        todo!()
    }

    unsafe fn get_program_name(&self, list_id: i32, program_index: i32, name: *mut u16) -> tresult {
        todo!()
    }

    unsafe fn get_program_info(
        &self,
        list_id: i32,
        program_index: i32,
        attribute_id: *const u8,
        attribute_value: *mut u16,
    ) -> tresult {
        todo!()
    }

    unsafe fn has_program_pitch_names(&self, id: i32, index: i32) -> tresult {
        todo!()
    }

    unsafe fn get_program_pitch_name(
        &self,
        id: i32,
        index: i32,
        pitch: i16,
        name: *mut u16,
    ) -> tresult {
        todo!()
    }

    unsafe fn get_selected_unit(&self) -> i32 {
        todo!()
    }

    unsafe fn select_unit(&self, id: i32) -> tresult {
        todo!()
    }

    unsafe fn get_unit_by_bus(
        &self,
        type_: i32,
        dir: i32,
        bus_index: i32,
        channel: i32,
        unit_id: *mut i32,
    ) -> tresult {
        todo!()
    }

    unsafe fn set_unit_program_data(
        &self,
        list_or_unit: i32,
        program_idx: i32,
        data: SharedVstPtr<dyn IBStream>,
    ) -> tresult {
        todo!()
    }
}
