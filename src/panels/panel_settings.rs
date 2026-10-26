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
use crate::services::config::Config;
use crate::theme;

pub struct DirtyChanged;

pub struct PanelSettings {
  config_path: PathBuf,
  editor: Entity<EditorState>,
  saved_contents: SharedString,
  dirty: bool,
  save_error: Option<SharedString>,
  _subscriptions: Vec<Subscription>,
}

impl EventEmitter<DirtyChanged> for PanelSettings {}

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
      save_error: None,
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
    // The app refuses to start on a config it cannot load, so never write one.
    let saved = Config::from_yaml(&contents, &self.config_path)
      .and_then(|_| Ok(fs::write(&self.config_path, contents.as_bytes())?));
    match saved {
      Ok(()) => {
        self.saved_contents = contents;
        self.save_error = None;
        self.update_dirty(cx);
      }
      Err(error) => self.save_error = Some(format!("{error:#}").into()),
    }
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
      .children(self.save_error.clone().map(|error| {
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
