//! Windows window shape.
//!
//! DWM draws the window shadow and the outline from the window's own shape, not
//! from the alpha channel of the DirectComposition surface. This module asks DWM
//! for a square frame so it matches the square, opaque content painted by the
//! application.

use std::ffi::c_void;

use gpui_kit::Window;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawWindowHandle;

// https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_window_corner_preference
const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
const DWMWCP_DONOTROUND: i32 = 1;

#[link(name = "dwmapi")]
unsafe extern "system" {
  fn DwmSetWindowAttribute(
    hwnd: isize,
    attribute: u32,
    value: *const c_void,
    value_size: u32,
  ) -> i32;
}

/// Ask DWM to keep this window's frame square.
///
/// Windows 11 (build 22000 and later) rounds windows by default and honours the
/// corner preference. Older builds reject the attribute, but their windows are
/// already square.
///
/// `Window::window_handle` resolves to the inherent gpui method returning an
/// `AnyWindowHandle`, so the trait method is called explicitly to reach the
/// platform window handle.
pub fn prefer_square_corners(window: &Window) {
  let Ok(handle) = HasWindowHandle::window_handle(window) else {
    return;
  };

  let RawWindowHandle::Win32(handle) = handle.as_raw() else {
    return;
  };

  let preference = DWMWCP_DONOTROUND;

  unsafe {
    DwmSetWindowAttribute(
      handle.hwnd.get(),
      DWMWA_WINDOW_CORNER_PREFERENCE,
      &preference as *const _ as *const c_void,
      size_of::<i32>() as u32,
    );
  }
}
