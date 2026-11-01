#![allow(unused)]
pub mod colors;

use std::sync::Arc;

pub use colors::*;
use gpui_kit::App;
use gpui_kit::component::Theme;
use gpui_kit::component::ThemeMode;
use gpui_kit::rgb;

/// Use this rather than `Theme::change`, which drops the overrides below and
/// repaints at most the one window it is given, leaving other windows stale.
pub fn set_mode(
  mode: ThemeMode,
  cx: &mut App,
) {
  Theme::change(mode, None, cx);
  // The component library has no per-input placeholder color; this theme
  // token is the only hook, and it also dims the library's other muted text.
  Theme::global_mut(cx).muted_foreground = rgb(COLOR_INPUT_PLACEHOLDER_FOREGROUND).into();
  Theme::global_mut(cx).popover = rgb(COLOR_PANEL_BACKGROUND).into();
  let highlight_theme = Arc::make_mut(&mut Theme::global_mut(cx).highlight_theme);
  highlight_theme.style.editor_background = Some(rgb(COLOR_EDITOR_BACKGROUND).into());
  highlight_theme.style.editor_active_line =
    Some(rgb(COLOR_EDITOR_LINE_HIGHLIGHT_BACKGROUND).into());
  crate::window::apply_theme_overrides(cx);
  cx.refresh_windows();
}
