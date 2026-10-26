use std::future::Future;
use std::sync::Arc;

use gpui_kit::component::Root;
use gpui_kit::component::Theme;
use gpui_kit::component::ThemeMode;
use gpui_kit::component::TitleBar;
use gpui_kit::*;

use super::app_window::AppRoot;

const TITLE_BAR_HEIGHT: f32 = 42.;
const TRAFFIC_LIGHT_RESERVED_WIDTH: f32 = 10.;

const WINDOW_RADIUS: f32 = 12.;

pub fn bootstrap<V, F, Fut>(
  init_callback: impl FnOnce(&mut App) + 'static,
  spawn_callback: F,
  root_component: impl FnOnce(&mut Window, &mut App) -> Entity<V> + 'static,
  window_callback: impl FnOnce(&mut Window, &mut App) + 'static,
) where
  V: Render + 'static,
  F: FnOnce(&mut AsyncApp, &mut WindowOptions) -> Fut + 'static,
  Fut: Future<Output = ()> + 'static,
{
  gpui_kit::application().run(|cx| {
    gpui_kit::init(cx);

    cx.set_cursor_hide_mode(CursorHideMode::Never);

    init_callback(cx);

    cx.spawn(async move |cx| {
      let mut options = TitleBar::window_options();

      options.window_bounds = Some(WindowBounds::Windowed(Bounds::new(
        point(px(120.), px(120.)),
        size(px(1460.), px(978.)),
      )));

      options.window_min_size = Some(size(px(600.), px(400.)));

      options.titlebar = Some(TitlebarOptions {
        title: None,
        appears_transparent: true,
        traffic_light_position: None,
      });

      options.window_background = WindowBackgroundAppearance::Transparent;
      options.window_decorations = Some(WindowDecorations::Client);

      spawn_callback(cx, &mut options).await;

      cx.open_window(options, move |window, cx| {
        Theme::change(ThemeMode::Dark, Some(window), cx);
        // The component library has no per-input placeholder color; this theme
        // token is the only hook, and it also dims the library's other muted text.
        Theme::global_mut(cx).muted_foreground =
          rgb(crate::theme::colors::COLOR_INPUT_PLACEHOLDER_FOREGROUND).into();
        let highlight_theme = Arc::make_mut(&mut Theme::global_mut(cx).highlight_theme);
        highlight_theme.style.editor_background =
          Some(rgb(crate::theme::colors::COLOR_EDITOR_BACKGROUND).into());
        highlight_theme.style.editor_active_line =
          Some(rgb(crate::theme::colors::COLOR_EDITOR_LINE_HIGHLIGHT_BACKGROUND).into());

        window_callback(window, cx);

        let view = cx.new(|cx| AppRoot {
          root_focus: cx.focus_handle().tab_stop(false),
          root: root_component(window, cx),
        });

        cx.new(|cx| {
          Root::new(view, window, cx)
            .bordered(false)
            .bg(rgba(0x00000000))
        })
      })
      .expect("failed to open window");
    })
    .detach();
  });
}
