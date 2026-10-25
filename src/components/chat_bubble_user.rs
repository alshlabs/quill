use std::rc::Rc;

use gpui_kit::AnyElement;
use gpui_kit::ParentElement as _;
use gpui_kit::RenderOnce;
use gpui_kit::base::StyledExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::ClickHandler;
use crate::theme::colors::COLOR_BLUE_GREY_800;
use crate::theme::colors::COLOR_GREY_50;

const MAX_WIDTH: f32 = 768.;
const PADDING_X: f32 = 12.;
const PADDING_Y: f32 = 13.;
const RADIUS: f32 = 12.;

#[derive(IntoElement)]
pub struct AppChatBubbleUser {
  id: ElementId,
  children: Vec<AnyElement>,
  on_click: Option<ClickHandler>,
  style: StyleRefinement,
}

impl AppChatBubbleUser {
  pub fn new(id: impl Into<ElementId>) -> Self {
    Self {
      id: id.into(),
      children: Vec::new(),
      on_click: None,
      style: StyleRefinement::default(),
    }
  }

  pub fn on_click(
    mut self,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    self.on_click = Some(Rc::new(handler));
    self
  }
}

impl Styled for AppChatBubbleUser {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for AppChatBubbleUser {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for AppChatBubbleUser {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    div()
      .id(self.id)
      // .flex()
      // .min_w_0()
      // .max_w(px(MAX_WIDTH))
      // .self_end()
      // .ml_auto()
      .rounded(px(RADIUS))
      .bg(rgb(COLOR_BLUE_GREY_800))
      .px(px(PADDING_X))
      .py(px(PADDING_Y))
      .text_sm()
      .font_weight(FontWeight::LIGHT)
      .line_height(px(20.))
      .text_color(rgb(COLOR_GREY_50))
      .when_some(self.on_click, |this, on_click| {
        this
          .cursor_pointer()
          .on_click(move |event, window, cx| on_click(event, window, cx))
      })
      .refine_style(&self.style)
      .children(self.children)
  }
}
