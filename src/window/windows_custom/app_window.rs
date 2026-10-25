//! Windows window shell.
//!
//! This platform is client-decorated: the rounded frame, drop shadow and 1px
//! border are drawn here around the content, and [`super::dwm`] stops DWM from
//! drawing a second, square outline and shadow behind them. [`super::hit_test`]
//! puts the resize band on the drawn frame edge, inside the shadow margin, so
//! the margin stays the shadow's to use.
//!
//! The title bar is drawn here too, because it carries the application's own
//! [`WindowControl`] buttons: the component library's title bar always renders
//! its own controls, whose icons this application cannot resolve, so the whole
//! bar is owned instead of only the strip that holds them.
//!
//! The strip is painted first, then the content, then the controls. That order
//! is also the hit-test order, since gpui hit tests the hitboxes in reverse: an
//! element the content places in the strip takes the click, the strip keeps the
//! drag everywhere else, and the controls stay above the content, which reaches
//! the top of the window and would otherwise paint them out. Deferring the
//! strip would put it above the content for both paint and hit testing, leaving
//! the content unable to cover it and blocking its clicks; deferred drawing
//! outranks the whole normal tree, so no priority reaches over it.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::window_controls::WindowControl;
use super::window_decoration::{self};
use crate::theme::colors::COLOR_GREY_900;
use crate::theme::colors::COLOR_GREY_800;
use crate::theme::colors::COLOR_GREY_50;

const TITLE_BAR_HEIGHT: f32 = 48.;
const TRAFFIC_LIGHT_RESERVED_WIDTH: f32 = 10.;
const BOUNDARY_THICKNESS: f32 = 1.;

const WINDOW_RADIUS: f32 = 12.;

/// Marks where the shell's strip ends, in the shell's own layer.
///
/// The strip is painted first so the content can own it, which means a border on
/// the strip itself is painted out by whatever the content draws there. This is
/// painted last instead, so the edge stays visible.
fn strip_boundary() -> impl IntoElement {
  div()
    .absolute()
    .top(px(TITLE_BAR_HEIGHT))
    .left_0()
    .right_0()
    .h(px(BOUNDARY_THICKNESS))
    .bg(rgb(COLOR_GREY_50))
}

pub struct AppRoot<V> {
  pub root_focus: FocusHandle,
  pub root: Entity<V>,
}

impl<V: Render + 'static> Render for AppRoot<V> {
  fn render(
    &mut self,
    window: &mut Window,
    _cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let chrome = window_decoration::window_chrome(window);

    let body = div()
      .flex()
      .flex_col()
      .size_full()
      .track_focus(&self.root_focus)
      .bg(rgb(COLOR_GREY_900))
      .text_color(rgb(COLOR_GREY_50))
      .border_color(rgb(COLOR_GREY_800))
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
      .relative()
      .child(self.title_bar(window))
      // The content is not inset: the delegate's root owns this coordinate
      // space from y=0, so it can place elements in the strip. It insets its own
      // content instead, leaving the strip's height to it.
      .child(self.root.clone())
      // The controls come last, after the content has painted its own top area
      // over the strip: they are the shell's, and stay above the application.
      .child(self.window_controls());
      // .child(strip_boundary());

    window_decoration::window_frame()
      .radius(px(WINDOW_RADIUS))
      .child(body)
  }
}

impl<V> AppRoot<V> {
  /// The caption buttons, pinned to the top right, painted last so the content
  /// reaching the top of the window cannot paint them out.
  ///
  /// The controls are always the shell's to draw here, since this platform never
  /// paints its own frame; gpui only stops routing them once the window is
  /// fullscreen.
  fn window_controls(&self) -> impl IntoElement {
    div()
      .absolute()
      .top_0()
      .right_0()
      .flex()
      .flex_row()
      .h(px(TITLE_BAR_HEIGHT))
      .child(WindowControl::Minimize)
      .child(WindowControl::Maximize)
      .child(WindowControl::Close)
  }

  /// The drag area the platform moves the window from, painted first, so the
  /// strip is only the drag target and the content draws over it.
  ///
  /// Dragging is declared to Windows as a [`WindowControlArea::Drag`] region
  /// rather than driven from mouse events, because gpui's Windows backend does
  /// not implement [`Window::start_window_move`]: the platform moves the window
  /// itself, which also keeps snapping and the caption's double-click
  /// maximise/restore. The strip is painted without a background, so the
  /// content's own top area shows through it: this is the window's top region,
  /// not the application's.
  fn title_bar(&self, _window: &Window) -> impl IntoElement {
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
      .child(
        div()
          .id("title-bar-drag")
          .flex()
          .flex_1()
          .items_center()
          .h_full()
          .window_control_area(WindowControlArea::Drag)
          .child(
            div()
              .pl(px(TRAFFIC_LIGHT_RESERVED_WIDTH))
              .pt(px(1.5))
              .flex()
              .items_center(),
          ),
      )
  }
}
