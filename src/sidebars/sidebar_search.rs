use gpui_kit::AnyElement;
use gpui_kit::RenderOnce;
use gpui_kit::*;

#[derive(IntoElement)]
pub struct SidebarSearch {
  children: Vec<AnyElement>,
  style: StyleRefinement,
}

impl SidebarSearch {
  pub fn new() -> Self {
    Self {
      children: Vec::new(),
      style: StyleRefinement::default(),
    }
  }
}

impl Styled for SidebarSearch {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for SidebarSearch {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for SidebarSearch {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    div().id(format!("sidebar_search")).child("search")
  }
}
