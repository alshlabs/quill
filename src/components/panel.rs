use gpui_kit::RenderOnce;
use gpui_kit::base::StyledExt;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme;

#[derive(IntoElement)]
pub struct AppPanel {
  id: ElementId,
  style: StyleRefinement,
  children: Vec<AnyElement>,
  bg: Option<Rgba>,
  outline: Option<Rgba>,
}

impl AppPanel {
  pub fn new(id: impl Into<ElementId>) -> Self {
    Self {
      id: id.into(),
      style: StyleRefinement::default(),
      children: Vec::new(),
      bg: Some(rgb(theme::COLOR_GREY_900)),
      outline: None,
    }
  }

  pub fn light(mut self) -> Self {
    self.bg = Some(rgb(theme::COLOR_GREY_800));
    self
  }

  pub fn x_light(mut self) -> Self {
    self.bg = Some(rgb(theme::COLOR_GREY_700));
    self
  }

  pub fn outline(mut self) -> Self {
    self.bg = None;
    self.outline = Some(rgb(theme::COLOR_GREY_700));
    self
  }
}

impl Styled for AppPanel {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for AppPanel {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for AppPanel {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    div()
      .id(self.id)
      .p(px(12.0))
      .rounded(px(12.0))
      .when(self.bg.is_some(), |this| {
        if let Some(bg) = self.bg {
          this.bg(bg)
        } else {
          this
        }
      })
      .when(self.outline.is_some(), |this| {
        if let Some(color) = self.outline {
          this.border_1().border_color(color)
        } else {
          this
        }
      })
      .refine_style(&self.style)
      // .bg(self.bg)
      .children(self.children)
  }
}
