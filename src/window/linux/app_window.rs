//! Linux (and other non-macOS, non-Windows) window shell.
//!
//! This path draws its own client-side decorations, rounded frame, drop shadow
//! and resize hit zones, while still tolerating server-side decorations from the
//! window manager.

use gpui_kit::component::Icon;
use gpui_kit::component::InteractiveElementExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::window_controls::WindowControl;
use super::window_decoration::{self};
use crate::icons::ICON_FEATHER;
use crate::theme::colors::COLOR_BRAND_FOREGROUND;
use crate::theme::colors::COLOR_FOREGROUND;
use crate::theme::colors::COLOR_WINDOW_BACKGROUND;
use crate::theme::colors::COLOR_WINDOW_INACTIVE_BACKGROUND;

const TITLE_BAR_HEIGHT: f32 = 42.;
const BRAND_LOGO_SIZE: f32 = 24.;
const BRAND_LOGO_INSET: f32 = 26. - BRAND_LOGO_SIZE / 2.;

const WINDOW_RADIUS: f32 = 12.;

struct TitleBarState {
  should_move: bool,
}

pub struct AppRoot<V> {
  pub root_focus: FocusHandle,
  pub root: Entity<V>,
}

impl<V: Render + 'static> Render for AppRoot<V> {
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let chrome = window_decoration::window_chrome(window);
    let background = rgb(if window.is_window_active() {
      COLOR_WINDOW_BACKGROUND
    } else {
      COLOR_WINDOW_INACTIVE_BACKGROUND
    });

    let body = div()
      .flex()
      .flex_col()
      .size_full()
      .track_focus(&self.root_focus)
      .bg(background)
      .text_color(rgb(COLOR_FOREGROUND))
      .border_color(background)
      .when(chrome.top_left, |this| this.rounded_tl(px(WINDOW_RADIUS)))
      .when(chrome.top_right, |this| this.rounded_tr(px(WINDOW_RADIUS)))
      .when(chrome.bottom_left, |this| {
        this.rounded_bl(px(WINDOW_RADIUS))
      })
      .when(chrome.bottom_right, |this| {
        this.rounded_br(px(WINDOW_RADIUS))
      })
      .when(chrome.border_top, |this| this.border_t(px(1.)))
      .when(chrome.border_right, |this| this.border_r(px(1.)))
      .when(chrome.border_bottom, |this| this.border_b(px(1.)))
      .when(chrome.border_left, |this| this.border_l(px(1.)))
      .child(deferred(self.title_bar(window, cx)))
      .child(self.root.clone());

    window_decoration::window_frame()
      .radius(px(WINDOW_RADIUS))
      .child(body)
  }
}

impl<V> AppRoot<V> {
  fn title_bar(
    &self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let is_client_decorated = matches!(window.window_decorations(), Decorations::Client { .. });
    let state = window.use_state(cx, |_, _| TitleBarState { should_move: false });

    div()
      .id("title-bar")
      // .bg(rgb(0x0000000))
      .absolute()
      .top_0()
      .left_0()
      .right_0()
      .h(px(TITLE_BAR_HEIGHT))
      .flex()
      .flex_row()
      .items_center()
      .on_double_click(|_, window, _| window.zoom_window())
      .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| {
        state.should_move = false;
      }))
      .on_mouse_down(
        MouseButton::Left,
        window.listener_for(&state, |state, _, _, _| {
          state.should_move = true;
        }),
      )
      .on_mouse_up(
        MouseButton::Left,
        window.listener_for(&state, |state, _, _, _| {
          state.should_move = false;
        }),
      )
      .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
        if state.should_move {
          state.should_move = false;
          window.start_window_move();
        }
      }))
      .child(
        div()
          .id("title-bar-drag")
          .flex()
          .flex_1()
          .items_center()
          .h_full()
          .when(is_client_decorated, |this| {
            this.on_mouse_down(MouseButton::Right, |event, window, _| {
              window.show_window_menu(event.position);
            })
          })
          .child(
            div()
              .pl(px(BRAND_LOGO_INSET))
              .pt(px(1.5))
              .flex()
              .items_center()
              .child(
                Icon::empty()
                  .data(ICON_FEATHER)
                  .h(px(BRAND_LOGO_SIZE))
                  .w(px(BRAND_LOGO_SIZE))
                  .text_color(rgb(COLOR_BRAND_FOREGROUND)),
              ),
          ),
      )
      .when(is_client_decorated, |this| {
        this
          .child(WindowControl::Minimize)
          .child(WindowControl::Maximize)
          .child(WindowControl::Close)
      })
  }
}
