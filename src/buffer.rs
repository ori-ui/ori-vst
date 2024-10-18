use std::marker::PhantomData;

/// The processing mode of a buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProcessMode {
    /// Real-time processing.
    Realtime,

    /// Buffered processing.
    Buffered,

    /// Offline processing.
    Offline,
}

/// The layout of a buffer.
#[derive(Clone, Debug)]
pub struct BufferLayout {
    /// The sample rate of the buffer.
    ///
    /// This is the number of samples per second.
    pub sample_rate: f32,

    /// The maximum buffer size.
    pub max_buffer_size: usize,

    /// The processing mode of the buffer.
    pub mode: ProcessMode,
}

/// A buffer of audio samples.
#[derive(Debug)]
pub struct Buffer<'a> {
    samples: usize,
    channels: &'a mut [&'a mut [f32]],
}

impl<'a> Buffer<'a> {
    /// Create a new buffer.
    ///
    /// Each channel should contain `samples` samples.
    pub fn new(samples: usize, channels: &'a mut [&'a mut [f32]]) -> Buffer<'a> {
        Buffer { samples, channels }
    }

    /// Create an empty buffer.
    pub fn empty() -> Buffer<'static> {
        Buffer {
            samples: 0,
            channels: &mut [],
        }
    }

    /// Get the number of samples in the buffer.
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Get the number of samples in the buffer.
    pub fn channels(&self) -> usize {
        self.channels.len()
    }

    /// Set the channel at the given `index`.
    ///
    /// The channel should contain `self.samples()` samples.
    pub fn set_channel(&mut self, index: usize, channel: &'a mut [f32]) {
        self.channels[index] = channel;
    }

    /// Get an iterator over the samples in the buffer.
    pub fn iter_samples(&mut self) -> ChannelsIter<'a> {
        ChannelsIter {
            buffers: self.channels as *mut _,
            samples: self.samples,
            sample: 0,
            marker: PhantomData,
        }
    }
}

/// An iterator over the channels of a buffer.
pub struct ChannelsIter<'a> {
    buffers: *mut [&'a mut [f32]],
    samples: usize,
    sample: usize,
    marker: PhantomData<&'a mut &'a mut ()>,
}

impl<'a> Iterator for ChannelsIter<'a> {
    type Item = ChannelIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sample >= self.samples {
            return None;
        }

        let channels = ChannelIter {
            buffers: self.buffers,
            sample: self.sample,
            channel: 0,
            marker: PhantomData,
        };

        self.sample += 1;

        Some(channels)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.samples - self.sample;
        (remaining, Some(remaining))
    }
}

/// An iterator over the samples of a channel.
pub struct ChannelIter<'a> {
    buffers: *mut [&'a mut [f32]],
    sample: usize,
    channel: usize,
    marker: PhantomData<&'a mut &'a mut ()>,
}

impl<'a> Iterator for ChannelIter<'a> {
    type Item = &'a mut f32;

    fn next(&mut self) -> Option<Self::Item> {
        let buffers = unsafe { &mut *self.buffers };

        if self.channel >= buffers.len() {
            return None;
        }

        let sample = &mut buffers[self.channel][self.sample];

        self.channel += 1;

        Some(sample)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let buffers = unsafe { &*self.buffers };
        let remaining = buffers.len() - self.channel;
        (remaining, Some(remaining))
    }
}
