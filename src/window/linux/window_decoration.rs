use gpui_kit::*;

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

pub fn window_chrome(window: &Window) -> WindowChrome {
  if window.is_maximized() || window.is_fullscreen() {
    return WindowChrome::default();
  }

  match window.window_decorations() {
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
