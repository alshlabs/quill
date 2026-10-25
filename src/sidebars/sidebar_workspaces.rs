use gpui_kit::AnyElement;
use gpui_kit::RenderOnce;
use gpui_kit::*;

#[derive(IntoElement)]
pub struct SidebarWorkspaces {
  children: Vec<AnyElement>,
  style: StyleRefinement,
}

impl SidebarWorkspaces {
  pub fn new() -> Self {
    Self {
      children: Vec::new(),
      style: StyleRefinement::default(),
    }
  }
}

impl Styled for SidebarWorkspaces {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for SidebarWorkspaces {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for SidebarWorkspaces {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    div()
      .id("sidebar_workspaces".to_string())
      .child("workspaces")
  }
}
