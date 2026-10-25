//! Windows window shell.
//!
//! DWM owns the window shape here: it draws the frame and the drop shadow from
//! the window's rectangle and its own corner preference, ignoring the alpha of
//! the painted content. The content is therefore painted square and opaque edge
//! to edge, and [`super::dwm::prefer_square_corners`] keeps the frame square, so
//! the corners and the shadow always agree.
//!
//! [`super::window_decoration`] (the rounded frame, shadow and resize zones used
//! on Linux) never renders on Windows: gpui's Windows backend reports
//! `Decorations::Server` for every window, so `WindowFrame` takes its early
//! return and its radius, inset, shadow and hit zones stay unused.
//!
//! The title bar is drawn here, because it carries the app's own
//! [`WindowControl`] buttons: the component library's title bar always renders
//! its own controls, whose icons this application cannot resolve. Dragging stays
//! with the platform — the strip is declared as a [`WindowControlArea::Drag`]
//! region, since gpui's Windows backend does not implement
//! [`Window::start_window_move`] — which also keeps snapping and the caption's
//! double-click maximise/restore.
//!
//! The strip is painted first, then the content, then the controls, so the
//! controls stay above the content, which reaches the top of the window and
//! would otherwise paint them out. Deferring the strip would put it above the
//! content for both paint and hit testing, leaving the content unable to cover
//! it and blocking its clicks; deferred drawing outranks the whole normal tree,
//! so no priority reaches over it.
//!
//! The strip must stop short of the controls. gpui answers the platform's hit
//! test by scanning the frame's control hitboxes in paint order and taking the
//! first one under the cursor — not the frontmost, which is what hover uses — so
//! a strip reaching under the buttons answers for them, and Windows then runs
//! the caption's command: a click on close drags the window instead of closing
//! it. The strip therefore reserves [`WINDOW_CONTROLS_WIDTH`] at the right edge.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::window_controls::WINDOW_CONTROLS;
use super::window_controls::WINDOW_CONTROLS_WIDTH;
use super::window_decoration::{self};
use crate::theme::colors::COLOR_GREY_50;
use crate::theme::colors::COLOR_GREY_900;

const TITLE_BAR_HEIGHT: f32 = 42.;
const TRAFFIC_LIGHT_RESERVED_WIDTH: f32 = 10.;
const BOUNDARY_THICKNESS: f32 = 1.;

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
    let body = div()
      .flex()
      .flex_col()
      .size_full()
      .track_focus(&self.root_focus)
      // .bg(rgb(0xffffff))
      .bg(rgb(COLOR_GREY_900))
      .text_color(rgb(COLOR_GREY_50))
      .relative()
      .child(self.title_bar(window))
      // The content is not inset: the delegate's root owns this coordinate
      // space from y=0, so it can place elements in the strip. It insets its own
      // content instead, leaving the strip's height to it.
      .child(self.root.clone())
      // The controls come last, after the content has painted its own top area
      // over the strip: they are the shell's, and stay above the application.
      .child(self.window_controls(window));
    // .child(strip_boundary());

    window_decoration::window_frame().child(body)
  }
}

impl<V> AppRoot<V> {
  /// The caption buttons, pinned to the top right, painted last so the content
  /// reaching the top of the window cannot paint them out.
  ///
  /// The controls are the shell's to draw, since the component library's bar
  /// cannot drop its own; they stay hidden in fullscreen, where the platform
  /// stops offering the control areas and the buttons could not act.
  fn window_controls(
    &self,
    window: &Window,
  ) -> impl IntoElement {
    let show_controls = !window.is_fullscreen();

    div()
      .absolute()
      .top_0()
      .right_0()
      .flex()
      .flex_row()
      .h(px(TITLE_BAR_HEIGHT))
      .when(show_controls, |this| this.children(WINDOW_CONTROLS))
  }

  /// The drag area the platform moves the window from, painted first, so the
  /// strip is only the drag target and the content draws over it.
  ///
  /// The strip is painted without a background, so the content's own top area
  /// shows through it: this is the window's top region, not the application's.
  /// It ends where the controls begin, which keeps the platform's hit test on
  /// them resolving to their own areas.
  fn title_bar(
    &self,
    window: &Window,
  ) -> impl IntoElement {
    div()
      .id("title-bar")
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
      .when(!window.is_fullscreen(), |this| {
        this.child(div().flex_shrink_0().w(px(WINDOW_CONTROLS_WIDTH)).h_full())
      })
  }
}
