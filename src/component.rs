use vst3_com::IID;
use vst3_sys::{
    base::{kInvalidArgument, kNoInterface, kResultFalse, kResultOk, tresult, IBStream, TBool},
    utils::SharedVstPtr,
    vst::{
        BusDirection, BusDirections, BusFlags, BusInfo, BusTypes, IComponent, IoMode, MediaType,
        MediaTypes, RoutingInfo,
    },
};

use crate::{util, RawPlugin, VstPlugin};

const K_AUDIO: i32 = MediaTypes::kAudio as i32;

const K_INPUT: i32 = BusDirections::kInput as i32;
const K_OUTPUT: i32 = BusDirections::kOutput as i32;

impl<P: VstPlugin> IComponent for RawPlugin<P> {
    unsafe fn get_controller_class_id(&self, _tuid: *mut IID) -> tresult {
        kNoInterface
    }

    unsafe fn set_io_mode(&self, _mode: IoMode) -> tresult {
        kResultOk
    }

    unsafe fn get_bus_count(&self, ty: MediaType, dir: BusDirection) -> i32 {
        let layout = self.state.audio_layout();

        match (ty, dir) {
            (K_AUDIO, K_INPUT) => layout.input_busses() as i32,
            (K_AUDIO, K_OUTPUT) => layout.output_busses() as i32,
            _ => 0,
        }
    }

    unsafe fn get_bus_info(
        &self,
        ty: MediaType,
        dir: BusDirection,
        index: i32,
        info: *mut BusInfo,
    ) -> tresult {
        let layout = self.state.audio_layout();

        match (ty, dir, index) {
            (K_AUDIO, K_INPUT, _) => {
                let info = &mut *info;
                info.media_type = K_AUDIO;
                info.direction = K_INPUT;
                info.flags = BusFlags::kDefaultActive as u32;

                if let Some(port) = layout.input_port(index as u32) {
                    if layout.is_input_main(index as u32) {
                        info.bus_type = BusTypes::kMain as i32;
                    } else {
                        info.bus_type = BusTypes::kAux as i32;
                    }

                    info.channel_count = port.channels as i32;

                    let name = layout.input_name(index as u32);
                    util::u16strcpy(&name, &mut info.name);

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            (K_AUDIO, K_OUTPUT, _) => {
                let info = &mut *info;
                info.media_type = K_AUDIO;
                info.direction = K_OUTPUT;
                info.flags = BusFlags::kDefaultActive as u32;

                if let Some(port) = layout.output_port(index as u32) {
                    if layout.is_output_main(index as u32) {
                        info.bus_type = BusTypes::kMain as i32;
                    } else {
                        info.bus_type = BusTypes::kAux as i32;
                    }

                    info.channel_count = port.channels as i32;

                    let name = layout.output_name(index as u32);
                    util::u16strcpy(&name, &mut info.name);

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn get_routing_info(
        &self,
        in_info: *mut RoutingInfo,
        out_info: *mut RoutingInfo,
    ) -> tresult {
        let layout = self.state.audio_layout();

        let in_info = &*in_info;
        let out_info = &mut *out_info;

        match (in_info.media_type, in_info.bus_index) {
            (K_AUDIO, 0) if layout.input.is_some() && layout.output.is_some() => {
                out_info.media_type = K_AUDIO;
                out_info.bus_index = in_info.bus_index;
                out_info.channel = in_info.channel;

                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn activate_bus(
        &self,
        _type_: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: TBool,
    ) -> tresult {
        kResultOk
    }

    unsafe fn set_active(&self, state: TBool) -> tresult {
        let mut plugin = self.state.plugin.lock();

        if state != 1 {
            plugin.deactivate();
            return kResultOk;
        }

        if let Some(buffer_layout) = self.state.buffer_layout() {
            let audio_layout = self.state.audio_layout();
            self.state.allocate_buffers(&audio_layout);

            let config = plugin.activate(&audio_layout, &buffer_layout);
            self.state.set_latency(config.latency);

            return kResultOk;
        }

        kResultFalse
    }

    unsafe fn set_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }
}
