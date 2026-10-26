use std::fs;
use std::path;

use gpui_kit::base::v_flex;
use gpui_kit::component::input::Editor;
use gpui_kit::component::input::EditorState;
use gpui_kit::*;

use crate::services::AppServices;
use crate::theme;

pub struct PanelSettings {
  config_path: SharedString,
  editor: Entity<EditorState>,
}

impl PanelSettings {
  pub fn new(
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> Self {
    let config_path = &AppServices::obtain().config.config_path;
    // Read from disk rather than the loaded `Config`, which has its env
    // references already interpolated.
    let contents = fs::read_to_string(config_path)
      .unwrap_or_else(|error| format!("# Failed to read {}: {error}\n", config_path.display()));

    // `QUILL_CONFIG` may hold a relative path.
    let absolute_path = path::absolute(config_path).unwrap_or_else(|_| config_path.clone());

    Self {
      config_path: absolute_path.display().to_string().into(),
      editor: cx.new(|cx| {
        EditorState::new(window, cx)
          .language("yaml")
          .default_value(contents)
      }),
    }
  }
}

impl Render for PanelSettings {
  fn render(
    &mut self,
    _window: &mut Window,
    _cx: &mut Context<Self>,
  ) -> impl IntoElement {
    v_flex()
      .id("panel_settings")
      .size_full()
      .child(
        div()
          .font_weight(FontWeight::LIGHT)
          .text_sm()
          .pt(px(2.0))
          .pl(px(12.0))
          .pb(px(2.0))
          .text_color(rgb(theme::colors::COLOR_INPUT_PLACEHOLDER_FOREGROUND))
          .child(self.config_path.clone()),
      )
      .child(
        div().flex_1().min_h_0().overflow_hidden().child(
          Editor::new(&self.editor)
            .size_full()
            // `Editor` pads its text 8px from the top and exposes no way to
            // change it, so the padding is shifted out of the clipped parent.
            .mt(px(-8.0))
            .bordered(false)
            .bg(transparent_black()),
        ),
      )
  }
}
