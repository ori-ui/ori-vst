use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};

use crate::{AudioLayout, BufferLayout, Buffers, Status, VstPlugin};

pub struct PluginState<P: VstPlugin> {
    pub plugin: Mutex<P>,
    pub audio_layout: Mutex<Arc<AudioLayout>>,
    pub buffer_layout: Mutex<Option<BufferLayout>>,
    pub buffers: Mutex<Buffers>,
    pub status: Mutex<Status>,
    pub latency: AtomicU32,
    pub processing: AtomicBool,
}

impl<P: VstPlugin> Default for PluginState<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: VstPlugin> PluginState<P> {
    pub fn new() -> Self {
        let plugin = P::new();

        Self {
            plugin: Mutex::new(plugin),
            audio_layout: Mutex::new(Arc::new(AudioLayout::default())),
            buffer_layout: Mutex::new(None),
            buffers: Mutex::new(Buffers::new()),
            status: Mutex::new(Status::Done),
            latency: AtomicU32::new(0),
            processing: AtomicBool::new(false),
        }
    }

    pub fn audio_layout(&self) -> Arc<AudioLayout> {
        self.audio_layout.lock().unwrap().clone()
    }

    pub fn set_audio_layout(&self, layout: AudioLayout) {
        *self.audio_layout.lock().unwrap() = Arc::new(layout);
    }

    pub fn buffer_layout(&self) -> Option<BufferLayout> {
        self.buffer_layout.lock().unwrap().clone()
    }

    pub fn set_buffer_layout(&self, layout: Option<BufferLayout>) {
        *self.buffer_layout.lock().unwrap() = layout;
    }

    pub fn status(&self) -> Status {
        *self.status.lock().unwrap()
    }

    pub fn set_status(&self, status: Status) {
        *self.status.lock().unwrap() = status;
    }

    pub fn latency(&self) -> u32 {
        self.latency.load(Ordering::SeqCst)
    }

    pub fn set_latency(&self, latency: u32) {
        self.latency.store(latency, Ordering::SeqCst);
    }

    pub fn processing(&self) -> bool {
        self.processing.load(Ordering::SeqCst)
    }

    pub fn set_processing(&self, processing: bool) {
        self.processing.store(processing, Ordering::SeqCst);
    }

    pub fn allocate_buffers(&self, layout: &AudioLayout) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.allocate(layout);
    }
}
