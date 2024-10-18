use std::ops::{Deref, DerefMut, RangeInclusive};

use crate::{Param, ParamFlags, Unit};

/// A floating-point parameter.
#[derive(Clone, Debug)]
pub struct Float {
    value: f32,
    default: f32,
    range: RangeInclusive<f32>,
    steps: Option<u32>,
    unit: Unit,
    flags: ParamFlags,
}

impl Float {
    /// Create a new floating-point parameter.
    pub fn new(default: f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value: default,
            default,
            range,
            steps: None,
            unit: Unit::Linear,
            flags: ParamFlags::empty(),
        }
    }

    /// Set the number of steps for the parameter.
    pub fn with_steps(mut self, steps: u32) -> Self {
        self.steps = Some(steps);
        self
    }

    /// Set the step size for the parameter.
    pub fn with_step_size(self, step_size: f32) -> Self {
        let steps = (*self.range.end() - *self.range.start()) / step_size;
        self.with_steps(steps as u32)
    }

    /// Set the unit for the parameter.
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }

    /// Set the flags for the parameter.
    pub fn with_flags(mut self, flags: ParamFlags) -> Self {
        self.flags |= flags;
        self
    }

    /// Enable automation for the parameter.
    ///
    /// This is equivalent to calling `with_flags(ParamFlags::AUTOMATE)`.
    pub fn with_automate(self) -> Self {
        self.with_flags(ParamFlags::AUTOMATE)
    }

    /// Enable read-only mode for the parameter.
    ///
    /// This is equivalent to calling `with_flags(ParamFlags::READ_ONLY)`.
    pub fn with_read_only(self) -> Self {
        self.with_flags(ParamFlags::READ_ONLY)
    }
}

impl Deref for Float {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Float {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl Param for Float {
    fn get(&self) -> f32 {
        self.value
    }

    fn set(&mut self, value: f32) {
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

    fn unit(&self) -> Unit {
        self.unit.clone()
    }

    fn flags(&self) -> ParamFlags {
        ParamFlags::empty()
    }
}
