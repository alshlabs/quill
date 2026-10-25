use gpui_kit::RenderOnce;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::text::TextView;
use gpui_kit::component::text::TextViewStyle;
use gpui_kit::*;

#[derive(IntoElement)]
pub struct AppMarkdown {
  id: ElementId,
  text: SharedString,
  style: StyleRefinement,
}

impl AppMarkdown {
  pub fn new(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
  ) -> Self {
    Self {
      id: id.into(),
      text: text.into(),
      style: StyleRefinement::default(),
    }
  }
}

impl Styled for AppMarkdown {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl RenderOnce for AppMarkdown {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    TextView::markdown(self.id, self.text)
      .style(TextViewStyle::default().paragraph_gap(rems(0.5)))
      .selectable(true)
      .refine_style(&self.style)
  }
}
