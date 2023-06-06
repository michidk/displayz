//! Common helpers for `displayz`

/// Height and Width of a Display (`usize`)
pub type Resolution = (usize, usize);

/// X/Y positions of a display.
pub type Position = (usize, usize);

/// `Vec` type of the `Display` struct, exposed on a platform-dependent basis.
pub type Displays = Vec<crate::Display>;

/// `Vec` type of the `Resolution` type, generally exposing a collection of available resolutions.
pub type Resolutions = Vec<Resolution>;

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
    /// Returns a boolean result, if the `Display` is currently focused or not.
    /// This is not available on all platforms, in the event it's not, it will *always* return `false`.
    fn is_focused(&self) -> bool {
        false
    }
    /// Returns the `Position` custom type of the `Display`.
    fn get_position(&self) -> Position;
    /// Returns the `Resolution` custom type of the `Display`.
    fn get_resolution(&self) -> Resolution;
    /// Returns the current supported `Resolutions` of the `Display`.
    fn get_supported_resolutions(&self) -> Resolutions;
    /// Returns the EDID `&str` of the `Display.
    fn get_edid(&self) -> &str;
}
