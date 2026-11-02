use gpui_kit::*;

#[derive(Clone, Copy, Default)]
pub struct WindowChrome {
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
    Decorations::Server => WindowChrome::default(),
    Decorations::Client { tiling } => WindowChrome {
      border_top: !tiling.top,
      border_right: !tiling.right,
      border_bottom: !tiling.bottom,
      border_left: !tiling.left,
    },
  }
}
