use std::time::SystemTime;

use gpui_kit::base::h_flex;
use gpui_kit::base::v_flex;
use gpui_kit::*;

use crate::app::NewChat;
use crate::services::agent;
use crate::services::agent::ChatSummary;
use crate::theme;

pub struct OpenChat(pub String);

pub struct SidebarChats {
  chats: Vec<ChatSummary>,
  error: Option<SharedString>,
  _refresh: Option<Task<()>>,
}

impl EventEmitter<OpenChat> for SidebarChats {}

impl SidebarChats {
  pub fn new(cx: &mut Context<Self>) -> Self {
    let mut this = Self {
      chats: Vec::new(),
      error: None,
      _refresh: None,
    };
    this.refresh(cx);
    this
  }

  pub fn refresh(
    &mut self,
    cx: &mut Context<Self>,
  ) {
    let chats = agent::list_chats();
    self._refresh = Some(cx.spawn(async move |this, cx| {
      let chats = chats.await;
      let _ = this.update(cx, |this, cx| {
        match chats {
          Ok(chats) => {
            this.chats = chats;
            this.error = None;
          }
          Err(error) => this.error = Some(format!("{error:#}").into()),
        }
        cx.notify();
      });
    }));
  }
}

impl Render for SidebarChats {
  fn render(
    &mut self,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let now = SystemTime::now();

    let chats = v_flex()
      .id("sidebar_chats")
      .flex_1()
      .min_h_0()
      .overflow_y_scroll()
      .gap(px(2.0))
      .children(self.error.clone().map(|error| {
        div()
          .p(px(6.0))
          .text_xs()
          .text_color(rgb(theme::colors::COLOR_RED_300))
          .child(error)
      }))
      .children(self.chats.iter().enumerate().map(|(ix, chat)| {
        let id = chat.id.clone();
        h_flex()
          .id(("sidebar-chat", ix))
          .gap(px(2.0))
          .px(px(8.0))
          .py(px(2.0))
          .rounded(px(6.0))
          .cursor_pointer()
          .text_sm()
          .text_color(rgb(theme::colors::COLOR_GREY_400))
          .hover(|this| {
            this
              .bg(rgb(theme::colors::COLOR_GREY_875))
              .text_color(rgb(theme::colors::COLOR_WHITE))
          })
          .on_click(cx.listener(move |_, _: &ClickEvent, _, cx| {
            cx.emit(OpenChat(id.clone()));
          }))
          .child(
            div()
              .flex_1()
              .min_w_0()
              .truncate()
              .text_sm()
              .child(chat.name.clone()),
          )
          .child(
            div()
              .flex_shrink_0()
              .text_xs()
              .text_color(rgb(theme::colors::COLOR_GREY_600))
              .child(relative_time(now, chat.updated_at)),
          )
      }));

    v_flex()
      .size_full()
      .p(px(4.0))
      .gap(px(6.0))
      .child(new_chat_button())
      .child(chats)
  }
}

fn new_chat_button() -> Stateful<Div> {
  h_flex()
    .id("sidebar-new-chat")
    .flex_shrink_0()
    .ml(px(4.0))
    .mr(px(4.0))
    .mt(px(4.0))
    .h(px(28.0))
    .justify_center()
    .rounded(px(6.0))
    .cursor_pointer()
    .bg(rgb(theme::colors::COLOR_WHITE))
    .text_sm()
    .font_weight(FontWeight::MEDIUM)
    .text_color(rgb(theme::colors::COLOR_GREY_900))
    .hover(|this| this.bg(rgb(theme::colors::COLOR_GREY_200)))
    .on_click(|_, window, cx| window.dispatch_action(NewChat.boxed_clone(), cx))
    .child("New Chat")
}

fn relative_time(
  now: SystemTime,
  then: SystemTime,
) -> String {
  let seconds = now.duration_since(then).unwrap_or_default().as_secs();
  match seconds {
    0..60 => "now".to_string(),
    60..3_600 => format!("{}m", seconds / 60),
    3_600..86_400 => format!("{}h", seconds / 3_600),
    86_400..604_800 => format!("{}d", seconds / 86_400),
    _ => format!("{}w", seconds / 604_800),
  }
}
