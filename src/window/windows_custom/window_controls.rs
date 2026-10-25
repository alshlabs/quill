//! Window control buttons for the Windows shell.
//!
//! These draw the app's own bootstrap icons, because the icons shipped by the
//! component library are resolved through an asset bundle this application does
//! not register.
//!
//! The buttons do not run the command themselves: each one declares a
//! [`WindowControlArea`], and Windows performs the action from its own
//! non-client hit testing. That is what makes restore possible —
//! [`Window::zoom_window`] only ever maximizes on this platform — and it keeps
//! the platform's snap layouts, press feedback and system menu behaviour
//! intact. Hover and press styling still come from gpui, which forwards
//! non-client mouse movement and buttons to the application.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Icon;
use gpui_kit::component::Sizable as _;
use gpui_kit::*;

use crate::icons::ICON_DASH;
use crate::icons::ICON_SQUARE;
use crate::icons::ICON_X;
use crate::theme::colors::COLOR_RED_700;

const WINDOW_CONTROL_WIDTH: f32 = 48.;

/// The box each icon is drawn into, sized so that all three glyphs land on the
/// same 10px extent and the same stroke weight.
///
/// Bootstrap draws the glyphs at different scales inside their shared 16-unit
/// viewBox: the square spans the whole of it, while the dash covers half its
/// width and the cross just under half of both axes. One icon size for all
/// three therefore leaves the square oversized and the other two undersized.
const MINIMIZE_ICON_SIZE: f32 = 20.;
const MAXIMIZE_ICON_SIZE: f32 = 10.;
const CLOSE_ICON_SIZE: f32 = 21.;

/// Diameter of the hover/active highlight, centred inside the button cell.
const WINDOW_CONTROL_HIGHLIGHT_SIZE: f32 = 30.;

#[derive(Clone, Copy, IntoElement)]
pub enum WindowControl {
  Minimize,
  Maximize,
  Close,
}

impl WindowControl {
  fn id(self) -> &'static str {
    match self {
      Self::Minimize => "window-control-minimize",
      Self::Maximize => "window-control-maximize",
      Self::Close => "window-control-close",
    }
  }

  fn icon(self) -> &'static [u8] {
    match self {
      Self::Minimize => ICON_DASH,
      Self::Maximize => ICON_SQUARE,
      Self::Close => ICON_X,
    }
  }

  fn icon_size(self) -> Pixels {
    match self {
      Self::Minimize => px(MINIMIZE_ICON_SIZE),
      Self::Maximize => px(MAXIMIZE_ICON_SIZE),
      Self::Close => px(CLOSE_ICON_SIZE),
    }
  }

  /// The cell keeps [`Self::id`]; the highlight needs its own element state to
  /// carry the pressed style.
  fn highlight_id(self) -> &'static str {
    match self {
      Self::Minimize => "window-control-minimize-highlight",
      Self::Maximize => "window-control-maximize-highlight",
      Self::Close => "window-control-close-highlight",
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
    _window: &mut Window,
    cx: &mut App,
  ) -> impl IntoElement {
    let theme = cx.theme();

    let foreground = theme.foreground;
    // Close keeps the icon white over the platform's own close red, rather than
    // the theme's danger pair, which reads as a muted salmon on this shell.
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
      .group(self.id())
      .text_color(foreground)
      .child(
        div()
          .id(self.highlight_id())
          .flex()
          .justify_center()
          .items_center()
          .size(px(WINDOW_CONTROL_HIGHLIGHT_SIZE))
          .rounded_full()
          .group_hover(self.id(), |style| {
            style.bg(hover_background).text_color(hover_foreground)
          })
          .group_active(self.id(), |style| {
            style.bg(active_background).text_color(hover_foreground)
          })
          .child(Icon::empty().data(self.icon()).with_size(self.icon_size())),
      )
  }
}
