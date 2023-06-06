#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"))]
mod x11;
pub use x11::*;

#[cfg(target_os = "linux")]
mod wayland;
pub use wayland::*;
