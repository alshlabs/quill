//! Windows window shape.
//!
//! DWM draws the window outline and drop shadow from the window's rectangle and
//! its own corner preference, ignoring the alpha channel of the
//! DirectComposition surface. A rounded frame painted by the application
//! therefore sits inside a square shadow, so this module switches DWM's
//! non-client rendering off and leaves the frame entirely to
//! [`super::window_decoration`].

use std::ffi::c_void;

use gpui_kit::Window;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawWindowHandle;

// https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_ncrendering_policy
const DWMWA_NCRENDERING_POLICY: u32 = 2;
const DWMNCRP_DISABLED: i32 = 1;

#[link(name = "dwmapi")]
unsafe extern "system" {
  fn DwmSetWindowAttribute(
    hwnd: isize,
    attribute: u32,
    value: *const c_void,
    value_size: u32,
  ) -> i32;
}

/// Stop DWM from drawing this window's non-client area, which removes the system
/// outline and drop shadow that would otherwise be drawn around the window
/// rectangle instead of around the rounded frame painted by
/// [`super::window_decoration`].
///
/// Fails silently where the attribute is rejected, in which case DWM keeps
/// drawing its own square outline and shadow.
///
/// `Window::window_handle` resolves to the inherent gpui method returning an
/// `AnyWindowHandle`, so the trait method is called explicitly to reach the
/// platform window handle.
pub fn disable_non_client_rendering(window: &Window) {
  let Ok(handle) = HasWindowHandle::window_handle(window) else {
    return;
  };

  let RawWindowHandle::Win32(handle) = handle.as_raw() else {
    return;
  };

  let policy = DWMNCRP_DISABLED;

  unsafe {
    DwmSetWindowAttribute(
      handle.hwnd.get(),
      DWMWA_NCRENDERING_POLICY,
      &policy as *const _ as *const c_void,
      size_of::<i32>() as u32,
    );
  }
}
