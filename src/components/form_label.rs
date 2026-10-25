use gpui_kit::RenderOnce;
use gpui_kit::base::StyledExt as _;
use gpui_kit::*;

#[derive(IntoElement)]
pub struct AppFormLabel {
  id: ElementId,
  style: StyleRefinement,
  children: Vec<AnyElement>,
}

impl AppFormLabel {
  pub fn new(id: impl Into<ElementId>) -> Self {
    Self {
      id: id.into(),
      children: Vec::new(),
      style: StyleRefinement::default(),
    }
  }
}

impl Styled for AppFormLabel {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for AppFormLabel {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for AppFormLabel {
  fn render(
    self,
    window: &mut Window,
    cx: &mut App,
  ) -> impl IntoElement {
    div()
      .id(self.id)
      .w_full()
      .font_weight(FontWeight::LIGHT)
      .text_sm()
      .mb(px(4.0))
      .refine_style(&self.style)
      .children(self.children)
  }
}
