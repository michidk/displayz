#[cfg(any(
    all(target_os = "linux", feature = "linux-x11"),
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"))]
mod x11;

#[cfg(any(
    all(target_os = "linux", feature = "linux-x11"),
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"))]
pub use x11::*;

#[cfg(all(target_os = "linux", feature = "linux-wayland"))]
mod wayland;

#[cfg(all(target_os = "linux", feature = "linux-wayland"))]
pub use wayland::*;
