use gpui_kit::AnyElement;
use gpui_kit::RenderOnce;
use gpui_kit::*;

#[derive(IntoElement)]
pub struct SidebarPlugins {
  children: Vec<AnyElement>,
  style: StyleRefinement,
}

impl SidebarPlugins {
  pub fn new() -> Self {
    Self {
      children: Vec::new(),
      style: StyleRefinement::default(),
    }
  }
}

impl Styled for SidebarPlugins {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for SidebarPlugins {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for SidebarPlugins {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    div().id(format!("sidebar_plugins")).child("plugins")
  }
}
