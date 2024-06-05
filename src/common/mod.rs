//! Common helpers for `displayz`

use std::fmt::{self, Debug, Display, Formatter};

/// Height and Width of a Display (`i32`)
pub struct Resolution(i32, i32);

/// X/Y positions of a display.
pub struct Position(i32, i32);

/// `Vec` type of the `Display` type, exposed on a platform-dependent basis.
pub type Displays = Vec<crate::Display>;

/// `Vec` type of the `Resolution` type, generally exposing a collection of available resolutions.
pub struct Resolutions(Vec<Resolution>);

impl Display for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.0, self.1)
    }
}

impl Debug for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Resolution of Display is: {}x{}", self.0, self.1)
    }
}

impl Debug for Resolutions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter())
            .finish()
            .expect("Unable to format `Debug` output for `Resolutions` struct.");
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Position of Display is: {},{}", self.0, self.1)
    }
}

/// `DisplayOutput` defines the Trait specification of a platform's `Display`.
/// A `Display` may contain:
/// - An EDID.
/// - State, including if the `Display` is active, primary, or has a fault.
/// - Other helper methods, returning data stored in-memory state.
pub trait DisplayOutput {
    /// Returns a boolean result, if the `Display` is the 'primary display' or not.
    fn is_primary(&self) -> bool;
    /// Returns a boolean result, if the `Display` is currently active or not.
    fn is_active(&self) -> bool;
    /// Returns the `Position` custom type of the `Display`.
    fn get_position(&self) -> Position {
        Position::default()
    }
    /// Returns the current `Resolution` of the `Display`.
    fn get_resolution(&self) -> Resolution {
        Resolution::default()
    }
    /// Returns the current supported `Resolutions` of the `Display`.
    fn get_supported_resolutions(&self) -> Resolutions {
        Resolutions::default()
    }
    /// Returns the EDID `&str` of the `Display.
    fn get_edid(&self) -> Option<&str>;
}
