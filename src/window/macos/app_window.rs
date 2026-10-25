//! macOS window shell.
//!
//! macOS keeps its native window decorations (including the traffic lights), so
//! there is no custom frame, chrome or resize handling here.

use gpui_kit::component::TitleBar;
use gpui_kit::*;

use crate::theme::colors::COLOR_GREY_50;
use crate::theme::colors::COLOR_GREY_900;

const TITLE_BAR_HEIGHT: f32 = 42.;
const TRAFFIC_LIGHT_RESERVED_WIDTH: f32 = 10.;

/// The window background is a tint over the blurred desktop rather than an
/// opaque fill; the alpha is the only lever for how much shows through.
const WINDOW_BACKGROUND_ALPHA: f32 = 0.8;

fn window_background() -> Rgba {
  let mut color = rgb(COLOR_GREY_900);
  color.a = WINDOW_BACKGROUND_ALPHA;
  color
}

pub struct AppRoot<V> {
  pub root_focus: FocusHandle,
  pub root: Entity<V>,
}

impl<V: Render + 'static> Render for AppRoot<V> {
  fn render(
    &mut self,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .flex()
      .flex_col()
      .size_full()
      .track_focus(&self.root_focus)
      .bg(window_background())
      .text_color(rgb(COLOR_GREY_50))
      .child(deferred(self.title_bar(cx)))
      .child(self.root.clone())
  }
}

impl<V> AppRoot<V> {
  fn title_bar(
    &self,
    _cx: &mut Context<Self>,
  ) -> impl IntoElement {
    TitleBar::new()
      .bg(rgba(0x00000000))
      // .bg(rgb(0x0000000))
      .border_color(rgba(0x00000000))
      .absolute()
      .top_0()
      .left_0()
      .right_0()
      .h(px(TITLE_BAR_HEIGHT))
      .child(
        div()
          .pl(px(TRAFFIC_LIGHT_RESERVED_WIDTH))
          .pt(px(1.5))
          .flex()
          .items_center(),
      )
  }
}
