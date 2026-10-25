use anyhow::Context as _;
use gpui_kit::base::h_flex;
use gpui_kit::base::v_flex;
use gpui_kit::component::Icon;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::Textarea;
use gpui_kit::component::input::TextareaState;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::components::AppChatBubbleUser;
use crate::components::AppMarkdown;
use crate::icons::ICON_ADD;
use crate::icons::ICON_ARROW_UP;
use crate::icons::ICON_CODE;
use crate::icons::ICON_MIC;
use crate::icons::ICON_SETTINGS;
use crate::icons::ICON_SPARKLE;
use crate::services::AppServices;
use crate::services::agent;
use crate::services::agent::AgentEvent;
use crate::services::agent::AgentHarness;
use crate::services::agent::AgentHarnessOptions;
use crate::services::agent::HistoryMessage;
use crate::services::agent::HistoryRole;
use crate::theme;

#[allow(dead_code)]
const TIP_LABEL: &str = "Tip:";
#[allow(dead_code)]
const TIP_COMMAND: &str = "/create-skill";
const CHAT_MAX_WIDTH: Pixels = px(750.0);
// The chat entry is overlaid on the message list, so the list needs room to
// scroll its last message clear of it.
const MESSAGES_BOTTOM_PADDING: Pixels = px(150.0);

#[derive(PartialEq)]
enum ChatRole {
  User,
  Agent,
  Error,
}

struct ChatMessage {
  role: ChatRole,
  message_id: Option<String>,
  content: String,
}

impl From<HistoryMessage> for ChatMessage {
  fn from(message: HistoryMessage) -> Self {
    Self {
      role: match message.role {
        HistoryRole::User => ChatRole::User,
        HistoryRole::Agent => ChatRole::Agent,
      },
      message_id: message.message_id,
      content: message.text,
    }
  }
}

pub enum ChatEvent {
  TitleChanged,
  /// The chat's session started or received a reply, so chat lists may be stale.
  Updated,
}

pub struct PanelChat {
  title: SharedString,
  session_id: Option<String>,
  input: Entity<TextareaState>,
  agent: anyhow::Result<AgentHarness>,
  messages: Vec<ChatMessage>,
  scroll_handle: ScrollHandle,
  response: Option<Task<()>>,
  _naming: Option<Task<()>>,
  _history: Option<Task<()>>,
  _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ChatEvent> for PanelChat {}

impl PanelChat {
  pub fn new(
    title: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> Self {
    let input = cx.new(|cx| {
      TextareaState::new(window, cx)
        .placeholder("Ask anything")
        .auto_grow(2, 12)
        .submit_on_enter(true)
    });

    let on_input = cx.subscribe_in(&input, window, |this, _, event: &InputEvent, window, cx| {
      if let InputEvent::PressEnter { shift: false, .. } = event {
        this.submit(window, cx);
      }
    });

    let (agent, naming) = connect_agent(None, cx);

    Self {
      title: title.into(),
      session_id: None,
      input,
      agent,
      messages: Vec::new(),
      scroll_handle: ScrollHandle::new(),
      response: None,
      _naming: naming,
      _history: None,
      _subscriptions: vec![on_input],
    }
  }

  pub fn title(&self) -> SharedString {
    self.title.clone()
  }

  pub fn session_id(&self) -> Option<&str> {
    self.session_id.as_deref()
  }

  /// Replaces this chat with a stored session: its history is shown and new
  /// prompts continue that session.
  pub fn load_session(
    &mut self,
    session_id: String,
    cx: &mut Context<Self>,
  ) {
    if self.session_id.as_deref() == Some(session_id.as_str()) {
      return;
    }

    self.response = None;
    self.messages.clear();
    self.session_id = Some(session_id.clone());
    (self.agent, self._naming) = connect_agent(Some(session_id.clone()), cx);

    let history = agent::load_history(session_id);
    self._history = Some(cx.spawn(async move |this, cx| {
      let history = history.await;
      let _ = this.update(cx, |this, cx| {
        match history {
          // Prepended, so prompts sent while the history loaded stay after it.
          Ok(history) => {
            this
              .messages
              .splice(0..0, history.messages.into_iter().map(ChatMessage::from));
            this.set_title(history.name, cx);
          }
          Err(error) => this.append(AgentEvent::Error(format!("{error:#}")), cx),
        }
        this.scroll_handle.scroll_to_bottom();
        cx.notify();
      });
    }));

    cx.notify();
  }

  fn set_title(
    &mut self,
    title: String,
    cx: &mut Context<Self>,
  ) {
    let title = title.trim();
    if title.is_empty() || self.title == title {
      return;
    }
    self.title = SharedString::from(title.to_string());
    cx.emit(ChatEvent::TitleChanged);
    cx.notify();
  }

  fn submit(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let text = self.input.read(cx).value().trim().to_string();
    if text.is_empty() || self.response.is_some() {
      return;
    }

    self
      .input
      .update(cx, |input, cx| input.set_value("", window, cx));

    let events = match &self.agent {
      Ok(agent) => Ok(agent.prompt(&text)),
      Err(error) => Err(format!("{error:#}")),
    };
    self.messages.push(ChatMessage {
      role: ChatRole::User,
      message_id: None,
      content: text,
    });
    self.scroll_handle.scroll_to_bottom();

    let mut events = match events {
      Ok(events) => events,
      Err(error) => {
        self.append(AgentEvent::Error(error), cx);
        cx.notify();
        return;
      }
    };

    self.response = Some(cx.spawn(async move |this, cx| {
      while let Some(event) = events.recv().await {
        let appended = this.update(cx, |this, cx| {
          this.append(event, cx);
          this.scroll_handle.scroll_to_bottom();
          cx.notify();
        });
        if appended.is_err() {
          return;
        }
      }
      let _ = this.update(cx, |this, cx| {
        this.response = None;
        cx.emit(ChatEvent::Updated);
        cx.notify();
      });
    }));

    cx.notify();
  }

  fn append(
    &mut self,
    event: AgentEvent,
    cx: &mut Context<Self>,
  ) {
    match event {
      AgentEvent::Session { id } => {
        self.session_id = Some(id);
        cx.emit(ChatEvent::Updated);
      }
      AgentEvent::Text { message_id, delta } => match self.messages.last_mut() {
        Some(last) if last.role == ChatRole::Agent && last.message_id == message_id => {
          last.content.push_str(&delta)
        }
        _ => self.messages.push(ChatMessage {
          role: ChatRole::Agent,
          message_id,
          content: delta,
        }),
      },
      AgentEvent::Error(error) => self.messages.push(ChatMessage {
        role: ChatRole::Error,
        message_id: None,
        content: error,
      }),
    }
  }
}

fn connect_agent(
  session_id: Option<String>,
  cx: &mut Context<PanelChat>,
) -> (anyhow::Result<AgentHarness>, Option<Task<()>>) {
  let mut agent = AppServices::obtain()
    .config
    .models
    .first()
    .context("No models configured")
    .map(|model| {
      AgentHarness::new(AgentHarnessOptions {
        session_id,
        ..model.into()
      })
    });

  let naming = agent
    .as_mut()
    .ok()
    .and_then(AgentHarness::take_names)
    .map(|mut names| {
      cx.spawn(async move |this, cx| {
        while let Some(name) = names.recv().await {
          let renamed = this.update(cx, |this, cx| this.set_title(name, cx));
          if renamed.is_err() {
            return;
          }
        }
      })
    });

  (agent, naming)
}

impl Render for PanelChat {
  fn render(
    &mut self,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let has_messages = !self.messages.is_empty();
    let on_send = cx.listener(|this, _: &ClickEvent, window, cx| this.submit(window, cx));

    div()
      .size_full()
      .p(px(8.0))
      .relative()
      .child(
        div()
          .id("chat-messages")
          .size_full()
          .overflow_y_scroll()
          .track_scroll(&self.scroll_handle)
          .child(
            v_flex()
              .w_full()
              .max_w(CHAT_MAX_WIDTH)
              .mx_auto()
              .px(px(16.0))
              .pt(px(16.0))
              .pb(MESSAGES_BOTTOM_PADDING)
              .gap(px(16.0))
              .children(self.messages.iter().enumerate().map(chat_message)),
          ),
      )
      .child(
        div()
          .absolute()
          .inset_0()
          .flex()
          .justify_center()
          .px(px(16.0))
          .map(|this| {
            if has_messages {
              this.items_end().pb(px(16.0))
            } else {
              this.items_center()
            }
          })
          .child(chat_entry(&self.input, on_send)),
      )
  }
}

fn chat_message((ix, message): (usize, &ChatMessage)) -> AnyElement {
  let id = ("chat-message", ix);
  match message.role {
    ChatRole::User => AppChatBubbleUser::new(id)
      .self_end()
      .max_w(px(600.0))
      .child(message.content.clone())
      .into_any_element(),
    ChatRole::Agent => AppMarkdown::new(id, message.content.clone())
      .text_sm()
      .into_any_element(),
    ChatRole::Error => div()
      .id(id)
      .text_sm()
      .text_color(rgb(theme::colors::COLOR_ERROR_FOREGROUND))
      .child(message.content.clone())
      .into_any_element(),
  }
}

fn chat_entry(
  input: &Entity<TextareaState>,
  on_send: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
  v_flex()
    // .bg(rgb(theme::colors::COLOR_INPUT_BACKGROUND))
    .border_1()
    .border_color(rgb(theme::colors::COLOR_INPUT_BORDER))
    .w_full()
    .max_w(CHAT_MAX_WIDTH)
    .rounded(px(10.0))
    // .child(tip(
    //   TIP_COMMAND,
    //   "to create a skill the agent can load when relevant.",
    // ))
    .child(
      div().px(px(12.0)).pt(px(12.0)).pb(px(4.0)).child(
        Textarea::new(input)
          .appearance(false)
          .text_color(rgb(theme::colors::COLOR_FOREGROUND)),
      ),
    )
    .child(toolbar(on_send))
}

#[allow(dead_code)]
fn tip(
  command: &str,
  description: &str,
) -> Div {
  let text = format!("{TIP_LABEL} Use {command} {description}");
  let command_start = text.find(command).unwrap_or(0);

  div()
    .px(px(12.0))
    .py(px(10.0))
    .border_b_1()
    .border_color(rgb(theme::colors::COLOR_EDITOR_BORDER))
    .text_sm()
    .text_color(rgb(theme::colors::COLOR_DESCRIPTION_FOREGROUND))
    .child(StyledText::new(text).with_highlights([
      (
        0..TIP_LABEL.len(),
        HighlightStyle {
          color: Some(rgb(theme::colors::COLOR_FOREGROUND).into()),
          font_weight: Some(FontWeight::BOLD),
          ..Default::default()
        },
      ),
      (
        command_start..command_start + command.len(),
        HighlightStyle {
          color: Some(rgb(theme::colors::COLOR_LINK_FOREGROUND).into()),
          ..Default::default()
        },
      ),
    ]))
}

fn toolbar(on_send: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Div {
  h_flex()
    .justify_between()
    .items_center()
    .px(px(6.0))
    .pb(px(6.0))
    .child(
      h_flex()
        .items_center()
        .gap(px(2.0))
        .child(toolbar_button(ICON_ADD, None))
        .child(toolbar_button(ICON_CODE, Some("Agent")))
        .child(toolbar_button(ICON_SPARKLE, Some("DeepSeek Flash")))
        .child(
          div()
            .w(px(1.0))
            .h(px(14.0))
            .bg(rgb(theme::colors::COLOR_INPUT_BORDER)),
        )
        // .child(
        //   div()
        //     .px(px(6.0))
        //     .text_sm()
        //     .text_color(rgb(theme::colors::COLOR_DESCRIPTION_FOREGROUND))
        //     .child("786K"),
        // )
        .child(toolbar_button(ICON_SETTINGS, None)),
    )
    .child(
      h_flex()
        .items_center()
        .gap(px(2.0))
        .child(toolbar_button(ICON_MIC, None))
        .child(
          toolbar_button(ICON_ARROW_UP, None)
            .id("chat-send")
            .text_color(rgba(theme::colors::COLOR_DISABLED_FOREGROUND_A))
            .on_click(on_send),
        ),
    )
}

fn toolbar_button(
  icon: &'static [u8],
  label: Option<&'static str>,
) -> Div {
  h_flex()
    .items_center()
    .gap(px(6.0))
    .h(px(28.0))
    .px(px(6.0))
    .rounded(px(6.0))
    .cursor_pointer()
    .text_xs()
    .text_color(rgb(theme::colors::COLOR_DESCRIPTION_FOREGROUND))
    .hover(|this| this.bg(rgba(theme::colors::COLOR_TOOLBAR_HOVER_BACKGROUND_A)))
    .child(Icon::empty().data(icon).h(px(14.0)).w(px(14.0)))
    .children(label)
}
