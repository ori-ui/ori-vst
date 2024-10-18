pub use ori_vst_macro::Params;

/// A parameter.
pub trait Param {
    /// Get the normalized value of the parameter.
    fn get(&self) -> f32;

    /// Set the normalized value of the parameter.
    fn set(&mut self, plain: f32);

    /// Get the default plain value of the parameter.
    fn default(&self) -> f32;

    /// Convert a normalized value to a plain value.
    fn plain(&self, normalized: f32) -> f32;

    /// Convert a plain value to a normalized value.
    fn normalize(&self, plain: f32) -> f32;

    /// Get the default normalized value of the parameter.
    fn default_normalized(&self) -> f32 {
        self.normalize(self.default())
    }

    /// Get the name of the parameter.
    fn name(&self) -> Option<&str> {
        None
    }

    /// Get the short name of the parameter.
    fn short(&self) -> Option<&str> {
        None
    }

    /// Get the unit of the parameter.
    fn unit(&self) -> Unit {
        Unit::Unknown
    }

    /// Get the number of steps of the parameter.
    fn steps(&self) -> Option<i32> {
        None
    }

    /// Get the flags of the parameter.
    fn flags(&self) -> ParamFlags {
        ParamFlags::empty()
    }

    /// Convert a plain value to a string.
    fn to_string(&self, plain: f32) -> String {
        format!("{:.2}", plain)
    }

    /// Convert a string to a plain value.
    #[allow(clippy::wrong_self_convention)]
    fn from_string(&self, string: &str) -> f32 {
        string.parse().unwrap_or_default()
    }
}

/// A collection of parameters.
pub trait Params {
    /// Compute the number of parameters.
    fn count(&self) -> usize;

    /// Get information about a parameter.
    fn info(&self, index: usize) -> Option<ParamInfo>;

    /// Get a mutable reference to a parameter.
    fn param(&mut self, index: usize) -> Option<&mut dyn Param>;

    /// Get the identifier of a parameter.
    ///
    /// This is a unique string that identifies the parameter.
    fn identifier(&self, index: usize) -> Option<String>;
}

impl Params for () {
    fn count(&self) -> usize {
        0
    }

    fn info(&self, _index: usize) -> Option<ParamInfo> {
        None
    }

    fn param(&mut self, _index: usize) -> Option<&mut dyn Param> {
        None
    }

    fn identifier(&self, _index: usize) -> Option<String> {
        None
    }
}

/// Information about a parameter.
#[derive(Clone, Debug)]
pub struct ParamInfo {
    /// The name of the parameter.
    pub name: String,

    /// The short name of the parameter.
    pub short: String,

    /// The unit of the parameter.
    pub unit: Unit,

    /// The step count of the parameter.
    pub step_count: i32,

    /// The default value of the parameter in normalized form.
    pub default_normalized: f32,

    /// The flags of the parameter.
    pub flags: ParamFlags,
}

/// The unit of a parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Linear unit.
    Linear,

    /// Decibels unit.
    Decibels,

    /// Frequency unit.
    Frequency,

    /// Time unit.
    Time,

    /// Percent unit.
    Percent,

    /// Semitones unit.
    Semitones,

    /// Cents unit.
    Cents,

    /// Phase unit.
    Phase,

    /// Sample unit.
    Sample,

    /// Binary unit.
    Binary,

    /// Count unit.
    Count,

    /// Meters unit.
    Meters,

    /// Radians unit.
    Radians,

    /// Hertz unit.
    Hertz,

    /// Custom unit.
    Custom(String),

    /// Unknown unit.
    Unknown,
}

impl Unit {
    /// Get the VST identifier of the unit.
    pub fn id(&self) -> i32 {
        match self {
            Unit::Linear => 0,
            Unit::Decibels => 1,
            Unit::Frequency => 2,
            Unit::Time => 3,
            Unit::Percent => 4,
            Unit::Semitones => 5,
            Unit::Cents => 6,
            Unit::Phase => 7,
            Unit::Sample => 8,
            Unit::Binary => 9,
            Unit::Count => 10,
            Unit::Meters => 11,
            Unit::Radians => 12,
            Unit::Hertz => 13,
            Unit::Custom(_) => 14,
            Unit::Unknown => 15,
        }
    }

    /// Get the label of the unit.
    pub fn label(&self) -> &str {
        match self {
            Unit::Linear => "",
            Unit::Decibels => "dB",
            Unit::Frequency => "Frequency",
            Unit::Time => "ms",
            Unit::Percent => "%",
            Unit::Semitones => "Semitones",
            Unit::Cents => "Cents",
            Unit::Phase => "Degrees",
            Unit::Sample => "Samples",
            Unit::Binary => "On/Off",
            Unit::Count => "Count",
            Unit::Meters => "Meters",
            Unit::Radians => "Radians",
            Unit::Hertz => "Hz",
            Unit::Custom(name) => name,
            Unit::Unknown => "Unknown",
        }
    }
}

bitflags::bitflags! {
    /// Flags for a parameter.
    #[derive(Clone, Copy, Debug)]
    pub struct ParamFlags: u32 {
        /// The parameter can be automated.
        const AUTOMATE = 1 << 0;

        /// The parameter is read-only.
        const READ_ONLY = 1 << 1;

        /// The parameter wraps around.
        const WRAP = 1 << 2;

        /// The parameter is a list.
        const LIST = 1 << 3;

        /// The parameter is a program change.
        const PROGRAM_CHANGE = 1 << 4;

        /// The parameter is a bypass parameter.
        const BYPASS = 1 << 5;

        /// The parameter is hidden.
        const HIDDEN = 1 << 6;

        /// The parameter changes the unit.
        const UNIT_CHANGE = 1 << 7;

        /// The parameter is read-only and can be automated.
        const READ_ONLY_AUTOMATE = 1 << 8;

        /// The parameter is discrete.
        const DISCRETE = 1 << 9;

        /// The parameter has a display index.
        const HAS_DISPLAY_INDEX = 1 << 10;
    }
}
