use std::ops::{Deref, DerefMut, RangeInclusive};

use crate::{Param, ParamFlags, Unit};

/// A floating-point parameter.
#[derive(Clone, Debug)]
pub struct Float {
    /// The name of the parameter.
    pub name: Option<String>,

    /// The short name of the parameter.
    pub short: Option<String>,

    /// The value of the parameter.
    pub value: f32,

    /// The default value of the parameter.
    pub default: f32,

    /// The range of the parameter.
    pub range: RangeInclusive<f32>,

    /// The number of steps for the parameter.
    pub steps: Option<u32>,

    /// The unit of the parameter.
    pub unit: Unit,

    /// The flags of the parameter.
    pub flags: ParamFlags,
}

impl Float {
    /// Create a new floating-point parameter.
    pub fn new(default: f32, range: RangeInclusive<f32>) -> Self {
        Self {
            name: None,
            short: None,
            value: default,
            default,
            range,
            steps: None,
            unit: Unit::Linear,
            flags: ParamFlags::empty(),
        }
    }

    /// Set the name of the parameter.
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the short name of the parameter.
    pub fn short(mut self, short: impl ToString) -> Self {
        self.short = Some(short.to_string());
        self
    }

    /// Set the number of steps for the parameter.
    pub fn steps(mut self, steps: u32) -> Self {
        self.steps = Some(steps);
        self
    }

    /// Set the step size for the parameter.
    pub fn step_size(self, step_size: f32) -> Self {
        let steps = (*self.range.end() - *self.range.start()) / step_size;
        self.steps(steps as u32)
    }

    /// Set the unit for the parameter.
    pub fn unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }

    /// Set the flags for the parameter.
    pub fn flags(mut self, flags: ParamFlags) -> Self {
        self.flags |= flags;
        self
    }

    /// Enable automation for the parameter.
    ///
    /// This is equivalent to calling `with_flags(ParamFlags::AUTOMATE)`.
    pub fn automate(self) -> Self {
        self.flags(ParamFlags::AUTOMATE)
    }

    /// Enable read-only mode for the parameter.
    ///
    /// This is equivalent to calling `with_flags(ParamFlags::READ_ONLY)`.
    pub fn read_only(self) -> Self {
        self.flags(ParamFlags::READ_ONLY)
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

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn short(&self) -> Option<&str> {
        self.short.as_deref()
    }

    fn unit(&self) -> Unit {
        self.unit.clone()
    }

    fn flags(&self) -> ParamFlags {
        ParamFlags::empty()
    }
}
