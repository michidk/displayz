//! A library to interact with the Windows API for display settings.
//!
//! This library provides an abstraction around some `winuser.h` calls relevant for modifying display settings.

mod display;
mod properties;
mod types;

#[cfg(feature = "json")]
pub mod json;

pub use display::*;
pub use properties::*;
pub use types::*;
