//! A library to interact with the Windows API for display settings.
//!
//! This library provides a high-level abstraction around the Windows Display Configuration API
//! for querying and modifying display settings such as resolution, orientation, position, and scaling.

mod display;
mod properties;
mod types;

#[cfg(feature = "json")]
pub mod json;

pub use display::*;
pub use properties::*;
pub use types::*;
