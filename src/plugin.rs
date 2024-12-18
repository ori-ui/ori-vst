#![allow(missing_docs)]

use std::{ffi::c_void, ptr::NonNull, sync::Arc};

use ori::core::{view::View, window::Window};
use uuid::Uuid;
use vst3_sys::{
    base::{kResultOk, tresult, IPluginBase},
    vst::{IAudioProcessor, IComponent, IEditController},
    VST3,
};

use crate::{AudioLayout, Buffer, BufferLayout, Params, PluginState};

/// A VST3 plugin.
pub trait VstPlugin: Sized + Send + 'static {
    /// Get the plugin information.
    fn info() -> Info;

    /// Get the audio layout of the plugin, given a set of input and output channel counts.
    fn layout(inputs: &[u32], outputs: &[u32]) -> Option<AudioLayout>;

    /// Get the default audio layout of the plugin.
    fn default_layout() -> AudioLayout {
        Self::layout(&[], &[]).unwrap_or_default()
    }

    /// Create a new instance of the plugin.
    fn new() -> Self;

    /// Get the parameters of the plugin.
    fn params(&mut self) -> &mut dyn Params {
        unsafe { &mut *NonNull::<()>::dangling().as_ptr() }
    }

    /// Create a new window.
    fn window() -> Window {
        Window::new()
    }

    /// Build the user interface of the plugin.
    fn ui(&mut self) -> impl View<Self> + 'static;

    /// Activate the plugin is activated.
    ///
    /// This allows the plugin to allocate any resources it needs.
    fn activate(&mut self, audio_layout: &AudioLayout, buffer_layout: &BufferLayout) -> Activate {
        let _ = (audio_layout, buffer_layout);

        Activate::new()
    }

    /// Deactivate the plugin.
    fn deactivate(&mut self) {}

    /// Reset the processing state of the plugin.
    fn reset(&mut self) {}

    /// Process the audio buffers.
    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        aux_buffers: &mut [Buffer<'_>],
        layout: BufferLayout,
    ) -> Process;
}

/// The plugin information.
#[derive(Clone, Debug)]
pub struct Info {
    /// The unique identifier of the plugin.
    pub uuid: Uuid,

    /// The name of the plugin.
    ///
    /// This must be not be longer than 128 characters.
    pub name: String,

    /// The subcategories of the plugin.
    pub subcategories: Vec<Subcategory>,

    /// The vendor of the plugin.
    ///
    /// This must be not be longer than 128 characters.
    pub vendor: String,

    /// The version of the plugin.
    ///
    /// This must be not be longer than 64 characters.
    pub version: String,

    /// The URL of the plugin.
    ///
    /// This must be not be longer than 128 characters.
    pub url: String,

    /// The email of the plugin.
    ///
    /// This must be not be longer than 128 characters.
    pub email: String,
}

/// The processing configuration.
#[derive(Clone, Debug)]
pub struct Activate {
    /// The number of latancy samples.
    pub latency: u32,
}

impl Default for Activate {
    fn default() -> Self {
        Self::new()
    }
}

impl Activate {
    /// Create a new processing configuration.
    pub fn new() -> Self {
        Self { latency: 0 }
    }

    /// Set the latency of the processing.
    pub fn with_latency(mut self, latency: u32) -> Self {
        self.latency = latency;
        self
    }
}

/// The processing status after processing a buffer.
#[derive(Clone, Copy, Debug)]
pub enum Process {
    /// The processing is done.
    Done,

    /// A finite number of samples still need to be processed, eg. for reverb.
    Tail(u32),

    /// Keep the processing state running.
    KeepAlive,
}

/// The category of the plugin.
#[derive(Clone, Copy, Debug)]
pub enum Subcategory {
    Fx,
    Instrument,
    Spatial,
    Analyzer,
    Delay,
    Distortion,
    Drum,
    Dynamics,
    Eq,
    External,
    Filter,
    Generator,
    Mastering,
    Modulation,
    Network,
    Piano,
    PitchShift,
    Restoration,
    Reverb,
    Sampler,
    Synth,
    Tools,
    UpDownMix,
    Mono,
    Stereo,
    Surround,
    Ambisonic,
    Custom(&'static str),
}

impl Subcategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Fx => "Fx",
            Self::Instrument => "Instrument",
            Self::Spatial => "Spatial",
            Self::Analyzer => "Analyzer",
            Self::Delay => "Delay",
            Self::Distortion => "Distortion",
            Self::Drum => "Drum",
            Self::Dynamics => "Dynamics",
            Self::Eq => "EQ",
            Self::External => "External",
            Self::Filter => "Filter",
            Self::Generator => "Generator",
            Self::Mastering => "Mastering",
            Self::Modulation => "Modulation",
            Self::Network => "Network",
            Self::Piano => "Piano",
            Self::PitchShift => "Pitch Shift",
            Self::Restoration => "Restoration",
            Self::Reverb => "Reverb",
            Self::Sampler => "Sampler",
            Self::Synth => "Synth",
            Self::Tools => "Tools",
            Self::UpDownMix => "Up-Downmix",
            Self::Mono => "Mono",
            Self::Stereo => "Stereo",
            Self::Surround => "Surround",
            Self::Ambisonic => "Ambisonics",
            Self::Custom(category) => category,
        }
    }
}

/// A raw wrapper around a VST3 plugin.
///
/// This should never be used directly.
#[VST3(implements(IComponent, IEditController, IAudioProcessor))]
pub struct RawPlugin<P: VstPlugin> {
    /// The state of the plugin.
    pub(crate) state: Arc<PluginState<P>>,
}

impl<P: VstPlugin> RawPlugin<P> {
    /// Create a new raw plugin.
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
