use gpui_kit::base::h_flex;
use gpui_kit::base::v_flex;
use gpui_kit::component::Icon;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::button::ButtonCustomVariant;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::Textarea;
use gpui_kit::component::input::TextareaState;
use gpui_kit::component::menu::DropdownMenu as _;
use gpui_kit::component::menu::PopupMenuItem;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::app::OpenSettings;
use crate::components::AppChatBubbleUser;
use crate::components::AppMarkdown;
use crate::icons::ICON_ARROW_UP;
use crate::icons::ICON_SPARKLE;
use crate::services::AppServices;
use crate::services::conversations::Conversation;
use crate::services::conversations::ConversationCommand;
use crate::services::conversations::ConversationId;
use crate::services::conversations::ConversationMessage;
use crate::services::conversations::ConversationSnapshot;
use crate::services::conversations::ConversationStatus;
use crate::services::conversations::ConversationUpdate;
use crate::services::conversations::MessageRole;
use crate::theme;

#[allow(dead_code)]
const TIP_LABEL: &str = "Tip:";
#[allow(dead_code)]
const TIP_COMMAND: &str = "/create-skill";
const CHAT_MAX_WIDTH: Pixels = px(750.0);
const MODEL_MENU_MIN_WIDTH: Pixels = px(200.0);
// The chat entry is overlaid on the message list, so the list needs room to
// scroll its last message clear of it.
const MESSAGES_BOTTOM_PADDING: Pixels = px(150.0);

pub enum ChatEvent {
  TitleChanged,
  Deleted,
}

pub struct PanelChat {
  conversation: Conversation,
  state: ConversationSnapshot,
  input: Entity<TextareaState>,
  scroll_handle: ScrollHandle,
  _updates: Task<()>,
  _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ChatEvent> for PanelChat {}

impl PanelChat {
  pub fn new(
    conversation: Conversation,
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

    let (state, mut updates) = conversation.subscribe();
    let updates = cx.spawn(async move |this, cx| {
      while let Some(update) = updates.recv().await {
        if this.update(cx, |this, cx| this.apply(update, cx)).is_err() {
          return;
        }
      }
    });

    Self {
      conversation,
      state,
      input,
      scroll_handle: ScrollHandle::new(),
      _updates: updates,
      _subscriptions: vec![on_input],
    }
  }

  pub fn title(&self) -> SharedString {
    self
      .state
      .title
      .clone()
      .map_or(SharedString::new_static("New Chat"), SharedString::from)
  }

  pub fn conversation_id(&self) -> Option<&ConversationId> {
    self.state.id.as_ref()
  }

  pub fn focus_input(
    &self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    self.input.update(cx, |input, cx| input.focus(window, cx));
  }

  fn apply(
    &mut self,
    update: ConversationUpdate,
    cx: &mut Context<Self>,
  ) {
    self.state.apply(&update);
    match update {
      ConversationUpdate::Title(_) => cx.emit(ChatEvent::TitleChanged),
      ConversationUpdate::Deleted => cx.emit(ChatEvent::Deleted),
      ConversationUpdate::History(_)
      | ConversationUpdate::Message(_)
      | ConversationUpdate::TextDelta { .. } => self.scroll_handle.scroll_to_bottom(),
      _ => {}
    }
    cx.notify();
  }

  fn submit(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let text = self.input.read(cx).value().trim().to_string();
    if text.is_empty() || self.state.status != ConversationStatus::Idle {
      return;
    }

    if self
      .conversation
      .send(ConversationCommand::Prompt(text))
      .is_ok()
    {
      self
        .input
        .update(cx, |input, cx| input.set_value("", window, cx));
    }
  }

  fn select_model(
    &mut self,
    name: SharedString,
  ) {
    let _ = self
      .conversation
      .send(ConversationCommand::SetModel(name.to_string()));
  }

  fn model_picker(
    &self,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let config = AppServices::obtain().config();
    let models = config
      .as_ref()
      .as_ref()
      .map(|config| {
        config
          .models
          .iter()
          .map(|model| SharedString::from(model.name.clone()))
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();
    let selected = config
      .as_ref()
      .as_ref()
      .ok()
      .and_then(|config| config.resolve_model(self.state.model.as_deref()))
      .map(|model| SharedString::from(model.name.clone()));
    let label = selected
      .clone()
      .unwrap_or(SharedString::new_static("No Model Configured"));
    let this = cx.entity().downgrade();

    Button::new("chat-model")
      .xsmall()
      .custom(
        ButtonCustomVariant::new(cx)
          .foreground(rgb(theme::colors::COLOR_DESCRIPTION_FOREGROUND).into())
          .hover(rgba(theme::colors::COLOR_TOOLBAR_HOVER_BACKGROUND_A).into())
          .active(rgba(theme::colors::COLOR_TOOLBAR_HOVER_BACKGROUND_A).into()),
      )
      .rounded(px(6.0))
      .h(px(28.0))
      .px(px(6.0))
      .cursor_pointer()
      .child(Icon::empty().data(ICON_SPARKLE).h(px(14.0)).w(px(14.0)))
      .child(label)
      .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
        models
          .iter()
          .fold(menu.min_w(MODEL_MENU_MIN_WIDTH), |menu, name| {
            let this = this.clone();
            let item_name = name.clone();
            menu.item(
              PopupMenuItem::new(name.clone())
                .checked(selected.as_ref() == Some(name))
                .on_click(move |_, _, cx| {
                  let _ = this.update(cx, |this, _| this.select_model(item_name.clone()));
                }),
            )
          })
          .separator()
          .menu("Settings", Box::new(OpenSettings))
      })
  }
}

impl Render for PanelChat {
  fn render(
    &mut self,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    // Markdown is parsed in the background, so messages keep growing after
    // the frame that consumed a one-shot scroll_to_bottom. Re-pin every frame
    // while the view was resting at the bottom.
    if self.scroll_handle.offset().y <= -self.scroll_handle.max_offset().y + px(1.0) {
      self.scroll_handle.scroll_to_bottom();
    }

    let has_messages = !self.state.messages.is_empty();
    let on_send = cx.listener(|this, _: &ClickEvent, window, cx| this.submit(window, cx));
    let model_picker = self.model_picker(cx);

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
              .children(self.state.messages.iter().enumerate().map(chat_message)),
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
          .child(chat_entry(&self.input, model_picker, on_send)),
      )
  }
}

fn chat_message((ix, message): (usize, &ConversationMessage)) -> AnyElement {
  let id = ("chat-message", ix);
  match message.role {
    MessageRole::User => AppChatBubbleUser::new(id)
      .self_end()
      .max_w(px(600.0))
      .child(message.text.clone())
      .into_any_element(),
    MessageRole::Agent => AppMarkdown::new(id, message.text.clone())
      .text_sm()
      .into_any_element(),
    MessageRole::Error => div()
      .id(id)
      .text_sm()
      .text_color(rgb(theme::colors::COLOR_ERROR_FOREGROUND))
      .child(message.text.clone())
      .into_any_element(),
  }
}

fn chat_entry(
  input: &Entity<TextareaState>,
  model_picker: impl IntoElement,
  on_send: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
  v_flex()
    // .bg(rgb(theme::colors::COLOR_INPUT_BACKGROUND))
    .bg(rgb(theme::colors::COLOR_EDITOR_BACKGROUND))
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
    .child(toolbar(model_picker, on_send))
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

fn toolbar(
  model_picker: impl IntoElement,
  on_send: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
  h_flex()
    .justify_between()
    .items_center()
    .px(px(6.0))
    .pb(px(6.0))
    .child(
      h_flex()
        .items_center()
        .gap(px(2.0))
        // .child(toolbar_button(ICON_ADD, None))
        // .child(toolbar_button(ICON_CODE, Some("Agent")))
        .child(model_picker), // .child(
                              //   div()
                              //     .w(px(1.0))
                              //     .h(px(14.0))
                              //     .bg(rgb(theme::colors::COLOR_INPUT_BORDER)),
                              // )
                              // .child(
                              //   div()
                              //     .px(px(6.0))
                              //     .text_sm()
                              //     .text_color(rgb(theme::colors::COLOR_DESCRIPTION_FOREGROUND))
                              //     .child("786K"),
                              // )
                              // .child(toolbar_button(ICON_SETTINGS, None)),
    )
    .child(
      h_flex()
        .items_center()
        .gap(px(2.0))
        // .child(toolbar_button(ICON_MIC, None))
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
