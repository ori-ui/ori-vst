use ori_vst::prelude::*;

#[derive(Params)]
pub struct GainPlugin {
    #[param(name = "Gain", unit = Unit::Decibels)]
    gain: Float,
}

impl VstPlugin for GainPlugin {
    fn info() -> Info {
        Info {
            uuid: uuid!("4c38d5eb-aa45-4ce4-95ed-af8993b2557d"),
            name: String::from("Gain (Ori vst3)"),
            vendor: String::from("ChangeCaps Inc."),
            version: String::from("0.1.0"),
            url: String::from("https://example.com"),
            email: String::from("jeff@example.com"),
        }
    }

    fn layout(_inputs: &[u32], _outputs: &[u32]) -> Option<AudioLayout> {
        let layout = AudioLayout::new()
            .with_input(AudioPort::new(2))
            .with_output(AudioPort::new(2));

        Some(layout)
    }

    fn new() -> Self {
        Self {
            gain: Float::new(1.0, 0.0..=20.0),
        }
    }

    fn params(&mut self) -> &mut dyn Params {
        self
    }

    fn ui(&mut self) -> impl View<Self> + 'static {
        let slider = slider(*self.gain)
            .range(0.0..=20.0)
            .on_input(|cx, data: &mut Self, value| {
                *data.gain = value;
                cx.rebuild();
            });

        let gain = text(format!("Gain: {}", *self.gain));

        center(vstack![slider, gain])
    }

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux_buffers: &mut [Buffer<'_>],
        _layout: BufferLayout,
    ) -> Process {
        for samples in buffer.iter_samples() {
            for sample in samples {
                *sample *= *self.gain;
            }
        }

        Process::Done
    }
}

vst3!(GainPlugin);
