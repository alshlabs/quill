use std::rc::Rc;

use gpui_kit::AnyElement;
use gpui_kit::ClickEvent;
use gpui_kit::ParentElement as _;
use gpui_kit::RenderOnce;
use gpui_kit::base::StyledExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::ClickHandler;
use crate::theme;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum AppButtonColor {
  #[default]
  Primary,
  Secondary,
  Error,
  Warn,
  Transparent,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum AppButtonVariant {
  #[default]
  Solid,
  Outline,
  Ghost,
}

struct AppButtonColors {
  foreground: Rgba,
  background: Rgba,
  border: Option<Rgba>,
}
impl AppButtonColor {
  fn colors(
    self,
    variant: AppButtonVariant,
  ) -> AppButtonColors {
    let none = rgba(0x00000000);
    let grey_50 = rgb(theme::COLOR_GREY_50);
    let grey_700 = rgb(theme::COLOR_GREY_700);
    let grey_900 = rgb(theme::COLOR_GREY_900);
    let red_600 = rgb(theme::COLOR_RED_600);
    let deep_orange_400 = rgb(theme::COLOR_DEEP_ORANGE_400);

    let (background, foreground, border) = match (variant, self) {
      (AppButtonVariant::Solid, Self::Primary) => (grey_50, grey_900, Some(grey_50)),
      (AppButtonVariant::Solid, Self::Secondary) => (grey_900, grey_50, None),
      (AppButtonVariant::Solid, Self::Error) => (red_600, grey_50, None),
      (AppButtonVariant::Solid, Self::Warn) => (deep_orange_400, grey_50, None),
      (AppButtonVariant::Solid, Self::Transparent) => (none, grey_50, None),

      (AppButtonVariant::Outline, Self::Primary) => (none, grey_50, Some(grey_50)),
      (AppButtonVariant::Outline, Self::Secondary) => (none, grey_50, Some(grey_700)),
      (AppButtonVariant::Outline, Self::Error) => (none, grey_50, Some(red_600)),
      (AppButtonVariant::Outline, Self::Warn) => (none, grey_50, Some(deep_orange_400)),
      (AppButtonVariant::Outline, Self::Transparent) => (none, grey_50, None),

      (AppButtonVariant::Ghost, Self::Primary) => (none, grey_50, None),
      (AppButtonVariant::Ghost, Self::Secondary) => (none, grey_50, None),
      (AppButtonVariant::Ghost, Self::Error) => (none, grey_50, None),
      (AppButtonVariant::Ghost, Self::Warn) => (none, grey_50, None),
      (AppButtonVariant::Ghost, Self::Transparent) => (none, grey_50, None),
    };

    AppButtonColors {
      foreground,
      background,
      border,
    }
  }
}

#[derive(IntoElement)]
pub struct AppButton {
  id: ElementId,
  color: AppButtonColor,
  variant: AppButtonVariant,
  children: Vec<AnyElement>,
  on_click: Option<ClickHandler>,
  style: StyleRefinement,
}

impl AppButton {
  pub fn new(id: impl Into<ElementId>) -> Self {
    Self {
      id: id.into(),
      color: AppButtonColor::default(),
      variant: AppButtonVariant::default(),
      children: Vec::new(),
      on_click: None,
      style: StyleRefinement::default(),
    }
  }

  pub fn color(
    mut self,
    color: AppButtonColor,
  ) -> Self {
    self.color = color;
    self
  }

  pub fn variant(
    mut self,
    variant: AppButtonVariant,
  ) -> Self {
    self.variant = variant;
    self
  }

  pub fn on_click(
    mut self,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    self.on_click = Some(Rc::new(handler));
    self
  }
}

impl Styled for AppButton {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}

impl ParentElement for AppButton {
  fn extend(
    &mut self,
    elements: impl IntoIterator<Item = AnyElement>,
  ) {
    self.children.extend(elements)
  }
}

impl RenderOnce for AppButton {
  fn render(
    self,
    _window: &mut Window,
    _cx: &mut App,
  ) -> impl IntoElement {
    let colors = self.color.colors(self.variant);

    div()
      .id(self.id)
      // .with_size(Size::Large)
      .cursor_pointer()
      .rounded(px(10.))
      .cursor_pointer()
      .pt(px(4.0))
      .pb(px(4.0))
      .pl(px(20.0))
      .pr(px(20.0))
      .h_auto()
      .w_auto()
      .bg(colors.background)
      .when(colors.border.is_some(), |this| {
        if let Some(color) = colors.border {
          this.border_1().border_color(color)
        } else {
          this
        }
      })
      .text_color(colors.foreground)
      .font_weight(FontWeight::LIGHT)
      .text_sm()
      .refine_style(&self.style)
      .children(self.children)
      .hover(|this| this.opacity(0.7))
      .when_some(self.on_click, |this, on_click| {
        this.on_click(move |event, window, cx| on_click(event, window, cx))
      })
  }
}
