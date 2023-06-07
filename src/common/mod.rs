//! Common helpers for `displayz`

/// Height and Width of a Display (`i32`)
pub struct Resolution(i32, i32);

/// X/Y positions of a display.
pub struct Position(i32, i32);

/// `Vec` type of the `Display` struct, exposed on a platform-dependent basis.
pub type Displays = Vec<crate::Display>;

/// `Vec` type of the `Resolution` type, generally exposing a collection of available resolutions.
/// NOTE: This type may be removed in future releases. You may rely on it *for now*. Further
/// developer feedback is required on future of this type's existence.
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
    /// Returns the current `Resolution` of the `Display`.
    fn get_resolution(&self) -> Resolution;
    /// Returns the current supported `Resolutions` of the `Display`.
    fn get_supported_resolutions(&self) -> Resolutions;
    /// Returns the EDID `&str` of the `Display.
    fn get_edid(&self) -> &str;
}
