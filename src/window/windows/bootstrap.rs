use std::future::Future;
use std::rc::Rc;

use gpui_kit::component::Root;
use gpui_kit::component::Theme;
use gpui_kit::component::ThemeMode;
use gpui_kit::component::TitleBar;
use gpui_kit::*;

use super::app_window::AppRoot;
use super::dwm;
use super::typography;
use crate::app::NewWindow;

const TITLE_BAR_HEIGHT: f32 = 35.;
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
        traffic_light_position: None,
      });

      options.window_background = WindowBackgroundAppearance::Transparent;
      // gpui's Windows backend has no client-side decorations: the window keeps
      // its native frame and `dwm` keeps it square, so no decoration mode is
      // requested here.

      spawn_callback(cx, &mut options).await;

      cx.open_window(options, move |window, cx| {
        dwm::prefer_square_corners(window);

        window_callback(window, cx);

        let view = cx.new(|cx| AppRoot {
          root_focus: cx.focus_handle().tab_stop(false),
          root: root_component(window, cx),
        });

        cx.new(|cx| {
          Root::new(view, window, cx)
            .bordered(false)
            .bg(rgba(0x00000000))
            .font(typography::app_font())
        })
      })
      .expect("failed to open window");
    })
    .detach();
  };

  gpui_kit::application().run(move |cx| {
    gpui_kit::init(cx);

    cx.set_cursor_hide_mode(CursorHideMode::Never);

    crate::theme::set_mode(ThemeMode::Dark, cx);

    init_callback(cx);

    cx.on_action({
      let open_main_window = open_main_window.clone();
      move |_: &NewWindow, cx| open_main_window(cx)
    });

    open_main_window(cx);
  });
}

pub fn apply_theme_overrides(cx: &mut App) {
  Theme::global_mut(cx).font_family = typography::FONT_FAMILY.into();
}
