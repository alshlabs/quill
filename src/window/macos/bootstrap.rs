use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::component::Root;
use gpui_kit::component::Theme;
use gpui_kit::component::ThemeMode;
use gpui_kit::component::TitleBar;
use gpui_kit::*;

use super::app_window::AppRoot;
use crate::theme;

const TRAFFIC_LIGHT_X: f32 = 15.;
const TRAFFIC_LIGHT_Y: f32 = 14.0;

const TITLE_BAR_HEIGHT: f32 = 48.;
const TRAFFIC_LIGHT_RESERVED_WIDTH: f32 = 10.;

pub fn bootstrap<V, F, Fut>(
  init_callback: impl FnOnce(&mut App) + 'static,
  spawn_callback: F,
  root_component: impl Fn(&mut Window, &mut App) -> Entity<V> + 'static,
  window_callback: impl Fn(&mut Window, &mut App) + 'static,
) where
  V: Render + 'static,
  F: Fn(&mut AsyncApp, &mut WindowOptions) -> Fut + 'static,
  Fut: Future<Output = ()> + 'static,
{
  let callbacks = Rc::new((spawn_callback, root_component, window_callback));

  let open_main_window = move |cx: &mut App| {
    let callbacks = callbacks.clone();
    cx.spawn(async move |cx| {
      let (spawn_callback, root_component, window_callback) = &*callbacks;
      let mut options = TitleBar::window_options();

      options.window_bounds = Some(WindowBounds::Windowed(Bounds::new(
        point(px(120.), px(120.)),
        size(px(1460.), px(978.)),
      )));

      options.window_min_size = Some(size(px(crate::window::WINDOW_MIN_WIDTH), px(400.)));

      options.titlebar = Some(TitlebarOptions {
        title: None,
        appears_transparent: true,
        traffic_light_position: Some(point(px(TRAFFIC_LIGHT_X), px(TRAFFIC_LIGHT_Y))),
      });

      // options.window_background = WindowBackgroundAppearance::Blurred;
      options.window_background = WindowBackgroundAppearance::Opaque;
      // options.window_background = WindowBackgroundAppearance::Transparent;
      options.window_decorations = Some(WindowDecorations::Client);

      spawn_callback(cx, &mut options).await;

      cx.open_window(options, move |window, cx| {
        Theme::change(ThemeMode::Dark, Some(window), cx);
        // The component library has no per-input placeholder color; this theme
        // token is the only hook, and it also dims the library's other muted text.
        Theme::global_mut(cx).muted_foreground =
          rgb(crate::theme::colors::COLOR_INPUT_PLACEHOLDER_FOREGROUND).into();
        Theme::global_mut(cx).popover = rgb(crate::theme::colors::COLOR_PANEL_BACKGROUND).into();
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
            // .bg(rgba(0x00000000))
            .bg(rgba(theme::colors::COLOR_BACKGROUND_500))
        })
      })
      .expect("failed to open window");
    })
    .detach();
  };

  let app = gpui_kit::application();

  app.on_reopen({
    let open_main_window = open_main_window.clone();
    move |cx| open_main_window(cx)
  });

  app.run(move |cx| {
    gpui_kit::init(cx);

    cx.set_cursor_hide_mode(CursorHideMode::Never);

    init_callback(cx);

    open_main_window(cx);
  });
}
