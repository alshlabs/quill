#![allow(dead_code)]

mod button;
mod chat_bubble_user;
mod form_label;
mod input;
mod markdown;
mod panel;

pub use chat_bubble_user::AppChatBubbleUser;
pub use markdown::AppMarkdown;

type ClickHandler =
  std::rc::Rc<dyn Fn(&gpui_kit::ClickEvent, &mut gpui_kit::Window, &mut gpui_kit::App)>;
