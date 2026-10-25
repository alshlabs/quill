//! Windows resize band.
//!
//! The frame is drawn one [`super::window_decoration::SHADOW_SIZE`] inside the
//! client area, because that margin is where its shadow lives, while the resize
//! borders Windows provides sit on the window rectangle — and gpui only replaces
//! the top border, having removed that one to host the title bar. Without help
//! the margin is dead space the width of the shadow, so the drawn frame edge
//! cannot be grabbed. This module subclasses the window procedure and answers
//! `WM_NCHITTEST` for a band centred on the drawn edge, which lets Windows
//! itself resize from the edge the user can see, with its own resize cursor,
//! snapping and maximise-on-drag intact.
//!
//! The band is deliberately independent of the shadow margin: it reaches
//! [`super::window_decoration::RESIZE_HIT_SIZE`] to either side of the drawn
//! edge and no further, so the margin can be widened for a larger shadow
//! without pushing the grab area out over the desktop.

use gpui_kit::Window;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawWindowHandle;

const GWL_STYLE: i32 = -16;
const WS_THICKFRAME: u32 = 0x0004_0000;
const WM_NCHITTEST: u32 = 0x0084;
const HTLEFT: isize = 10;
const HTRIGHT: isize = 11;
const HTTOP: isize = 12;
const HTTOPLEFT: isize = 13;
const HTTOPRIGHT: isize = 14;
const HTBOTTOM: isize = 15;
const HTBOTTOMLEFT: isize = 16;
const HTBOTTOMRIGHT: isize = 17;
const SUBCLASS_ID: usize = 1;
const REFERENCE_DPI: f32 = 96.;
/// The subclass has one word of reference data, so the two logical-pixel widths
/// it carries are packed into it half a word each.
const WIDTH_MASK: usize = 0xffff;
const MARGIN_SHIFT: u32 = 16;

#[repr(C)]
#[derive(Default)]
struct Point {
  x: i32,
  y: i32,
}

#[repr(C)]
#[derive(Default)]
struct Rect {
  left: i32,
  top: i32,
  right: i32,
  bottom: i32,
}

#[link(name = "comctl32")]
unsafe extern "system" {
  fn SetWindowSubclass(
    hwnd: isize,
    procedure: SubclassProcedure,
    id: usize,
    data: usize,
  ) -> i32;
  fn DefSubclassProc(
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
  ) -> isize;
}

#[link(name = "user32")]
unsafe extern "system" {
  fn GetClientRect(
    hwnd: isize,
    rect: *mut Rect,
  ) -> i32;
  fn GetDpiForWindow(hwnd: isize) -> u32;
  fn GetWindowLongPtrW(
    hwnd: isize,
    index: i32,
  ) -> isize;
  fn IsZoomed(hwnd: isize) -> i32;
  fn ScreenToClient(
    hwnd: isize,
    point: *mut Point,
  ) -> i32;
}

type SubclassProcedure = unsafe extern "system" fn(
  hwnd: isize,
  message: u32,
  wparam: usize,
  lparam: isize,
  id: usize,
  data: usize,
) -> isize;

/// Accept the resize band around the drawn frame edge.
///
/// `margin` is the logical width of the transparent shadow margin
/// [`super::window_decoration`] paints around the frame, and `band` is how far
/// to either side of the drawn edge the resize band reaches. Both are rounded
/// to whole logical pixels. Installed once per window; a failure only leaves
/// resizing on the platform's own border.
pub fn extend_resize_border(
  window: &Window,
  margin: f32,
  band: f32,
) {
  let Ok(handle) = HasWindowHandle::window_handle(window) else {
    return;
  };

  let RawWindowHandle::Win32(handle) = handle.as_raw() else {
    return;
  };

  let margin = (margin.round() as usize) & WIDTH_MASK;
  let band = (band.round() as usize) & WIDTH_MASK;

  unsafe {
    SetWindowSubclass(
      handle.hwnd.get(),
      subclass_procedure,
      SUBCLASS_ID,
      margin | (band << MARGIN_SHIFT),
    );
  }
}

unsafe extern "system" fn subclass_procedure(
  hwnd: isize,
  message: u32,
  wparam: usize,
  lparam: isize,
  _id: usize,
  widths: usize,
) -> isize {
  if message == WM_NCHITTEST {
    if let Some(edge) = unsafe { frame_edge(hwnd, lparam, widths) } {
      return edge;
    }
  }

  unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

/// The resize edge under the cursor when it falls in the band around the drawn
/// frame edge, in which case that band is treated as the frame border.
///
/// Fullscreen windows lose their sizing frame and Windows keeps the size of a
/// maximized window to itself, so both are left to the platform.
unsafe fn frame_edge(
  hwnd: isize,
  lparam: isize,
  widths: usize,
) -> Option<isize> {
  let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32;
  if style & WS_THICKFRAME == 0 || unsafe { IsZoomed(hwnd) } != 0 {
    return None;
  }

  let mut rect = Rect::default();
  if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
    return None;
  }

  let mut point = Point {
    x: (lparam as i32) as i16 as i32,
    y: ((lparam >> 16) as i32) as i16 as i32,
  };
  if unsafe { ScreenToClient(hwnd, &mut point) } == 0 {
    return None;
  }

  let scale_factor = unsafe { GetDpiForWindow(hwnd) } as f32 / REFERENCE_DPI;
  let scale = |width: usize| ((width & WIDTH_MASK) as f32 * scale_factor).round() as i32;
  let margin = scale(widths);
  let band = scale(widths >> MARGIN_SHIFT);

  let (left, right) = (margin, rect.right - margin);
  let (top, bottom) = (margin, rect.bottom - margin);

  let on_left = point.x >= left - band && point.x < left + band;
  let on_right = point.x >= right - band && point.x < right + band;
  let on_top = point.y >= top - band && point.y < top + band;
  let on_bottom = point.y >= bottom - band && point.y < bottom + band;

  match (on_top, on_bottom, on_left, on_right) {
    (true, _, true, _) => Some(HTTOPLEFT),
    (true, _, _, true) => Some(HTTOPRIGHT),
    (_, true, true, _) => Some(HTBOTTOMLEFT),
    (_, true, _, true) => Some(HTBOTTOMRIGHT),
    (true, _, _, _) => Some(HTTOP),
    (_, true, _, _) => Some(HTBOTTOM),
    (_, _, true, _) => Some(HTLEFT),
    (_, _, _, true) => Some(HTRIGHT),
    _ => None,
  }
}
