//! macOS window shell.
//!
//! macOS keeps its native window decorations (including the traffic lights), so
//! there is no custom frame, chrome or resize handling here.

use gpui_kit::component::TitleBar;
use gpui_kit::*;

use crate::app::Update;
use crate::theme::colors::COLOR_BUTTON_BACKGROUND;
use crate::theme::colors::COLOR_BUTTON_FOREGROUND;
use crate::theme::colors::COLOR_BUTTON_HOVER_BACKGROUND;
use crate::theme::colors::COLOR_FOREGROUND;
use crate::theme::colors::COLOR_WINDOW_BACKGROUND;

const TITLE_BAR_HEIGHT: f32 = 42.;
const TRAFFIC_LIGHT_RESERVED_WIDTH: f32 = 10.;

const UPDATE_BUTTON_HEIGHT: f32 = 24.;
const UPDATE_BUTTON_GAP: f32 = 12.;

/// The window background is a tint over the blurred desktop rather than an
/// opaque fill; the alpha is the only lever for how much shows through.
const WINDOW_BACKGROUND_ALPHA: f32 = 0.8;

fn window_background() -> Rgba {
  let mut color = rgb(COLOR_WINDOW_BACKGROUND);
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
      .text_color(rgb(COLOR_FOREGROUND))
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
      .child(update_button())
  }
}

fn update_button() -> Stateful<Div> {
  div()
    .id("title-bar-update")
    .flex()
    .flex_shrink_0()
    .items_center()
    .h(px(UPDATE_BUTTON_HEIGHT))
    .px(px(10.))
    .mr(px(UPDATE_BUTTON_GAP))
    .rounded(px(4.))
    .cursor_pointer()
    .text_xs()
    .bg(rgb(COLOR_BUTTON_BACKGROUND))
    .text_color(rgb(COLOR_BUTTON_FOREGROUND))
    .hover(|this| this.bg(rgb(COLOR_BUTTON_HOVER_BACKGROUND)))
    .on_mouse_down(MouseButton::Left, |_, window, cx| {
      window.prevent_default();
      cx.stop_propagation();
    })
    .on_click(|_, window, cx| {
      cx.stop_propagation();
      window.dispatch_action(Update.boxed_clone(), cx);
    })
    .child("Update Available")
}
