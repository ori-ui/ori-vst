use std::{ffi::c_void, ptr::NonNull, sync::Arc};

use ori::core::{view::View, window::Window};
use uuid::Uuid;
use vst3_sys::{
    base::{kResultOk, tresult, IPluginBase},
    vst::{IAudioProcessor, IComponent, IEditController},
    VST3,
};

use crate::{AudioLayout, Buffer, BufferLayout, Params, PluginState};

#[derive(Clone, Debug)]
pub struct PluginInfo {
    pub cid: Uuid,
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub url: String,
    pub email: String,
}

pub trait VstPlugin: Sized + Send + 'static {
    fn info() -> PluginInfo;

    fn layout(inputs: &[u32], outputs: &[u32]) -> Option<AudioLayout>;

    fn new() -> Self;

    fn params(&mut self) -> &mut dyn Params {
        unsafe { &mut *NonNull::<()>::dangling().as_ptr() }
    }

    fn window() -> Window {
        Window::new()
    }

    fn ui(&mut self) -> impl View<Self> + 'static;

    fn activate(&mut self, _audio_layout: &AudioLayout, _buffer_layout: &BufferLayout) {}

    fn deactivate(&mut self) {}

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        aux_buffers: &mut [Buffer<'_>],
        layout: BufferLayout,
    ) -> Status;
}

#[derive(Clone, Copy, Debug)]
pub enum Status {
    Done,
    Tail(u32),
    KeepAlive,
}

#[VST3(implements(IComponent, IEditController, IAudioProcessor))]
pub struct RawPlugin<P: VstPlugin> {
    pub state: Arc<PluginState<P>>,
}

impl<P: VstPlugin> RawPlugin<P> {
    pub fn new() -> Box<Self> {
        Self::allocate(Arc::new(PluginState::new()))
    }
}

impl<P: VstPlugin> IPluginBase for RawPlugin<P> {
    unsafe fn initialize(&self, _context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}
