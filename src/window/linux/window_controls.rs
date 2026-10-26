//! Window control buttons for the Linux shell.
//!
//! These draw the app's own caption glyphs, because the icons shipped by the
//! component library are resolved through an asset bundle this application does
//! not register.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Icon;
use gpui_kit::component::Sizable as _;
use gpui_kit::*;

use crate::icons::ICON_WINDOW_CLOSE;
use crate::icons::ICON_WINDOW_MAXIMIZE;
use crate::icons::ICON_WINDOW_MINIMIZE;
use crate::theme::colors::COLOR_RED_700;

const WINDOW_CONTROL_WIDTH: f32 = 32.;

/// The glyphs are drawn on a 16-unit grid with strokes on half units, so they
/// only stay pixel-crisp when rendered at exactly this size.
const WINDOW_CONTROL_ICON_SIZE: f32 = 16.;

/// Diameter of the hover/active highlight, centred inside the button cell.
const WINDOW_CONTROL_HIGHLIGHT_SIZE: f32 = 26.;

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
      Self::Minimize => ICON_WINDOW_MINIMIZE,
      Self::Maximize => ICON_WINDOW_MAXIMIZE,
      Self::Close => ICON_WINDOW_CLOSE,
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
      .group(self.id())
      .text_color(foreground)
      .on_mouse_down(MouseButton::Left, |_, window, cx| {
        window.prevent_default();
        cx.stop_propagation();
      })
      .on_click(move |_, window, cx| {
        cx.stop_propagation();
        match self {
          Self::Minimize => window.minimize_window(),
          Self::Maximize => window.zoom_window(),
          Self::Close => window.remove_window(),
        }
      })
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
          .child(
            Icon::empty()
              .data(self.icon())
              .with_size(px(WINDOW_CONTROL_ICON_SIZE)),
          ),
      )
  }
}
