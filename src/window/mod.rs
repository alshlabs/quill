#![allow(dead_code)]

//! Platform dispatch for the application window shell.
//!
//! Every supported platform owns a fully self-contained implementation.
//! This is required due to differences in the way platforms do things like
//! task tray and window decorations.

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod inner;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod inner;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "linux/mod.rs"]
mod inner;

pub use inner::bootstrap;

pub const WINDOW_MIN_WIDTH: f32 = 600.;

// #[path = "windows/mod.rs"]
// mod inner1;

// #[path = "linux/mod.rs"]
// mod inner2;

// #[path = "macos/mod.rs"]
// mod inner3;
