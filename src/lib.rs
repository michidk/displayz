//! A library to interact with the Windows API for display settings.
//!
//! This library provides an abstraction around some `winuser.h` calls relevant for modifying display settings.
#[cfg_attr(
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ),
    path = "platforms/unix/mod.rs"
)]
#[cfg_attr(target_os = "windows", path = "platforms/windows/mod.rs")]
#[cfg_attr(target_os = "macos", path = "platforms/macos/mod.rs")]
mod platform;
pub use crate::platform::*;

mod common;
pub use crate::common::*;

pub type Display = Box<dyn DisplayOutput>;
