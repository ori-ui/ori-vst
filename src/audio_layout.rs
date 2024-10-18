use crate::Buffer;

/// The layout of the audio ports.
#[derive(Clone, Debug, Default)]
pub struct AudioLayout {
    /// The main input port.
    pub input: Option<AudioPort>,

    /// The main output port.
    pub output: Option<AudioPort>,

    /// The auxiliary input ports.
    pub aux_input: Vec<AudioPort>,

    /// The auxiliary output ports.
    pub aux_output: Vec<AudioPort>,
}

impl AudioLayout {
    /// Create a new audio layout.
    pub fn new() -> AudioLayout {
        AudioLayout::default()
    }

    /// Set the main input port.
    pub fn with_input(mut self, input: AudioPort) -> AudioLayout {
        self.input = Some(input);
        self
    }

    /// Set the main output port.
    pub fn with_output(mut self, output: AudioPort) -> AudioLayout {
        self.output = Some(output);
        self
    }

    /// Add an auxiliary input port.
    pub fn with_aux_input(mut self, aux_input: AudioPort) -> AudioLayout {
        self.aux_input.push(aux_input);
        self
    }

    /// Add an auxiliary output port.
    pub fn with_aux_output(mut self, aux_output: AudioPort) -> AudioLayout {
        self.aux_output.push(aux_output);
        self
    }

    /// Check if the layout has a main buffer.
    pub fn has_main_buffer(&self) -> bool {
        self.input.is_some() || self.output.is_some()
    }

    /// Get the number of auxiliary buffers.
    pub fn aux_buffers(&self) -> usize {
        usize::max(self.aux_input.len(), self.aux_output.len())
    }

    /// Get the number of buffers.
    pub fn buffers(&self) -> usize {
        self.aux_buffers() + self.has_main_buffer() as usize
    }

    /// Get the number of input channels.
    pub fn input_channels(&self) -> u32 {
        self.input.as_ref().map(|p| p.channels).unwrap_or(0)
    }

    /// Get the number of output channels.
    pub fn output_channels(&self) -> u32 {
        self.output.as_ref().map(|p| p.channels).unwrap_or(0)
    }

    /// Get the number of input busses.
    pub fn input_busses(&self) -> u32 {
        self.input.is_some() as u32 + self.aux_input.len() as u32
    }

    /// Get the number of output busses.
    pub fn output_busses(&self) -> u32 {
        self.output.is_some() as u32 + self.aux_output.len() as u32
    }

    /// Get the input port at the given index.
    ///
    /// The main input port, if present, is at index 0. The auxiliary input ports follow.
    pub fn input_port(&self, mut index: u32) -> Option<&AudioPort> {
        if index == 0 && self.input.is_some() {
            return self.input.as_ref();
        }

        if self.input.is_some() {
            index -= 1;
        }

        self.aux_input.get(index as usize)
    }

    /// Get the output port at the given index.
    ///
    /// The main output port, if present, is at index 0. The auxiliary output ports follow.
    pub fn output_port(&self, mut index: u32) -> Option<&AudioPort> {
        if index == 0 && self.output.is_some() {
            return self.output.as_ref();
        }

        if self.output.is_some() {
            index -= 1;
        }

        self.aux_output.get(index as usize)
    }

    /// Check if the input at the given index is the main input.
    pub fn is_input_main(&self, index: u32) -> bool {
        index == 0 && self.input.is_some()
    }

    /// Check if the output at the given index is the main output.
    pub fn is_output_main(&self, index: u32) -> bool {
        index == 0 && self.output.is_some()
    }

    /// Get the name of the input at the given index.
    pub fn input_name(&self, mut index: u32) -> String {
        let Some(port) = self.input_port(index) else {
            return format!("Sidechain Input {}", index);
        };

        match port.name {
            Some(ref name) => name.clone(),
            None if self.is_input_main(index) => String::from("Input"),
            None => {
                if self.input.is_some() {
                    index -= 1;
                }

                format!("Sidechain Input {}", index)
            }
        }
    }

    /// Get the name of the output at the given index.
    pub fn output_name(&self, mut index: u32) -> String {
        let Some(port) = self.output_port(index) else {
            return format!("Aux Output {}", index);
        };

        match port.name {
            Some(ref name) => name.clone(),
            None if self.is_output_main(index) => String::from("Output"),
            None => {
                if self.output.is_some() {
                    index -= 1;
                }

                format!("Aux Output {}", index)
            }
        }
    }
}

/// The layout of an audio port.
#[derive(Clone, Debug)]
pub struct AudioPort {
    /// The number of channels.
    pub channels: u32,

    /// The name of the port.
    pub name: Option<String>,
}

impl AudioPort {
    /// Create a new audio port.
    pub fn new(channels: u32) -> AudioPort {
        AudioPort {
            channels,
            name: None,
        }
    }

    /// Create a new named audio port.
    pub fn named(channels: u32, name: impl Into<String>) -> AudioPort {
        AudioPort {
            channels,
            name: Some(name.into()),
        }
    }
}

/// A collection of audio buffers.
#[derive(Debug, Default)]
pub(crate) struct Buffers {
    channels: Vec<Vec<*mut [f32]>>,
    buffers: Vec<Buffer<'static>>,
}

unsafe impl Send for Buffers {}

impl Buffers {
    /// Create a new collection of audio buffers.
    pub(crate) fn new() -> Buffers {
        Buffers::default()
    }

    /// Allocate the buffers for the given layout.
    pub(crate) fn allocate(&mut self, layout: &AudioLayout) {
        self.channels.resize(layout.buffers(), Vec::new());
        self.buffers.resize_with(layout.buffers(), Buffer::empty);

        for i in 0..layout.buffers() {
            let input = layout.input_port(i as u32);
            let output = layout.output_port(i as u32);

            let channels = usize::max(
                input.map(|p| p.channels as usize).unwrap_or(0),
                output.map(|p| p.channels as usize).unwrap_or(0),
            );

            self.channels[i].resize(channels, &mut []);
        }
    }

    ///
    /// # Safety
    /// - The buffers aren't initialized, `set_channel` must be called for each channel.
    /// - The buffers may not live longer than `self`.
    pub(crate) unsafe fn get(&mut self, samples: usize) -> &mut [Buffer<'static>] {
        for (i, buffer) in self.buffers.iter_mut().enumerate() {
            let channels = self.channels[i].as_mut_slice() as *mut _ as *mut _;
            *buffer = Buffer::new(samples, &mut *channels);
        }

        self.buffers.as_mut_slice()
    }
}
