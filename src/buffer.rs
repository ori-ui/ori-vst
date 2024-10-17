use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProcessMode {
    Realtime,
    Buffered,
    Offline,
}

#[derive(Clone, Debug)]
pub struct BufferLayout {
    pub sample_rate: f32,
    pub max_buffer_size: usize,
    pub mode: ProcessMode,
}

#[derive(Debug)]
pub struct Buffer<'a> {
    samples: usize,
    channels: &'a mut [&'a mut [f32]],
}

impl<'a> Buffer<'a> {
    pub fn new(samples: usize, channels: &'a mut [&'a mut [f32]]) -> Buffer<'a> {
        Buffer { samples, channels }
    }

    pub fn empty() -> Buffer<'static> {
        Buffer {
            samples: 0,
            channels: &mut [],
        }
    }

    pub fn channels(&self) -> usize {
        self.channels.len()
    }

    pub fn set_channel(&mut self, index: usize, channel: &'a mut [f32]) {
        self.channels[index] = channel;
    }

    pub fn iter_samples(&mut self) -> SampleIter<'a> {
        SampleIter {
            buffers: self.channels as *mut _,
            samples: self.samples,
            sample: 0,
            marker: PhantomData,
        }
    }
}

pub struct SampleIter<'a> {
    buffers: *mut [&'a mut [f32]],
    samples: usize,
    sample: usize,
    marker: PhantomData<&'a mut &'a mut ()>,
}

impl<'a> Iterator for SampleIter<'a> {
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
