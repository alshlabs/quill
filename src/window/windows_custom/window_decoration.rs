//! Windows client-side window frame.
//!
//! gpui's Windows backend never reports client-side decorations: `Decorations`
//! defaults to `Server` on the platform window trait and neither
//! `window_decorations` nor `request_decorations` is overridden there, so the
//! platform answer cannot decide what to paint. [`super::dwm`] switches DWM's
//! non-client rendering off, so the window really is ours to decorate, and
//! [`decorations`] reports that instead of asking the platform.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

pub(super) const SHADOW_SIZE: Pixels = px(16.);
pub(super) const RESIZE_HIT_SIZE: Pixels = px(4.);
const RADIUS: Pixels = px(12.);

#[derive(Clone, Copy, Default)]
pub struct WindowChrome {
  pub top_left: bool,
  pub top_right: bool,
  pub bottom_left: bool,
  pub bottom_right: bool,
  pub border_top: bool,
  pub border_right: bool,
  pub border_bottom: bool,
  pub border_left: bool,
}

/// The decorations this shell draws for itself.
///
/// Windows never tiles, and DWM's non-client rendering is disabled in
/// [`super::dwm`], so every window is client-decorated with untiled edges.
/// Maximized and fullscreen windows are handled by [`window_chrome`], which
/// paints no corners, padding or shadow for them.
fn decorations() -> Decorations {
  Decorations::Client {
    tiling: Tiling::default(),
  }
}

pub fn window_chrome(window: &Window) -> WindowChrome {
  if window.is_maximized() || window.is_fullscreen() {
    return WindowChrome::default();
  }

  match decorations() {
    Decorations::Server => WindowChrome {
      top_left: true,
      top_right: true,
      bottom_left: true,
      bottom_right: true,
      ..Default::default()
    },
    Decorations::Client { tiling } => WindowChrome {
      top_left: !(tiling.top || tiling.left),
      top_right: !(tiling.top || tiling.right),
      bottom_left: !(tiling.bottom || tiling.left),
      bottom_right: !(tiling.bottom || tiling.right),
      border_top: !tiling.top,
      border_right: !tiling.right,
      border_bottom: !tiling.bottom,
      border_left: !tiling.left,
    },
  }
}

pub fn window_frame() -> WindowFrame {
  WindowFrame::new()
}

#[derive(IntoElement)]
pub struct WindowFrame {
  radius: Pixels,
  shadow_size: Pixels,
  resize_hit_size: Pixels,
  children: Vec<AnyElement>,
}

impl Default for WindowFrame {
  fn default() -> Self {
    Self {
      radius: RADIUS,
      shadow_size: SHADOW_SIZE,
      resize_hit_size: RESIZE_HIT_SIZE,
      children: Vec::new(),
    }
  }
}

impl WindowFrame {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn radius(
    mut self,
    radius: impl Into<Pixels>,
  ) -> Self {
    self.radius = radius.into();
    self
  }
}

impl ParentElement for WindowFrame {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements);
  }
}

fn client_frame_insets(
  shadow_size: Pixels,
  tiling: &Tiling,
) -> Edges<Pixels> {
  let mut insets = Edges::all(shadow_size);
  if tiling.top {
    insets.top = px(0.);
  }
  if tiling.bottom {
    insets.bottom = px(0.);
  }
  if tiling.left {
    insets.left = px(0.);
  }
  if tiling.right {
    insets.right = px(0.);
  }
  insets
}

fn cursor_for_edge(edge: ResizeEdge) -> CursorStyle {
  match edge {
    ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
    ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
    ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
    ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
  }
}

fn resize_edge(
  pos: Point<Pixels>,
  size: Size<Pixels>,
  insets: Edges<Pixels>,
  tiling: &Tiling,
  hit_size: Pixels,
) -> Option<ResizeEdge> {
  let inner_left = insets.left;
  let inner_right = size.width - insets.right;
  let inner_top = insets.top;
  let inner_bottom = size.height - insets.bottom;

  let on_left = pos.x >= inner_left - hit_size
    && pos.x <= inner_left + hit_size
    && pos.y >= inner_top - hit_size
    && pos.y <= inner_bottom + hit_size;
  let on_right = pos.x >= inner_right - hit_size
    && pos.x <= inner_right + hit_size
    && pos.y >= inner_top - hit_size
    && pos.y <= inner_bottom + hit_size;
  let on_top = pos.y >= inner_top - hit_size
    && pos.y <= inner_top + hit_size
    && pos.x >= inner_left - hit_size
    && pos.x <= inner_right + hit_size;
  let on_bottom = pos.y >= inner_bottom - hit_size
    && pos.y <= inner_bottom + hit_size
    && pos.x >= inner_left - hit_size
    && pos.x <= inner_right + hit_size;

  if !tiling.top && !tiling.left && on_top && on_left {
    return Some(ResizeEdge::TopLeft);
  }
  if !tiling.top && !tiling.right && on_top && on_right {
    return Some(ResizeEdge::TopRight);
  }
  if !tiling.bottom && !tiling.left && on_bottom && on_left {
    return Some(ResizeEdge::BottomLeft);
  }
  if !tiling.bottom && !tiling.right && on_bottom && on_right {
    return Some(ResizeEdge::BottomRight);
  }
  if !tiling.top && on_top {
    return Some(ResizeEdge::Top);
  }
  if !tiling.bottom && on_bottom {
    return Some(ResizeEdge::Bottom);
  }
  if !tiling.left && on_left {
    return Some(ResizeEdge::Left);
  }
  if !tiling.right && on_right {
    return Some(ResizeEdge::Right);
  }

  None
}

fn resize_hit_zones(
  window_size: Size<Pixels>,
  shadow_size: Pixels,
  hit_size: Pixels,
  tiling: &Tiling,
) -> Vec<AnyElement> {
  if tiling.is_tiled() {
    return Vec::new();
  }

  let insets = client_frame_insets(shadow_size, tiling);
  let inner_left = insets.left;
  let inner_right = window_size.width - insets.right;
  let inner_top = insets.top;
  let inner_bottom = window_size.height - insets.bottom;
  let frame_origin = point(insets.left, insets.top);
  let band = hit_size + hit_size;
  let span_x = inner_right - inner_left + band;
  let span_y = inner_bottom - inner_top + band;

  let mut zones: Vec<AnyElement> = Vec::new();

  let mut push_zone = |edge: ResizeEdge, origin: Point<Pixels>, zone_size: Size<Pixels>| {
    let origin = origin - frame_origin;
    zones.push(
      div()
        .absolute()
        .left(origin.x)
        .top(origin.y)
        .w(zone_size.width)
        .h(zone_size.height)
        .cursor(cursor_for_edge(edge))
        .into_any_element(),
    );
  };

  if !tiling.top {
    push_zone(
      ResizeEdge::Top,
      point(inner_left - hit_size, inner_top - hit_size),
      Size::new(span_x, band),
    );
  }
  if !tiling.bottom {
    push_zone(
      ResizeEdge::Bottom,
      point(inner_left - hit_size, inner_bottom - hit_size),
      Size::new(span_x, band),
    );
  }
  if !tiling.left {
    push_zone(
      ResizeEdge::Left,
      point(inner_left - hit_size, inner_top - hit_size),
      Size::new(band, span_y),
    );
  }
  if !tiling.right {
    push_zone(
      ResizeEdge::Right,
      point(inner_right - hit_size, inner_top - hit_size),
      Size::new(band, span_y),
    );
  }
  if !tiling.top && !tiling.left {
    push_zone(
      ResizeEdge::TopLeft,
      point(inner_left - hit_size, inner_top - hit_size),
      Size::new(band, band),
    );
  }
  if !tiling.top && !tiling.right {
    push_zone(
      ResizeEdge::TopRight,
      point(inner_right - hit_size, inner_top - hit_size),
      Size::new(band, band),
    );
  }
  if !tiling.bottom && !tiling.left {
    push_zone(
      ResizeEdge::BottomLeft,
      point(inner_left - hit_size, inner_bottom - hit_size),
      Size::new(band, band),
    );
  }
  if !tiling.bottom && !tiling.right {
    push_zone(
      ResizeEdge::BottomRight,
      point(inner_right - hit_size, inner_bottom - hit_size),
      Size::new(band, band),
    );
  }

  zones
}

impl RenderOnce for WindowFrame {
  fn render(
    self,
    window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    let Decorations::Client { tiling } = decorations() else {
      return div().size_full().children(self.children).into_any_element();
    };

    let platform_inset = self.shadow_size;
    let is_floating = !window.is_maximized() && !window.is_fullscreen();
    let visual_shadow = if tiling.is_tiled() || !is_floating {
      px(0.)
    } else {
      self.shadow_size
    };
    let resize_hit_size = self.resize_hit_size;
    let radius = self.radius;
    let chrome = window_chrome(window);
    let window_size = window.window_bounds().get_bounds().size;
    let is_active = window.is_window_active();
    let show_shadow = is_floating && !tiling.is_tiled();
    // gpui starts resizes and paints the resize cursor from its own
    // `WM_NCHITTEST` edges at the window boundary, and `Window::start_window_resize`
    // is a no-op on Windows, so these overlay zones would only promise a resize
    // that cannot start where the cursor suggests it.
    let show_resize_zones = false;

    window.set_client_inset(platform_inset);

    div()
      .id("window-frame")
      .bg(transparent_black())
      .flex()
      .flex_col()
      .overflow_hidden()
      .when(!tiling.top && is_floating, |this| this.pt(visual_shadow))
      .when(!tiling.bottom && is_floating, |this| this.pb(visual_shadow))
      .when(!tiling.left && is_floating, |this| this.pl(visual_shadow))
      .when(!tiling.right && is_floating, |this| this.pr(visual_shadow))
      .on_mouse_down(MouseButton::Left, move |_, window, _| {
        let Decorations::Client { tiling } = decorations() else {
          return;
        };
        if tiling.is_tiled() {
          return;
        }
        let size = window.window_bounds().get_bounds().size;
        let pos = window.mouse_position();
        let insets = client_frame_insets(platform_inset, &tiling);
        if let Some(edge) = resize_edge(pos, size, insets, &tiling, resize_hit_size) {
          window.start_window_resize(edge);
        }
      })
      .size_full()
      .child(
        div()
          .cursor(CursorStyle::default())
          .flex_1()
          .min_h_0()
          .min_w_0()
          .overflow_hidden()
          .when(chrome.top_left, |this| this.rounded_tl(radius))
          .when(chrome.top_right, |this| this.rounded_tr(radius))
          .when(chrome.bottom_left, |this| this.rounded_bl(radius))
          .when(chrome.bottom_right, |this| this.rounded_br(radius))
          .when(show_shadow, |this| {
            let opacity = if is_active { 1.0 } else { 0.7 };
            this.shadow(vec![
              BoxShadow {
                color: Hsla {
                  h: 0.,
                  s: 0.,
                  l: 0.,
                  a: 0.18 * opacity,
                },
                blur_radius: px(10.),
                spread_radius: px(-1.),
                offset: point(px(0.), px(2.)),
                inset: false,
              },
              BoxShadow {
                color: Hsla {
                  h: 0.,
                  s: 0.,
                  l: 0.,
                  a: 0.18 * opacity,
                },
                blur_radius: px(3.),
                spread_radius: px(0.),
                offset: point(px(0.), px(1.)),
                inset: false,
              },
            ])
          })
          .on_mouse_move(|_, _, cx| {
            cx.stop_propagation();
          })
          .bg(transparent_black())
          .children(self.children),
      )
      .when(show_resize_zones, |this| {
        this.child(div().absolute().size_full().children(resize_hit_zones(
          window_size,
          platform_inset,
          resize_hit_size,
          &tiling,
        )))
      })
      .into_any_element()
  }
}
