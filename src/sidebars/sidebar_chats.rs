use std::future::Future;
use std::time::SystemTime;

use gpui_kit::base::h_flex;
use gpui_kit::base::v_flex;
use gpui_kit::component::Icon;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::Input;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::InputState;
use gpui_kit::component::menu::ContextMenuExt as _;
use gpui_kit::component::menu::PopupMenuItem;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::app::NewChat;
use crate::icons::ICON_NEW_FILE;
use crate::services::AppServices;
use crate::services::conversations::ConversationId;
use crate::services::conversations::ConversationSummary;
use crate::services::workspace::Workspace;
use crate::theme;

pub struct OpenChat(pub ConversationId);

struct Rename {
  id: ConversationId,
  input: Entity<InputState>,
  _subscription: Subscription,
}

pub struct SidebarChats {
  workspace: Workspace,
  chats: Vec<ConversationSummary>,
  error: Option<SharedString>,
  renaming: Option<Rename>,
  _updates: Task<()>,
}

impl EventEmitter<OpenChat> for SidebarChats {}

impl SidebarChats {
  pub fn new(
    workspace: Workspace,
    cx: &mut Context<Self>,
  ) -> Self {
    Self {
      _updates: Self::subscribe(&workspace, cx),
      workspace,
      chats: Vec::new(),
      error: None,
      renaming: None,
    }
  }

  pub fn set_workspace(
    &mut self,
    workspace: Workspace,
    cx: &mut Context<Self>,
  ) {
    *self = Self::new(workspace, cx);
    cx.notify();
  }

  fn subscribe(
    workspace: &Workspace,
    cx: &mut Context<Self>,
  ) -> Task<()> {
    let mut lists = AppServices::obtain()
      .conversations()
      .subscribe(workspace.clone());
    cx.spawn(async move |this, cx| {
      while let Some(list) = lists.recv().await {
        let updated = this.update(cx, |this, cx| {
          match list {
            Ok(chats) => {
              this.chats = chats;
              this.error = None;
            }
            Err(error) => this.error = Some(error.into()),
          }
          cx.notify();
        });
        if updated.is_err() {
          return;
        }
      }
    })
  }

  fn start_rename(
    &mut self,
    id: ConversationId,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let Some(chat) = self.chats.iter().find(|chat| chat.id == id) else {
      return;
    };

    let name = chat.name.clone();
    let input = cx.new(|cx| InputState::new(window, cx).default_value(name));
    let subscription = cx.subscribe_in(&input, window, |this, _, event: &InputEvent, _, cx| {
      if let InputEvent::PressEnter { .. } | InputEvent::Blur = event {
        this.finish_rename(true, cx);
      }
    });
    // Focusing before the menu dismisses keeps it from restoring focus to the
    // row, which would blur the input and commit the rename immediately.
    input.update(cx, |input, cx| {
      input.focus(window, cx);
      input.select_all(window, cx);
    });

    self.renaming = Some(Rename {
      id,
      input,
      _subscription: subscription,
    });
    cx.notify();
  }

  fn finish_rename(
    &mut self,
    commit: bool,
    cx: &mut Context<Self>,
  ) {
    let Some(rename) = self.renaming.take() else {
      return;
    };
    cx.notify();

    let name = rename.input.read(cx).value().trim().to_string();
    let Some(chat) = self.chats.iter_mut().find(|chat| chat.id == rename.id) else {
      return;
    };
    if !commit || name.is_empty() || chat.name == name {
      return;
    }
    chat.name = name.clone();

    let save = AppServices::obtain()
      .conversations()
      .rename(&rename.id, name);
    self.report(save, cx);
  }

  fn delete_chat(
    &mut self,
    id: ConversationId,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let delete = AppServices::obtain().conversations().delete(&id);
    self.report(delete, cx);
  }

  fn clone_chat(
    &mut self,
    id: ConversationId,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let duplicate = AppServices::obtain().conversations().duplicate(&id);
    self.report(duplicate, cx);
  }

  /// Successful changes come back through the list subscription; only failures
  /// need showing here.
  fn report(
    &mut self,
    change: impl Future<Output = anyhow::Result<()>> + 'static,
    cx: &mut Context<Self>,
  ) {
    cx.spawn(async move |this, cx| {
      let Err(error) = change.await else {
        return;
      };
      let _ = this.update(cx, |this, cx| {
        this.error = Some(format!("{error:#}").into());
        cx.notify();
      });
    })
    .detach();
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
        let rename = self.renaming.as_ref().filter(|rename| rename.id == chat.id);
        h_flex()
          .id(("sidebar-chat", ix))
          .gap(px(2.0))
          .rounded(px(6.0))
          .text_size(px(13.0))
          .text_color(rgb(theme::colors::COLOR_SIDEBAR_FOREGROUND))
          .map(|this| match rename {
            Some(rename) => this
              .px(px(4.0))
              .py(px(1.0))
              .child(rename_input(&rename.input, cx)),
            None => this
              .px(px(8.0))
              .py(px(2.0))
              .cursor_pointer()
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
              ),
          })
          .context_menu({
            let this = this.clone();
            let id = chat.id.clone();
            move |menu, _, _| {
              menu
                .item(chat_menu_item("Rename", &this, &id, Self::start_rename))
                .item(chat_menu_item("Clone", &this, &id, Self::clone_chat))
                .item(chat_menu_item("Delete", &this, &id, Self::delete_chat))
            }
          })
      }));

    v_flex()
      .size_full()
      .p(px(4.0))
      .gap(px(6.0))
      .child(header(&self.workspace))
      .child(chats)
  }
}

fn chat_menu_item(
  label: &'static str,
  this: &WeakEntity<SidebarChats>,
  id: &ConversationId,
  action: fn(&mut SidebarChats, ConversationId, &mut Window, &mut Context<SidebarChats>),
) -> PopupMenuItem {
  let this = this.clone();
  let id = id.clone();
  PopupMenuItem::new(label).on_click(move |_, window, cx| {
    let _ = this.update(cx, |this, cx| action(this, id.clone(), window, cx));
  })
}

fn rename_input(
  input: &Entity<InputState>,
  cx: &mut Context<SidebarChats>,
) -> Div {
  div()
    .flex_1()
    .min_w_0()
    .rounded(px(2.0))
    .border_1()
    .border_color(rgb(theme::colors::COLOR_FOCUS_BORDER))
    .bg(rgb(theme::colors::COLOR_INPUT_BACKGROUND))
    // The input consumes Escape itself without emitting an event.
    .capture_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
      if event.keystroke.key == "escape" {
        cx.stop_propagation();
        this.finish_rename(false, cx);
      }
    }))
    .child(
      Input::new(input)
        .appearance(false)
        .xsmall()
        .px(px(3.0))
        .py_0()
        .text_size(px(13.0))
        .text_color(rgb(theme::colors::COLOR_FOREGROUND)),
    )
}

fn header(workspace: &Workspace) -> Div {
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
    .child(div().min_w_0().truncate().child(workspace.name()))
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
