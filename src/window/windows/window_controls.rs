//! Window control buttons for the Windows shell.
//!
//! These draw the caption glyphs Windows itself uses, from the Segoe MDL2 Assets
//! font that ships with Windows 10 and 11, so the buttons match native windows
//! in weight and size.
//!
//! The commands stay with the platform: each button declares a
//! [`WindowControlArea`], so Windows runs the command from its own non-client
//! hit testing, which is what makes restore possible — [`Window::zoom_window`]
//! only ever maximizes on this platform — and keeps snap layouts and the
//! caption's system menu intact.
//!
//! The highlight, however, is ours to paint. Windows lights a caption button up
//! inside the window's non-client area, and gpui creates this window with a
//! thick frame but no caption, so there is nothing for it to light: the buttons
//! draw the standard square highlight themselves — the theme's hover and active
//! pair, and the caption's red for close.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use crate::theme::colors::COLOR_RED_700;

const WINDOW_CONTROL_WIDTH: f32 = 48.;

const GLYPH_FONT_FAMILY: &str = "Segoe MDL2 Assets";
const GLYPH_SIZE: f32 = 10.;

const GLYPH_MINIMIZE: &str = "\u{E921}";
const GLYPH_MAXIMIZE: &str = "\u{E922}";
const GLYPH_RESTORE: &str = "\u{E923}";
const GLYPH_CLOSE: &str = "\u{E8BB}";

#[derive(Clone, Copy, IntoElement)]
pub enum WindowControl {
  Minimize,
  Maximize,
  Close,
}

/// The caption buttons in layout order, which is also the order they are
/// painted in.
pub const WINDOW_CONTROLS: [WindowControl; 3] = [
  WindowControl::Minimize,
  WindowControl::Maximize,
  WindowControl::Close,
];

/// The width the buttons occupy. The title bar reserves it so the drag region
/// never reaches under them.
pub const WINDOW_CONTROLS_WIDTH: f32 = WINDOW_CONTROL_WIDTH * WINDOW_CONTROLS.len() as f32;

impl WindowControl {
  fn id(self) -> &'static str {
    match self {
      Self::Minimize => "window-control-minimize",
      Self::Maximize => "window-control-maximize",
      Self::Close => "window-control-close",
    }
  }

  fn glyph(
    self,
    window: &Window,
  ) -> &'static str {
    match self {
      Self::Minimize => GLYPH_MINIMIZE,
      Self::Maximize if window.is_maximized() => GLYPH_RESTORE,
      Self::Maximize => GLYPH_MAXIMIZE,
      Self::Close => GLYPH_CLOSE,
    }
  }

  fn area(self) -> WindowControlArea {
    match self {
      Self::Minimize => WindowControlArea::Min,
      Self::Maximize => WindowControlArea::Max,
      Self::Close => WindowControlArea::Close,
    }
  }

  fn is_close(self) -> bool {
    matches!(self, Self::Close)
  }
}

impl RenderOnce for WindowControl {
  fn render(
    self,
    window: &mut Window,
    cx: &mut App,
  ) -> impl IntoElement {
    let theme = cx.theme();

    let foreground = theme.foreground;
    // Close keeps the icon white over the caption's red, as Windows draws it,
    // rather than the theme's danger pair, which reads as a muted salmon here.
    let hover_foreground = if self.is_close() {
      theme.foreground
    } else {
      theme.secondary_foreground
    };
    let hover_background = if self.is_close() {
      Hsla::from(rgb(COLOR_RED_700))
    } else {
      theme.secondary_hover
    };
    let active_background = if self.is_close() {
      theme.danger_active
    } else {
      theme.secondary_active
    };

    div()
      .id(self.id())
      .flex()
      .flex_shrink_0()
      .justify_center()
      .items_center()
      .w(px(WINDOW_CONTROL_WIDTH))
      .h_full()
      .window_control_area(self.area())
      .font_family(GLYPH_FONT_FAMILY)
      .text_size(px(GLYPH_SIZE))
      .line_height(px(GLYPH_SIZE))
      .text_color(foreground)
      .hover(|style| style.bg(hover_background).text_color(hover_foreground))
      .active(|style| style.bg(active_background).text_color(hover_foreground))
      .child(self.glyph(window))
  }
}
