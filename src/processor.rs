use std::{ptr, slice};

use vst3_sys::{
    base::{kInvalidArgument, kResultFalse, kResultOk, tresult, TBool},
    vst::{
        k50, k51, k70Cine, k71Cine, kEmpty, kMono, kStereo, AudioBusBuffers, BusDirection,
        BusDirections, IAudioProcessor, ProcessData, ProcessModes, ProcessSetup,
        SpeakerArrangement, SymbolicSampleSizes,
    },
};

use crate::{Buffer, BufferLayout, ProcessMode, RawPlugin, Process, VstPlugin};

const K_INPUT: i32 = BusDirections::kInput as i32;
const K_OUTPUT: i32 = BusDirections::kOutput as i32;

const K_REALTIME: i32 = ProcessModes::kRealtime as i32;
const K_PREFETCH: i32 = ProcessModes::kPrefetch as i32;
const K_OFFLINE: i32 = ProcessModes::kOffline as i32;

fn channel_count(channels: u32) -> u64 {
    match channels {
        0 => kEmpty,
        1 => kMono,
        2 => kStereo,
        5 => k50,
        6 => k51,
        7 => k70Cine,
        8 => k71Cine,
        n => (1 << n) - 1,
    }
}

impl<P: VstPlugin> IAudioProcessor for RawPlugin<P> {
    unsafe fn set_bus_arrangements(
        &self,
        inputs_ptr: *mut SpeakerArrangement,
        num_ins: i32,
        outputs_ptr: *mut SpeakerArrangement,
        num_outs: i32,
    ) -> tresult {
        if num_ins < 0 || num_outs < 0 {
            return kInvalidArgument;
        }

        let mut inputs = Vec::with_capacity(num_ins as usize);
        let mut outputs = Vec::with_capacity(num_outs as usize);

        for i in 0..num_ins {
            inputs.push(*inputs_ptr.add(i as usize) as u32);
        }

        for i in 0..num_outs {
            outputs.push(*outputs_ptr.add(i as usize) as u32);
        }

        if let Some(layout) = P::layout(&inputs, &outputs) {
            self.state.set_audio_layout(layout);

            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn get_bus_arrangement(
        &self,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> tresult {
        let layout = self.state.audio_layout();

        match dir {
            K_INPUT => {
                if let Some(port) = layout.input_port(index as u32) {
                    *arr = channel_count(port.channels);

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            K_OUTPUT => {
                if let Some(port) = layout.output_port(index as u32) {
                    *arr = channel_count(port.channels);

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn can_process_sample_size(&self, symbolic_sample_size: i32) -> tresult {
        if symbolic_sample_size == SymbolicSampleSizes::kSample32 as i32 {
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn get_latency_samples(&self) -> u32 {
        self.state.latency()
    }

    unsafe fn setup_processing(&self, setup: *const ProcessSetup) -> tresult {
        let setup = &*setup;

        let mode = match setup.process_mode {
            K_REALTIME => ProcessMode::Realtime,
            K_PREFETCH => ProcessMode::Buffered,
            K_OFFLINE => ProcessMode::Offline,
            _ => return kInvalidArgument,
        };

        self.state.set_buffer_layout(Some(BufferLayout {
            sample_rate: setup.sample_rate as f32,
            max_buffer_size: setup.max_samples_per_block as usize,
            mode,
        }));

        kResultOk
    }

    unsafe fn set_processing(&self, state: TBool) -> tresult {
        let processing = state != 0;

        self.state.set_status(Process::Done);
        self.state.set_processing(processing);

        if processing {
            if let Some(mut plugin) = self.state.plugin.try_lock() {
                plugin.reset();
            }
        }

        kResultOk
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let data = &mut *data;

        let Some(buffer_layout) = self.state.buffer_layout() else {
            return kResultFalse;
        };

        let audio_layout = self.state.audio_layout();

        let samples = data.num_samples as usize;

        if is_param_flush(data) {
            return kResultOk;
        }

        let mut buffers = self.state.buffers.lock();
        let buffers = buffers.get(samples);

        let mut empty = Buffer::empty();

        let (main_buffer, aux_buffers) = if audio_layout.has_main_buffer() {
            let (main, aux) = buffers.split_at_mut(1);

            (&mut main[0], aux)
        } else {
            (&mut empty, buffers)
        };

        let mut input_index = 0;
        let mut output_index = 0;

        if audio_layout.has_main_buffer() {
            let input = if audio_layout.input.is_some() {
                let input = &mut *data.inputs.add(input_index);
                input_index += 1;

                Some(input)
            } else {
                None
            };

            let output = if audio_layout.output.is_some() {
                let output = &mut *data.outputs.add(output_index);
                output_index += 1;

                Some(output)
            } else {
                None
            };

            update_buffer(main_buffer, samples, input, output);
        }

        for i in 0..audio_layout.aux_buffers() {
            let input = if input_index < data.num_inputs as usize {
                let input = &mut *data.inputs.add(input_index);
                input_index += 1;

                Some(input)
            } else {
                None
            };

            let output = if output_index < data.num_outputs as usize {
                let output = &mut *data.outputs.add(output_index);
                output_index += 1;

                Some(output)
            } else {
                None
            };

            update_buffer(&mut aux_buffers[i], samples, input, output);
        }

        let mut plugin = self.state.plugin.lock();
        let status = plugin.process(main_buffer, aux_buffers, buffer_layout);
        self.state.set_status(status);

        kResultOk
    }

    unsafe fn get_tail_samples(&self) -> u32 {
        match self.state.status() {
            Process::Done => 0,
            Process::Tail(n) => n,
            Process::KeepAlive => u32::MAX,
        }
    }
}

fn is_param_flush(data: &mut ProcessData) -> bool {
    data.num_samples == 0 || data.num_outputs == 0 || data.outputs.is_null()
}

unsafe fn update_buffer(
    buffer: &mut Buffer,
    samples: usize,
    input: Option<&mut AudioBusBuffers>,
    output: Option<&mut AudioBusBuffers>,
) {
    match (input, output) {
        // nothing to do
        (None, None) => {}
        (None, Some(output)) => update_buffer_single(buffer, samples, output),
        (Some(input), None) => update_buffer_single(buffer, samples, input),
        (Some(input), Some(output)) => update_buffer_input_output(buffer, samples, input, output),
    }
}

unsafe fn update_buffer_single(buffer: &mut Buffer, samples: usize, audio: &mut AudioBusBuffers) {
    for i in 0..audio.num_channels {
        let audio = audio.buffers.add(i as usize) as *mut *mut f32;
        let audio = slice::from_raw_parts_mut(*audio, samples);
        buffer.set_channel(i as usize, audio);
    }
}

unsafe fn update_buffer_input_output(
    buffer: &mut Buffer,
    samples: usize,
    input: &mut AudioBusBuffers,
    output: &mut AudioBusBuffers,
) {
    for i in 0..buffer.channels() as i32 {
        let input_buffer = input.buffers.add(i as usize) as *mut *mut f32;
        let output_buffer = output.buffers.add(i as usize) as *mut *mut f32;

        if i >= input.num_channels {
            let output_buffer = slice::from_raw_parts_mut(*output_buffer, samples);
            buffer.set_channel(i as usize, output_buffer);
        } else if i >= output.num_channels {
            let input_buffer = slice::from_raw_parts_mut(*input_buffer, samples);
            buffer.set_channel(i as usize, input_buffer);
        } else {
            ptr::copy_nonoverlapping(*input_buffer, *output_buffer, samples);

            let output_buffer = slice::from_raw_parts_mut(*output_buffer, samples);
            buffer.set_channel(i as usize, output_buffer);
        }
    }
}
