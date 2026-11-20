use core::fmt;
use std::ops::{Add, Neg, Sub};
use std::str::FromStr;

use thiserror::Error;
use windows::Win32::Foundation::POINTL;

/// Contains the position of a display
#[derive(Default, Copy, Clone, PartialEq)]
pub struct Position(pub POINTL);

impl Eq for Position {}

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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Frequency(pub u32);

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

/// Display orientation (rotation)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    Landscape = 1,
    Portrait = 2,
    LandscapeFlipped = 3,
    PortraitFlipped = 4,
}

impl Orientation {
    /// Convert from DISPLAYCONFIG_ROTATION value
    pub fn from_rotation(rotation: u32) -> Self {
        match rotation {
            1 => Orientation::Landscape,
            2 => Orientation::Portrait,
            3 => Orientation::LandscapeFlipped,
            4 => Orientation::PortraitFlipped,
            _ => Orientation::Landscape, // Default fallback
        }
    }

    /// Convert to DISPLAYCONFIG_ROTATION value
    pub fn to_rotation(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Orientation::Landscape => write!(f, "landscape"),
            Orientation::Portrait => write!(f, "portrait"),
            Orientation::LandscapeFlipped => write!(f, "landscape_flipped"),
            Orientation::PortraitFlipped => write!(f, "portrait_flipped"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseOrientationError {
    #[error("Unknown orientation: {0}")]
    UnknownOrientation(String),
}

impl FromStr for Orientation {
    type Err = ParseOrientationError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "landscape" | "0" | "1" => Ok(Orientation::Landscape),
            "portrait" | "90" | "2" => Ok(Orientation::Portrait),
            "landscape_flipped" | "180" | "3" => Ok(Orientation::LandscapeFlipped),
            "portrait_flipped" | "270" | "4" => Ok(Orientation::PortraitFlipped),
            _ => Err(ParseOrientationError::UnknownOrientation(s.to_string())),
        }
    }
}

/// Display scaling mode
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Scaling {
    #[default]
    Identity = 0,
    Centered = 1,
    Stretched = 2,
    AspectRatioCenteredMax = 3,
    Custom = 4,
    Preferred = 128,
}

impl Scaling {
    pub fn from_value(value: i32) -> Self {
        match value {
            0 => Scaling::Identity,
            1 => Scaling::Centered,
            2 => Scaling::Stretched,
            3 => Scaling::AspectRatioCenteredMax,
            4 => Scaling::Custom,
            128 => Scaling::Preferred,
            _ => Scaling::Identity,
        }
    }

    pub fn to_value(&self) -> i32 {
        *self as i32
    }
}

impl fmt::Display for Scaling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scaling::Identity => write!(f, "identity"),
            Scaling::Centered => write!(f, "centered"),
            Scaling::Stretched => write!(f, "stretched"),
            Scaling::AspectRatioCenteredMax => write!(f, "aspect_ratio_centered_max"),
            Scaling::Custom => write!(f, "custom"),
            Scaling::Preferred => write!(f, "preferred"),
        }
    }
}

/// Bit depth / pixel format
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BitDepth {
    #[default]
    Bpp8 = 8,
    Bpp16 = 16,
    Bpp32 = 32,
}

impl BitDepth {
    pub fn from_pixel_format(format: i32) -> Self {
        // DISPLAYCONFIG_PIXELFORMAT values
        match format {
            1 => BitDepth::Bpp8,  // DISPLAYCONFIG_PIXELFORMAT_8BPP
            2 => BitDepth::Bpp16, // DISPLAYCONFIG_PIXELFORMAT_16BPP
            3 => BitDepth::Bpp32, // DISPLAYCONFIG_PIXELFORMAT_32BPP
            _ => BitDepth::Bpp32, // Default
        }
    }

    pub fn to_pixel_format(&self) -> i32 {
        // DISPLAYCONFIG_PIXELFORMAT values
        match self {
            BitDepth::Bpp8 => 1,  // DISPLAYCONFIG_PIXELFORMAT_8BPP
            BitDepth::Bpp16 => 2, // DISPLAYCONFIG_PIXELFORMAT_16BPP
            BitDepth::Bpp32 => 3, // DISPLAYCONFIG_PIXELFORMAT_32BPP
        }
    }
}

impl fmt::Display for BitDepth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} bpp", *self as u32)
    }
}

/// Scanline ordering (progressive vs interlaced)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ScanlineOrdering {
    Unspecified = 0,
    #[default]
    Progressive = 1,
    InterlacedUpperFieldFirst = 2,
    InterlacedLowerFieldFirst = 3,
}

impl ScanlineOrdering {
    pub fn from_value(value: i32) -> Self {
        match value {
            0 => ScanlineOrdering::Unspecified,
            1 => ScanlineOrdering::Progressive,
            2 => ScanlineOrdering::InterlacedUpperFieldFirst,
            3 => ScanlineOrdering::InterlacedLowerFieldFirst,
            _ => ScanlineOrdering::Unspecified,
        }
    }

    pub fn to_value(&self) -> i32 {
        *self as i32
    }
}

impl fmt::Display for ScanlineOrdering {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScanlineOrdering::Unspecified => write!(f, "unspecified"),
            ScanlineOrdering::Progressive => write!(f, "progressive"),
            ScanlineOrdering::InterlacedUpperFieldFirst => write!(f, "interlaced_upper_first"),
            ScanlineOrdering::InterlacedLowerFieldFirst => write!(f, "interlaced_lower_first"),
        }
    }
}

/// Display connector type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ConnectorType {
    HD15 = 0, // VGA
    SVIDEO = 1,
    CompositeVideo = 2,
    ComponentVideo = 3,
    DVI = 4,
    HDMI = 5,
    LVDS = 6,
    DJpn = 8,
    SDI = 9,
    DisplayPortExternal = 10,
    DisplayPortEmbedded = 11,
    UDIExternal = 12,
    UDIEmbedded = 13,
    SDTVDONGLE = 14,
    Miracast = 15,
    IndirectWired = 16,
    IndirectVirtual = 17,
    Internal = -2147483648, // 0x80000000
    Other = -1,
}

impl ConnectorType {
    pub fn from_value(value: i32) -> Self {
        match value {
            0 => ConnectorType::HD15,
            1 => ConnectorType::SVIDEO,
            2 => ConnectorType::CompositeVideo,
            3 => ConnectorType::ComponentVideo,
            4 => ConnectorType::DVI,
            5 => ConnectorType::HDMI,
            6 => ConnectorType::LVDS,
            8 => ConnectorType::DJpn,
            9 => ConnectorType::SDI,
            10 => ConnectorType::DisplayPortExternal,
            11 => ConnectorType::DisplayPortEmbedded,
            12 => ConnectorType::UDIExternal,
            13 => ConnectorType::UDIEmbedded,
            14 => ConnectorType::SDTVDONGLE,
            15 => ConnectorType::Miracast,
            16 => ConnectorType::IndirectWired,
            17 => ConnectorType::IndirectVirtual,
            -2147483648 => ConnectorType::Internal,
            _ => ConnectorType::Other,
        }
    }
}

impl fmt::Display for ConnectorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectorType::HD15 => write!(f, "VGA (HD15)"),
            ConnectorType::SVIDEO => write!(f, "S-Video"),
            ConnectorType::CompositeVideo => write!(f, "Composite Video"),
            ConnectorType::ComponentVideo => write!(f, "Component Video"),
            ConnectorType::DVI => write!(f, "DVI"),
            ConnectorType::HDMI => write!(f, "HDMI"),
            ConnectorType::LVDS => write!(f, "LVDS"),
            ConnectorType::DJpn => write!(f, "D-Jpn"),
            ConnectorType::SDI => write!(f, "SDI"),
            ConnectorType::DisplayPortExternal => write!(f, "DisplayPort (External)"),
            ConnectorType::DisplayPortEmbedded => write!(f, "DisplayPort (Embedded)"),
            ConnectorType::UDIExternal => write!(f, "UDI (External)"),
            ConnectorType::UDIEmbedded => write!(f, "UDI (Embedded)"),
            ConnectorType::SDTVDONGLE => write!(f, "SDTV Dongle"),
            ConnectorType::Miracast => write!(f, "Miracast"),
            ConnectorType::IndirectWired => write!(f, "Indirect Wired"),
            ConnectorType::IndirectVirtual => write!(f, "Indirect Virtual"),
            ConnectorType::Internal => write!(f, "Internal"),
            ConnectorType::Other => write!(f, "Other"),
        }
    }
}
