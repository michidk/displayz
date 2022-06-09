use core::fmt;
use std::cell::RefCell;
use std::ops::{Add, Neg, Sub};
use std::str::FromStr;

use thiserror::Error;
use winsafe::{co, prelude::NativeBitflag, GmidxEnum, DISPLAY_DEVICE, POINT};

/// Error type for the display module
#[derive(Error, Debug)]
pub enum DisplayPropertiesError {
    #[error("Display {0} has no settings")]
    NoSettings(String),
    #[error("Error when calling the Windows API")]
    WinAPI(#[from] co::ERROR),
    #[error("Apply failed, returned flags: {0}")]
    ApplyFailed(co::DISP_CHANGE),
    #[error("Invalid orientation: {0}")]
    InvalidOrientation(String),
    #[error("Invalid fixed output: {0}")]
    InvalidFixedOutput(String),
}

type Result<T = ()> = std::result::Result<T, DisplayPropertiesError>;

/// Contains the properties of a display
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayProperties {
    pub name: String,

    pub string: String,
    pub key: String,

    pub active: bool,
    pub primary: bool,

    pub settings: Option<RefCell<DisplaySettings>>,
}

impl fmt::Display for DisplayProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Display {{ key: {}, name: {}, string: {}, active: {}, primary: {} }}",
            self.key, self.name, self.string, self.active, self.primary
        )
    }
}

/// Contains the settings of a display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplaySettings {
    pub position: Position,
    pub resolution: Resolution,
    pub orientation: Orientation,
    pub fixed_output: FixedOutput,
}

impl DisplayProperties {
    /// Create a display properties struct from a winsafe display
    pub fn from_winsafe(device: &DISPLAY_DEVICE) -> Result<DisplayProperties> {
        let active = device.StateFlags.has(co::DISPLAY_DEVICE::ACTIVE);
        let settings = if active {
            Some(RefCell::new(Self::fetch_settings(&device.DeviceName())?))
        } else {
            None
        };

        Ok(DisplayProperties {
            name: device.DeviceName(),
            string: device.DeviceString(),
            key: device.DeviceKey(),
            active,
            primary: device.StateFlags.has(co::DISPLAY_DEVICE::PRIMARY_DEVICE),
            settings,
        })
    }

    /// Fetch the settings of a display
    fn fetch_settings(name: &str) -> Result<DisplaySettings> {
        let mut devmode = winsafe::DEVMODE::default();
        winsafe::EnumDisplaySettings(
            Some(name),
            GmidxEnum::Enum(winsafe::co::ENUM_SETTINGS::CURRENT),
            &mut devmode,
        )?;

        Ok(DisplaySettings {
            position: Position(devmode.dmPosition()),
            resolution: Resolution::new(devmode.dmPelsWidth, devmode.dmPelsHeight),
            orientation: Orientation::from_winsafe(devmode.dmDisplayOrientation())?,
            fixed_output: FixedOutput::from_winsafe(devmode.dmDisplayFixedOutput())?,
        })
    }

    /// Apply the settings of the display
    pub fn apply(&self) -> Result {
        if self.settings.is_none() {
            return Err(DisplayPropertiesError::NoSettings(self.name.to_string()));
        }
        let settings = self.settings.as_ref().unwrap().borrow(); // safe, because we just checked it

        let mut flags =
            winsafe::co::CDS::UPDATEREGISTRY | winsafe::co::CDS::NORESET | winsafe::co::CDS::GLOBAL;

        if self.primary {
            flags |= winsafe::co::CDS::SET_PRIMARY;
        }

        let mut devmode = winsafe::DEVMODE::from_display_settings(
            settings.position,
            settings.orientation,
            settings.fixed_output,
            settings.resolution,
        );

        let result = winsafe::ChangeDisplaySettingsEx(Some(&self.name), Some(&mut devmode), flags);
        // use into_ok_or_err as soon it is stable
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(DisplayPropertiesError::ApplyFailed(err)),
        }
    }
}

/// Provides methods to set properties of `winsafe::DEVMODE`
trait FromDisplaySettings {
    fn set_position(&mut self, position: Position);
    fn set_orientation(&mut self, orientation: Orientation);
    fn set_fixed_output(&mut self, fixed_output: FixedOutput);
    fn set_resolution(&mut self, resolution: Resolution);

    /// Converts display settings into a `winsafe::DEVMODE` struct
    fn from_display_settings(
        position: Position,
        orientation: Orientation,
        fixed_output: FixedOutput,
        resolution: Resolution,
    ) -> winsafe::DEVMODE {
        let mut devmode = winsafe::DEVMODE::default();
        devmode.set_position(position);
        devmode.set_orientation(orientation);
        devmode.set_fixed_output(fixed_output);
        devmode.set_resolution(resolution);
        devmode
    }
}

impl FromDisplaySettings for winsafe::DEVMODE {
    fn set_position(&mut self, position: Position) {
        self.set_dmPosition(position.0);
        self.dmFields |= winsafe::co::DM::POSITION;
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        self.set_dmDisplayOrientation(orientation.to_winsafe());
        self.dmFields |= winsafe::co::DM::DISPLAYORIENTATION;
    }

    fn set_fixed_output(&mut self, fixed_output: FixedOutput) {
        self.set_dmDisplayFixedOutput(fixed_output.to_winsafe());
        self.dmFields |= winsafe::co::DM::DISPLAYFIXEDOUTPUT;
    }

    fn set_resolution(&mut self, resolution: Resolution) {
        self.dmPelsWidth = resolution.width;
        self.dmPelsHeight = resolution.height;
        self.dmFields |= winsafe::co::DM::PELSWIDTH | winsafe::co::DM::PELSHEIGHT;
    }
}

/// Contains the position of a display
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position(POINT);

impl Position {
    /// Create a position
    pub fn new(x: i32, y: i32) -> Self {
        Self(POINT { x, y })
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(POINT {
            x: self.0.x + other.0.x,
            y: self.0.y + other.0.y,
        })
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(POINT {
            x: self.0.x - other.0.x,
            y: self.0.y - other.0.y,
        })
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(POINT {
            x: -self.0.x,
            y: -self.0.y,
        })
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0.x, self.0.y)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Point")
            .field(&self.0.x)
            .field(&self.0.y)
            .finish()
    }
}

/// Errors that occur while parsing a position from a string
#[derive(Error, Debug)]
pub enum ParsePositionError {
    #[error("Error parsing integer")]
    IntError(#[from] std::num::ParseIntError),
    #[error("First part missing")]
    FirstPart,
    #[error("Second part missing. Expected format: <x>,<y>")]
    SecondPart,
}

impl FromStr for Position {
    type Err = ParsePositionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.split(',');
        let x = parts.next().ok_or(ParsePositionError::FirstPart)?.parse()?;
        let y = parts
            .next()
            .ok_or(ParsePositionError::SecondPart)?
            .parse()?;
        Ok(Self::new(x, y))
    }
}

/// Contains the resolution of a display
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    /// Creates a new resolution
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Errors that occur while parsing a resolution from a string
#[derive(Error, Debug)]
pub enum ParseResolutionError {
    #[error("Error parsing integer")]
    IntError(#[from] std::num::ParseIntError),
    #[error("First integer missing")]
    FirstPart,
    #[error("Second integer missing. Expected format: <width>x<height>")]
    SecondPart,
}

impl FromStr for Resolution {
    type Err = ParsePositionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.split('x');
        let width = parts.next().ok_or(ParsePositionError::FirstPart)?.parse()?;
        let height = parts
            .next()
            .ok_or(ParsePositionError::SecondPart)?
            .parse()?;
        Ok(Self::new(width, height))
    }
}

/// Contains the orientation of a display
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Orientation {
    Landscape,        // default
    LandscapeFlipped, // upside-down
    Portrait,         // rotate right
    PortraitFlipped,  // rotate left
}

impl Orientation {
    /// Creates a new orientation from `winsafe::co::DMD0`
    fn from_winsafe(co_dmdo: co::DMDO) -> Result<Self> {
        match co_dmdo {
            co::DMDO::DEFAULT => Ok(Orientation::Landscape),
            co::DMDO::D90 => Ok(Orientation::PortraitFlipped),
            co::DMDO::D180 => Ok(Orientation::LandscapeFlipped),
            co::DMDO::D270 => Ok(Orientation::Portrait),
            _ => Err(DisplayPropertiesError::InvalidOrientation(
                co_dmdo.to_string(),
            )),
        }
    }

    /// Creates the winsafe orientation struct
    fn to_winsafe(self) -> co::DMDO {
        match self {
            Orientation::Landscape => co::DMDO::DEFAULT,
            Orientation::PortraitFlipped => co::DMDO::D90,
            Orientation::LandscapeFlipped => co::DMDO::D180,
            Orientation::Portrait => co::DMDO::D270,
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Landscape => write!(f, "Default"),
            Orientation::LandscapeFlipped => write!(f, "UpsideDown"),
            Orientation::Portrait => write!(f, "Right"),
            Orientation::PortraitFlipped => write!(f, "Left"),
        }
    }
}

/// Errors that occur while parsing an orientation from a string
#[derive(Error, Debug)]
pub enum ParseOrientationError {
    #[error("Invalid orientation. Allowed values: `Default`, `UpsideDown`, `Right`, `Left`")]
    InvalidOrientation,
}

impl FromStr for Orientation {
    type Err = ParseOrientationError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" | "landscape" => Ok(Orientation::Landscape),
            "upsidedown" | "landscapeflipped" => Ok(Orientation::LandscapeFlipped),
            "right" | "portrait" => Ok(Orientation::Portrait),
            "left" | "portraitflipped" => Ok(Orientation::PortraitFlipped),
            _ => Err(ParseOrientationError::InvalidOrientation),
        }
    }
}

/// Contains the fixed output of a display
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixedOutput {
    Default,
    Stretch,
    Center,
}

impl FixedOutput {
    /// Creates a new fixed output struct from `winsafe::co::DMDF0`
    fn from_winsafe(co_dmdfo: co::DMDFO) -> Result<Self> {
        match co_dmdfo {
            co::DMDFO::DEFAULT => Ok(FixedOutput::Default),
            co::DMDFO::STRETCH => Ok(FixedOutput::Stretch),
            co::DMDFO::CENTER => Ok(FixedOutput::Center),
            _ => Err(DisplayPropertiesError::InvalidFixedOutput(
                co_dmdfo.to_string(),
            )),
        }
    }

    /// Creates a winsafe struct
    fn to_winsafe(self) -> co::DMDFO {
        match self {
            FixedOutput::Default => co::DMDFO::DEFAULT,
            FixedOutput::Stretch => co::DMDFO::STRETCH,
            FixedOutput::Center => co::DMDFO::CENTER,
        }
    }
}

impl fmt::Display for FixedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedOutput::Default => write!(f, "Default"),
            FixedOutput::Stretch => write!(f, "Stretch"),
            FixedOutput::Center => write!(f, "Center"),
        }
    }
}

/// Errors that occur while parsing a fixed output from a string
#[derive(Error, Debug)]
pub enum ParseFixedOutputError {
    #[error("Invalid fxed output mode. Allowed values: `Default`, `Stretch`, `Center`")]
    InvalidFixedOutput,
}

impl FromStr for FixedOutput {
    type Err = ParseFixedOutputError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(FixedOutput::Default),
            "stretch" => Ok(FixedOutput::Stretch),
            "center" => Ok(FixedOutput::Center),
            _ => Err(ParseFixedOutputError::InvalidFixedOutput),
        }
    }
}
