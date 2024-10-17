use std::ops::RangeInclusive;

pub trait Param {
    /// Get the normalized value of the parameter.
    fn normalized(&self) -> f32;

    /// Set the normalized value of the parameter.
    fn set_normalized(&mut self, normalized: f32);

    /// Get the default normalized value of the parameter.
    fn default(&self) -> f32;

    /// Convert a normalized value to a plain value.
    fn plain(&self, normalized: f32) -> f32;

    /// Convert a plain value to a normalized value.
    fn normalize(&self, plain: f32) -> f32;

    fn to_string(&self, normalized: f32) -> String {
        self.plain(normalized).to_string()
    }
}

pub trait Params {
    fn count(&self) -> usize;

    fn param(&mut self, index: usize) -> Option<&mut dyn Param>;
}

impl Params for () {
    fn count(&self) -> usize {
        0
    }

    fn param(&mut self, _index: usize) -> Option<&mut dyn Param> {
        None
    }
}

pub struct F32 {
    value: f32,
    default: f32,
    range: RangeInclusive<f32>,
}

impl Param for F32 {
    fn normalized(&self) -> f32 {
        self.value
    }

    fn set_normalized(&mut self, value: f32) {
        self.value = value;
    }

    fn default(&self) -> f32 {
        self.default
    }

    fn plain(&self, normalized: f32) -> f32 {
        let size = *self.range.end() - *self.range.start();
        *self.range.start() + normalized * size
    }

    fn normalize(&self, plain: f32) -> f32 {
        let size = *self.range.end() - *self.range.start();
        (plain - *self.range.start()) / size
    }
}
