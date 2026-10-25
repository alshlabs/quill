use gpui_kit::Entity;
use gpui_kit::RenderOnce;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::input::Input;
use gpui_kit::component::input::InputState;
use gpui_kit::*;

use crate::theme;

const RADIUS: f32 = 10.;
const PADDING_X: f32 = 10.;
const PADDING_Y: f32 = 6.;

#[derive(IntoElement)]
pub struct AppInput {
  id: ElementId,
  state: Entity<InputState>,
  placeholder: Option<SharedString>,
  style: StyleRefinement,
}

impl AppInput {
  pub fn new(
    id: impl Into<ElementId>,
    state: Entity<InputState>,
  ) -> Self {
    Self {
      id: id.into(),
      state,
      placeholder: None,
      style: StyleRefinement::default(),
    }
  }

  pub fn placeholder(
    mut self,
    placeholder: impl Into<SharedString>,
  ) -> Self {
    self.placeholder = Some(placeholder.into());
    self
  }
}

impl Styled for AppInput {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl RenderOnce for AppInput {
  fn render(
    self,
    window: &mut Window,
    cx: &mut App,
  ) -> impl IntoElement {
    if let Some(placeholder) = self.placeholder {
      let current = self.state.read(cx).presentation().placeholder().clone();

      if current != placeholder {
        self.state.update(cx, |state, cx| {
          state.set_placeholder(placeholder, window, cx)
        });
      }
    }

    Input::new(&self.state)
      .id(self.id)
      .w_full()
      .rounded(px(RADIUS))
      .border_color(rgb(theme::COLOR_GREY_700))
      .mb(px(8.0))
      .border_1()
      .bg(rgb(theme::COLOR_GREY_900))
      .px(px(PADDING_X))
      .py(px(PADDING_Y))
      .refine_style(&self.style)
  }
}
