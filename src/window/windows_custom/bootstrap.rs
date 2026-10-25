use gpui_kit::component::Root;
use gpui_kit::component::Theme;
use gpui_kit::component::ThemeMode;
use gpui_kit::component::TitleBar;
use gpui_kit::*;

use std::future::Future;

use super::app_window::AppRoot;
use super::dwm;
use super::hit_test;
use super::window_decoration;

const TITLE_BAR_HEIGHT: f32 = 48.;
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

        dwm::disable_non_client_rendering(window);
        hit_test::extend_resize_border(
          window,
          f32::from(window_decoration::SHADOW_SIZE),
          f32::from(window_decoration::RESIZE_HIT_SIZE),
        );

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
