use gpui_kit::AnyElement;
use gpui_kit::RenderOnce;
use gpui_kit::base::h_flex;
use gpui_kit::base::v_flex;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::app::NewChat;
use crate::icons::ICON_ASTERISK;
use crate::icons::ICON_COMMAND;
use crate::icons::ICON_COMMENT;
use crate::icons::ICON_FEATHER;
use crate::icons::ICON_FOLDER_OPENED;
use crate::icons::ICON_GEAR;
use crate::icons::ICON_SHIFT;
use crate::icons::ICON_THREE_BARS;
use crate::theme;

#[derive(IntoElement)]
pub struct PanelAboutBlank {
  id: ElementId,
  children: Vec<AnyElement>,
  style: StyleRefinement,
}

impl PanelAboutBlank {
  pub fn new(id: impl Into<ElementId>) -> Self {
    Self {
      id: id.into(),
      children: Vec::new(),
      style: StyleRefinement::default(),
    }
  }
}

impl Styled for PanelAboutBlank {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for PanelAboutBlank {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for PanelAboutBlank {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    body(&self.id)
  }
}

fn body(id: &ElementId) -> Stateful<Div> {
  v_flex()
    .id(format!("panel_about_blank_{}", id))
    .on_click(|event, window, cx| {
      if event.click_count() == 2 {
        window.dispatch_action(NewChat.boxed_clone(), cx);
      }
    })
    .justify_center()
    .items_center()
    .w_full()
    .h_full()
    .text_color(rgb(theme::colors::COLOR_FOREGROUND))
    .child(
      Icon::empty()
        .data(ICON_ASTERISK)
        .data(ICON_FEATHER)
        .h(px(270.0))
        .w(px(270.0))
        .text_color(rgb(theme::colors::COLOR_LETTERPRESS)),
    )
    .child(
      v_flex()
        .gap(px(8.0))
        .child(suggested_item(
          ICON_COMMENT,
          "Start new chat",
          shortcut(&[], "N"),
        ))
        .child(suggested_item(
          ICON_FOLDER_OPENED,
          "Open Workspace",
          shortcut(&[], "O"),
        ))
        .child(suggested_item(
          ICON_THREE_BARS,
          "Show All Commands",
          shortcut(&[Modifier::Shift], "P"),
        ))
        .child(suggested_item(
          ICON_GEAR,
          "Configure Agents",
          shortcut(&[], ","),
        )),
    )
}

enum Modifier {
  Shift,
}

fn shortcut(
  modifiers: &[Modifier],
  key: &'static str,
) -> Vec<AnyElement> {
  let mut keys = Vec::new();
  if cfg!(target_os = "macos") {
    keys.extend(modifiers.iter().map(|modifier| match modifier {
      Modifier::Shift => key_icon(ICON_SHIFT),
    }));
    keys.push(key_icon(ICON_COMMAND));
  } else {
    keys.push("Ctrl".into_any_element());
    keys.extend(modifiers.iter().map(|modifier| match modifier {
      Modifier::Shift => "Shift".into_any_element(),
    }));
  }
  keys.push(key.into_any_element());
  keys
}

fn key_icon(icon: &'static [u8]) -> AnyElement {
  Icon::empty()
    .data(icon)
    .h(px(11.0))
    .w(px(11.0))
    .into_any_element()
}

fn suggested_item(
  icon: &'static [u8],
  text: &str,
  keys: impl IntoIterator<Item = AnyElement>,
) -> Div {
  h_flex()
    .w(px(300.0))
    .gap(px(8.0))
    .items_center()
    .child(
      Icon::empty()
        .data(icon)
        .h(px(16.0))
        .w(px(16.0))
        .text_color(rgb(theme::colors::COLOR_ICON_FOREGROUND)),
    )
    .child(label(text))
    .child(
      h_flex()
        .gap(px(4.0))
        .children(keys.into_iter().map(|key| keycap().child(key))),
    )
}

fn label(text: &str) -> Div {
  div()
    .flex_1()
    .font_weight(FontWeight::LIGHT)
    .text_sm()
    .text_color(rgb(theme::colors::COLOR_WATERMARK_FOREGROUND))
    .child(text.to_string())
}

fn keycap() -> Div {
  h_flex()
    .justify_center()
    .items_center()
    .min_w(px(24.0))
    .h(px(22.0))
    .px(px(6.0))
    .rounded(px(4.0))
    .border_1()
    .border_color(rgb(theme::colors::COLOR_KEYBINDING_BORDER))
    .bg(rgb(theme::colors::COLOR_KEYBINDING_BACKGROUND))
    .text_xs()
    .text_color(rgb(theme::colors::COLOR_FOREGROUND))
}
