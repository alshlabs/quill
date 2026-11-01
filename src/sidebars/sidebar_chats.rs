use std::time::SystemTime;

use gpui_kit::base::h_flex;
use gpui_kit::base::v_flex;
use gpui_kit::component::Icon;
use gpui_kit::component::menu::ContextMenuExt as _;
use gpui_kit::component::menu::PopupMenuItem;
use gpui_kit::*;

use crate::app::NewChat;
use crate::icons::ICON_NEW_FILE;
use crate::services::agent;
use crate::services::agent::ChatSummary;
use crate::theme;

pub struct OpenChat(pub String);

pub struct ChatDeleted(pub String);

pub struct SidebarChats {
  chats: Vec<ChatSummary>,
  error: Option<SharedString>,
  _refresh: Option<Task<()>>,
}

impl EventEmitter<OpenChat> for SidebarChats {}

impl EventEmitter<ChatDeleted> for SidebarChats {}

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

  fn delete_chat(
    &mut self,
    session_id: String,
    cx: &mut Context<Self>,
  ) {
    let delete = agent::delete_chat(session_id.clone());
    cx.spawn(async move |this, cx| {
      let result = delete.await;
      let _ = this.update(cx, |this, cx| {
        if result.is_ok() {
          cx.emit(ChatDeleted(session_id));
        }
        this.finish_change(result, cx);
      });
    })
    .detach();
  }

  fn clone_chat(
    &mut self,
    session_id: String,
    cx: &mut Context<Self>,
  ) {
    let clone = agent::clone_chat(session_id);
    cx.spawn(async move |this, cx| {
      let result = clone.await;
      let _ = this.update(cx, |this, cx| this.finish_change(result, cx));
    })
    .detach();
  }

  fn finish_change(
    &mut self,
    result: anyhow::Result<()>,
    cx: &mut Context<Self>,
  ) {
    match result {
      Ok(()) => self.refresh(cx),
      Err(error) => {
        self.error = Some(format!("{error:#}").into());
        cx.notify();
      }
    }
  }
}

impl Render for SidebarChats {
  fn render(
    &mut self,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let now = SystemTime::now();
    let this = cx.entity().downgrade();

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
          .text_color(rgb(theme::colors::COLOR_ERROR_FOREGROUND))
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
          .text_size(px(13.0))
          .text_color(rgb(theme::colors::COLOR_SIDEBAR_FOREGROUND))
          .hover(|this| {
            this
              .bg(rgb(theme::colors::COLOR_LIST_HOVER_BACKGROUND))
              .text_color(rgb(theme::colors::COLOR_WHITE))
          })
          .on_click(cx.listener(move |_, _: &ClickEvent, _, cx| {
            cx.emit(OpenChat(id.clone()));
          }))
          .child(div().flex_1().min_w_0().truncate().child(chat.name.clone()))
          .child(
            div()
              .flex_shrink_0()
              .text_xs()
              .text_color(rgb(theme::colors::COLOR_DESCRIPTION_FOREGROUND))
              .child(relative_time(now, chat.updated_at)),
          )
          .context_menu({
            let this = this.clone();
            let id = chat.id.clone();
            move |menu, _, _| {
              menu
                .item(chat_menu_item("Clone", &this, &id, Self::clone_chat))
                .item(chat_menu_item("Delete", &this, &id, Self::delete_chat))
            }
          })
      }));

    v_flex()
      .size_full()
      .p(px(4.0))
      .gap(px(6.0))
      .child(header())
      .child(chats)
  }
}

fn chat_menu_item(
  label: &'static str,
  this: &WeakEntity<SidebarChats>,
  session_id: &str,
  action: fn(&mut SidebarChats, String, &mut Context<SidebarChats>),
) -> PopupMenuItem {
  let this = this.clone();
  let session_id = session_id.to_string();
  PopupMenuItem::new(label).on_click(move |_, _, cx| {
    let _ = this.update(cx, |this, cx| action(this, session_id.clone(), cx));
  })
}

fn header() -> Div {
  h_flex()
    .flex_shrink_0()
    .justify_between()
    .items_center()
    .h(px(22.0))
    .pl(px(8.0))
    .pr(px(2.0))
    .text_size(px(11.0))
    .font_weight(FontWeight::BOLD)
    .text_color(rgb(theme::colors::COLOR_SIDEBAR_FOREGROUND))
    .child(div().min_w_0().truncate().child("Global workspace"))
    .child(header_new_chat())
}

fn header_new_chat() -> Stateful<Div> {
  h_flex()
    .id("sidebar-chats-new-chat")
    .flex_shrink_0()
    .justify_center()
    .items_center()
    .w(px(22.0))
    .h(px(22.0))
    .rounded(px(4.0))
    .cursor_pointer()
    .text_color(rgb(theme::colors::COLOR_ICON_FOREGROUND))
    .hover(|this| {
      this
        .bg(rgba(theme::colors::COLOR_TOOLBAR_HOVER_BACKGROUND_A))
        .text_color(rgb(theme::colors::COLOR_FOREGROUND))
    })
    .on_click(|_, window, cx| window.dispatch_action(NewChat.boxed_clone(), cx))
    .child(Icon::empty().data(ICON_NEW_FILE).h(px(16.0)).w(px(16.0)))
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
