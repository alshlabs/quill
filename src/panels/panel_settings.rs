use std::fs;
use std::path;
use std::path::PathBuf;

use gpui_kit::base::v_flex;
use gpui_kit::component::input::Editor;
use gpui_kit::component::input::EditorState;
use gpui_kit::component::input::InputEvent;
use gpui_kit::*;

use crate::app::Save;
use crate::services::AppServices;
use crate::services::agent;
use crate::services::config::Config;
use crate::theme;

pub struct DirtyChanged;

pub struct PanelSettings {
  config_path: PathBuf,
  editor: Entity<EditorState>,
  saved_contents: SharedString,
  dirty: bool,
  config_error: Option<SharedString>,
  _subscriptions: Vec<Subscription>,
}

impl EventEmitter<DirtyChanged> for PanelSettings {}

impl PanelSettings {
  pub fn new(
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> Self {
    let services = AppServices::obtain();
    let config_path = services.config_path();
    // Read from disk rather than the loaded `Config`, which has its env
    // references already interpolated.
    let contents = fs::read_to_string(config_path)
      .unwrap_or_else(|error| format!("# Failed to read {}: {error}\n", config_path.display()));

    // `QUILL_CONFIG` may hold a relative path.
    let absolute_path = path::absolute(config_path).unwrap_or_else(|_| config_path.to_path_buf());

    let editor = cx.new(|cx| {
      EditorState::new(window, cx)
        .language("yaml")
        .default_value(contents)
    });
    let on_change = cx.subscribe(&editor, |this, _, event: &InputEvent, cx| {
      if let InputEvent::Change = event {
        this.update_dirty(cx);
      }
    });

    Self {
      config_path: absolute_path,
      saved_contents: editor.read(cx).value(),
      editor,
      dirty: false,
      config_error: config_error(&services.config()),
      _subscriptions: vec![on_change],
    }
  }

  pub fn is_dirty(&self) -> bool {
    self.dirty
  }

  fn update_dirty(
    &mut self,
    cx: &mut Context<Self>,
  ) {
    let dirty = self.editor.read(cx).value() != self.saved_contents;
    if dirty != self.dirty {
      self.dirty = dirty;
      cx.emit(DirtyChanged);
    }
  }

  fn save(
    &mut self,
    _: &Save,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let contents = self.editor.read(cx).value();
    if let Err(error) = fs::write(&self.config_path, contents.as_bytes()) {
      self.config_error = Some(format!("Failed to save: {error}").into());
      cx.notify();
      return;
    }
    self.saved_contents = contents.clone();
    self.update_dirty(cx);

    let config = Config::from_yaml(&contents, &self.config_path);
    if let Ok(config) = &config {
      agent::apply_config(config);
    }
    self.config_error = config_error(&config);
    AppServices::obtain().set_config(config);
    cx.notify();
  }
}

impl Render for PanelSettings {
  fn render(
    &mut self,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    v_flex()
      .id("panel_settings")
      .size_full()
      .on_action(cx.listener(Self::save))
      .child(
        div()
          .font_weight(FontWeight::LIGHT)
          .text_sm()
          .pt(px(2.0))
          .pl(px(12.0))
          .pb(px(2.0))
          .text_color(rgb(theme::colors::COLOR_INPUT_PLACEHOLDER_FOREGROUND))
          .child(self.config_path.display().to_string()),
      )
      .children(self.config_error.clone().map(|error| {
        div()
          .text_sm()
          .py(px(2.0))
          .pl(px(12.0))
          .text_color(rgb(theme::colors::COLOR_ERROR_FOREGROUND))
          .child(error)
      }))
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

fn config_error(config: &anyhow::Result<Config>) -> Option<SharedString> {
  config
    .as_ref()
    .err()
    .map(|error| format!("{error:#}").into())
}
