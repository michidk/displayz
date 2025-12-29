use core::fmt;
use std::cell::{Cell, RefCell};
use std::ops::{Add, Neg, Sub};
use std::str::FromStr;

use thiserror::Error;
use windows::Win32::Foundation::{POINT, POINTL};
use windows::Win32::Graphics::Gdi::{
    DISPLAY_DEVICEW,
    DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW,
    DEVMODE_DISPLAY_ORIENTATION, DEVMODE_DISPLAY_FIXED_OUTPUT,
    DEVMODE_FIELD_FLAGS,
    CDS_TYPE, DISP_CHANGE,
    ChangeDisplaySettingsExW,
};
use windows::core::PCWSTR;

/// Error type for the display module
#[derive(Error, Debug)]
pub enum DisplayPropertiesError {
    #[error("Display {0} has no settings")]
    NoSettings(String),
    #[error("Error when calling the Windows API: {0}")]
    WinAPI(String),
    #[error("Apply failed, returned code: {0}")]
    ApplyFailed(i32),
    #[error("Invalid orientation: {0}")]
    InvalidOrientation(String),
    #[error("Invalid fixed output: {0}")]
    InvalidFixedOutput(String),
}

type Result<T = ()> = std::result::Result<T, DisplayPropertiesError>;

/// Contains the properties of a display
#[derive(Clone, Debug)]
pub struct DisplayProperties {
    pub name: String,

    pub string: String,
    pub key: String,

    pub active: bool,
    pub primary: Cell<bool>,

    pub settings: Option<RefCell<DisplaySettings>>,
}

impl fmt::Display for DisplayProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Display {{ key: {}, name: {}, string: {}, active: {}, primary: {} }}",
            self.key,
            self.name,
            self.string,
            self.active,
            self.primary.get()
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
    pub frequency: Frequency,
}

impl DisplayProperties {
    /// Create a display properties struct from a Windows display device
    pub fn from_windows(device: &DISPLAY_DEVICEW) -> Result<DisplayProperties> {
        let active = (device.StateFlags & 0x00000001) != 0; // DISPLAY_DEVICE_ATTACHED_TO_DESKTOP

        // Convert device name from wide string
        let name = unsafe {
            let len = device.DeviceName.iter().position(|&c| c == 0).unwrap_or(device.DeviceName.len());
            String::from_utf16_lossy(&device.DeviceName[..len])
        };

        let settings = if active {
            Some(RefCell::new(Self::fetch_settings(&name)?))
        } else {
            None
        };

        // Convert device string from wide string
        let string = unsafe {
            let len = device.DeviceString.iter().position(|&c| c == 0).unwrap_or(device.DeviceString.len());
            String::from_utf16_lossy(&device.DeviceString[..len])
        };

        // Convert device key from wide string
        let key = unsafe {
            let len = device.DeviceKey.iter().position(|&c| c == 0).unwrap_or(device.DeviceKey.len());
            String::from_utf16_lossy(&device.DeviceKey[..len])
        };

        Ok(DisplayProperties {
            name,
            string,
            key,
            active,
            primary: Cell::new((device.StateFlags & 0x00000004) != 0), // DISPLAY_DEVICE_PRIMARY_DEVICE
            settings,
        })
    }

    /// Fetch the settings of a display
    fn fetch_settings(name: &str) -> Result<DisplaySettings> {
        let mut devmode: DEVMODEW = unsafe { std::mem::zeroed() };
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

        // Convert name to wide string
        let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

        let result = unsafe {
            EnumDisplaySettingsW(
                PCWSTR(wide_name.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut devmode,
            )
        };

        if !result.as_bool() {
            return Err(DisplayPropertiesError::WinAPI(
                format!("EnumDisplaySettingsW failed for display {}", name)
            ));
        }

        Ok(DisplaySettings {
            position: Position(POINTL {
                x: unsafe { devmode.Anonymous1.Anonymous2.dmPosition.x },
                y: unsafe { devmode.Anonymous1.Anonymous2.dmPosition.y },
            }),
            resolution: Resolution::new(devmode.dmPelsWidth, devmode.dmPelsHeight),
            orientation: Orientation::from_windows(unsafe { devmode.Anonymous1.Anonymous2.dmDisplayOrientation.0 })?,
            fixed_output: FixedOutput::from_windows(unsafe { devmode.Anonymous1.Anonymous2.dmDisplayFixedOutput.0 })?,
            frequency: Frequency(devmode.dmDisplayFrequency),
        })
    }

    /// Apply the settings of the display
    pub fn apply(&self) -> Result {
        if self.settings.is_none() {
            return Err(DisplayPropertiesError::NoSettings(self.name.to_string()));
        }
        let settings = self.settings.as_ref().unwrap().borrow(); // safe, because we just checked it

        let mut flags = 0x00000001u32 | 0x00000004u32 | 0x00000008u32; // CDS_UPDATEREGISTRY | CDS_NORESET | CDS_GLOBAL

        if self.primary.get() {
            flags |= 0x00000002u32; // CDS_SET_PRIMARY
        }

        let devmode = DEVMODEW::from_display_settings(
            settings.position,
            settings.orientation,
            settings.fixed_output,
            settings.resolution,
            settings.frequency,
        );

        log::debug!(
            "Applying settings for {}: primary={}, flags={:?}",
            self.name,
            self.primary.get(),
            flags
        );

        // Convert name to wide string
        let wide_name: Vec<u16> = self.name.encode_utf16().chain(std::iter::once(0)).collect();

        let result = unsafe {
            ChangeDisplaySettingsExW(
                PCWSTR(wide_name.as_ptr()),
                Some(&devmode),
                None,
                CDS_TYPE(flags),
                None,
            )
        };

        if result.0 == 0 { // DISP_CHANGE_SUCCESSFUL
            log::debug!("Successfully applied settings for {}", self.name);
            Ok(())
        } else {
            log::error!("Failed to apply settings for {}: {:?}", self.name, result);
            Err(DisplayPropertiesError::ApplyFailed(result.0))
        }
    }
}

/// Provides methods to set properties of `DEVMODEW`
trait FromDisplaySettings {
    fn set_position(&mut self, position: Position);
    fn set_orientation(&mut self, orientation: Orientation);
    fn set_fixed_output(&mut self, fixed_output: FixedOutput);
    fn set_resolution(&mut self, resolution: Resolution);
    fn set_frequency(&mut self, frequency: Frequency);

    /// Converts display settings into a `DEVMODEW` struct
    fn from_display_settings(
        position: Position,
        orientation: Orientation,
        fixed_output: FixedOutput,
        resolution: Resolution,
        frequency: Frequency,
    ) -> DEVMODEW {
        let mut devmode: DEVMODEW = unsafe { std::mem::zeroed() };
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        devmode.set_position(position);
        devmode.set_orientation(orientation);
        devmode.set_fixed_output(fixed_output);
        devmode.set_resolution(resolution);
        devmode.set_frequency(frequency);
        devmode
    }
}

impl FromDisplaySettings for DEVMODEW {
    fn set_position(&mut self, position: Position) {
        unsafe {
            self.Anonymous1.Anonymous2.dmPosition = position.0;
        }
        self.dmFields |= DEVMODE_FIELD_FLAGS(0x00000020); // DM_POSITION
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        unsafe {
            self.Anonymous1.Anonymous2.dmDisplayOrientation = DEVMODE_DISPLAY_ORIENTATION(orientation.to_windows());
        }
        self.dmFields |= DEVMODE_FIELD_FLAGS(0x00000080); // DM_DISPLAYORIENTATION
    }

    fn set_fixed_output(&mut self, fixed_output: FixedOutput) {
        unsafe {
            self.Anonymous1.Anonymous2.dmDisplayFixedOutput = DEVMODE_DISPLAY_FIXED_OUTPUT(fixed_output.to_windows());
        }
        self.dmFields |= DEVMODE_FIELD_FLAGS(0x20000000); // DM_DISPLAYFIXEDOUTPUT
    }

    fn set_resolution(&mut self, resolution: Resolution) {
        self.dmPelsWidth = resolution.width;
        self.dmPelsHeight = resolution.height;
        self.dmFields |= DEVMODE_FIELD_FLAGS(0x00080000 | 0x00100000); // DM_PELSWIDTH | DM_PELSHEIGHT
    }

    fn set_frequency(&mut self, frequency: Frequency) {
        self.dmDisplayFrequency = frequency.0;
        self.dmFields |= DEVMODE_FIELD_FLAGS(0x00400000); // DM_DISPLAYFREQUENCY
    }
}

/// Contains the position of a display
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct Position(POINTL);

impl std::hash::Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.x.hash(state);
        self.0.y.hash(state);
    }
}

impl Position {
    /// Create a position
    pub fn new(x: i32, y: i32) -> Self {
        Self(POINTL { x, y })
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(POINTL {
            x: self.0.x + other.0.x,
            y: self.0.y + other.0.y,
        })
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(POINTL {
            x: self.0.x - other.0.x,
            y: self.0.y - other.0.y,
        })
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(POINTL {
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
    type Err = ParseResolutionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.split('x');
        let width = parts
            .next()
            .ok_or(ParseResolutionError::FirstPart)?
            .parse()?;
        let height = parts
            .next()
            .ok_or(ParseResolutionError::SecondPart)?
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
    /// Creates a new orientation from Windows constant
    fn from_windows(dmdo: u32) -> Result<Self> {
        match dmdo {
            0 => Ok(Orientation::Landscape),        // DMDO_DEFAULT
            1 => Ok(Orientation::PortraitFlipped),  // DMDO_90
            2 => Ok(Orientation::LandscapeFlipped), // DMDO_180
            3 => Ok(Orientation::Portrait),         // DMDO_270
            _ => Err(DisplayPropertiesError::InvalidOrientation(
                dmdo.to_string(),
            )),
        }
    }

    /// Creates the Windows orientation constant
    fn to_windows(self) -> u32 {
        match self {
            Orientation::Landscape => 0,        // DMDO_DEFAULT
            Orientation::PortraitFlipped => 1,  // DMDO_90
            Orientation::LandscapeFlipped => 2, // DMDO_180
            Orientation::Portrait => 3,         // DMDO_270
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
    /// Creates a new fixed output struct from Windows constant
    fn from_windows(dmdfo: u32) -> Result<Self> {
        match dmdfo {
            0 => Ok(FixedOutput::Default), // DMDFO_DEFAULT
            1 => Ok(FixedOutput::Stretch), // DMDFO_STRETCH
            2 => Ok(FixedOutput::Center),  // DMDFO_CENTER
            _ => Err(DisplayPropertiesError::InvalidFixedOutput(
                dmdfo.to_string(),
            )),
        }
    }

    /// Creates a Windows constant
    fn to_windows(self) -> u32 {
        match self {
            FixedOutput::Default => 0, // DMDFO_DEFAULT
            FixedOutput::Stretch => 1, // DMDFO_STRETCH
            FixedOutput::Center => 2,  // DMDFO_CENTER
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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Frequency(u32);

impl Frequency {
    pub fn new(v: u32) -> Self {
        Self(v)
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum ParseFrequencyError {
    #[error("Error parsing integer")]
    IntError(#[from] std::num::ParseIntError),
}

impl FromStr for Frequency {
    type Err = ParseFrequencyError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Frequency(s.parse::<u32>()?))
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
