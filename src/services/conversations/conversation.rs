use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use std::sync::Weak;

use anyhow::Context as _;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

use super::ConversationId;
use super::Registry;
use super::goose;
use super::goose::AgentEvent;
use super::goose::AgentHarness;
use super::goose::AgentHarnessOptions;
use super::goose::HistoryMessage;
use super::goose::HistoryRole;
use crate::services::config::Config;
use crate::services::config::ConfigService;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum ConversationStatus {
  #[default]
  Idle,
  Loading,
  Responding,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MessageRole {
  User,
  Agent,
  Error,
}

#[derive(Clone)]
pub struct ConversationMessage {
  pub role: MessageRole,
  pub message_id: Option<String>,
  pub text: String,
}

impl ConversationMessage {
  fn user(text: String) -> Self {
    Self {
      role: MessageRole::User,
      message_id: None,
      text,
    }
  }

  fn error(text: String) -> Self {
    Self {
      role: MessageRole::Error,
      message_id: None,
      text,
    }
  }
}

impl From<HistoryMessage> for ConversationMessage {
  fn from(message: HistoryMessage) -> Self {
    Self {
      role: match message.role {
        HistoryRole::User => MessageRole::User,
        HistoryRole::Agent => MessageRole::Agent,
      },
      message_id: message.message_id,
      text: message.text,
    }
  }
}

#[derive(Clone, Default)]
pub struct ConversationSnapshot {
  /// Unset until the first prompt stores the conversation.
  pub id: Option<ConversationId>,
  pub title: Option<String>,
  /// Unset means the first configured model.
  pub model: Option<String>,
  pub status: ConversationStatus,
  pub messages: Vec<ConversationMessage>,
  pub deleted: bool,
}

impl ConversationSnapshot {
  /// Subscribers keep their copy with this too, so it always matches the
  /// service's.
  pub fn apply(
    &mut self,
    update: &ConversationUpdate,
  ) {
    match update {
      ConversationUpdate::Saved(id) => self.id = Some(id.clone()),
      ConversationUpdate::Title(title) => self.title = Some(title.clone()),
      ConversationUpdate::Model(model) => self.model = Some(model.clone()),
      ConversationUpdate::Status(status) => self.status = *status,
      ConversationUpdate::History(messages) => self.messages = messages.clone(),
      ConversationUpdate::Message(message) => self.messages.push(message.clone()),
      ConversationUpdate::TextDelta { message_id, delta } => match self.messages.last_mut() {
        Some(last) if last.role == MessageRole::Agent && last.message_id == *message_id => {
          last.text.push_str(delta)
        }
        _ => self.messages.push(ConversationMessage {
          role: MessageRole::Agent,
          message_id: message_id.clone(),
          text: delta.clone(),
        }),
      },
      ConversationUpdate::Deleted => self.deleted = true,
    }
  }
}

#[derive(Clone)]
pub enum ConversationUpdate {
  Saved(ConversationId),
  Title(String),
  Model(String),
  Status(ConversationStatus),
  /// Replaces the messages; only sent while loading, before any prompt.
  History(Vec<ConversationMessage>),
  Message(ConversationMessage),
  /// Extends the last message when it is the same agent message.
  TextDelta {
    message_id: Option<String>,
    delta: String,
  },
  Deleted,
}

pub enum ConversationCommand {
  Prompt(String),
  SetModel(String),
}

#[derive(Clone)]
pub struct Conversation {
  shared: Arc<Shared>,
}

pub(super) struct Shared {
  registry: Weak<Registry>,
  /// Only used when storing a new conversation; a stored one keeps its own.
  working_dir: Option<PathBuf>,
  prompts: UnboundedSender<String>,
  state: Mutex<SharedState>,
}

struct SharedState {
  snapshot: ConversationSnapshot,
  subscribers: Vec<UnboundedSender<ConversationUpdate>>,
}

impl Conversation {
  pub(super) fn spawn(
    registry: &Arc<Registry>,
    id: Option<ConversationId>,
    working_dir: Option<PathBuf>,
  ) -> Self {
    let (prompts, queue) = unbounded_channel();
    let snapshot = ConversationSnapshot {
      status: match id {
        Some(_) => ConversationStatus::Loading,
        None => ConversationStatus::Idle,
      },
      id,
      ..Default::default()
    };
    let shared = Arc::new(Shared {
      registry: Arc::downgrade(registry),
      working_dir,
      prompts,
      state: Mutex::new(SharedState {
        snapshot,
        subscribers: Vec::new(),
      }),
    });

    goose::runtime().spawn(run(Arc::downgrade(&shared), registry.config.clone(), queue));

    Self { shared }
  }

  pub(super) fn upgrade(shared: &Weak<Shared>) -> Option<Self> {
    shared.upgrade().map(|shared| Self { shared })
  }

  pub(super) fn downgrade(&self) -> Weak<Shared> {
    Arc::downgrade(&self.shared)
  }

  /// The current state, and every change to it from then on.
  pub fn subscribe(&self) -> (ConversationSnapshot, UnboundedReceiver<ConversationUpdate>) {
    let (tx, rx) = unbounded_channel();
    let mut state = self.state();
    state.subscribers.push(tx);
    (state.snapshot.clone(), rx)
  }

  /// A prompt is refused while the conversation is loading or responding, or
  /// once it has been deleted.
  pub fn send(
    &self,
    command: ConversationCommand,
  ) -> Result<()> {
    match command {
      ConversationCommand::Prompt(text) => {
        let mut state = self.state();
        if state.snapshot.deleted {
          bail!("The conversation was deleted");
        }
        if state.snapshot.status != ConversationStatus::Idle {
          bail!("The conversation is busy");
        }
        state.apply(ConversationUpdate::Message(ConversationMessage::user(
          text.clone(),
        )));
        state.apply(ConversationUpdate::Status(ConversationStatus::Responding));
        if self.shared.prompts.send(text).is_err() {
          let error = "The conversation has stopped";
          state.apply(ConversationUpdate::Message(ConversationMessage::error(
            error.to_string(),
          )));
          state.apply(ConversationUpdate::Status(ConversationStatus::Idle));
          bail!(error);
        }
      }
      ConversationCommand::SetModel(name) => self.apply(ConversationUpdate::Model(name)),
    }
    Ok(())
  }

  pub(super) fn apply(
    &self,
    update: ConversationUpdate,
  ) {
    self.state().apply(update);
  }

  fn state(&self) -> MutexGuard<'_, SharedState> {
    self
      .shared
      .state
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
  }

  fn reload_list(&self) {
    if let Some(registry) = self.shared.registry.upgrade() {
      registry.reload();
    }
  }

  async fn load(&self) {
    let Some(id) = self.state().snapshot.id.clone() else {
      return;
    };

    match goose::load_history(id.0).await {
      Ok(history) => {
        self.apply(ConversationUpdate::Title(history.name));
        self.apply(ConversationUpdate::History(
          history.messages.into_iter().map(Into::into).collect(),
        ));
      }
      Err(error) => self.apply(ConversationUpdate::Message(ConversationMessage::error(
        format!("{error:#}"),
      ))),
    }
    self.apply(ConversationUpdate::Status(ConversationStatus::Idle));
  }

  async fn respond(
    &self,
    connection: &mut Option<Connection>,
    config: &ConfigService,
    text: String,
  ) {
    match self.connect(connection, config) {
      Ok(agent) => {
        let mut events = agent.prompt(&text);
        while let Some(event) = events.recv().await {
          match event {
            AgentEvent::Session { id } => self.save(ConversationId(id)),
            AgentEvent::Text { message_id, delta } => {
              self.apply(ConversationUpdate::TextDelta { message_id, delta })
            }
            AgentEvent::Error(error) => self.apply(ConversationUpdate::Message(
              ConversationMessage::error(error),
            )),
          }
        }
      }
      Err(error) => self.apply(ConversationUpdate::Message(ConversationMessage::error(
        format!("{error:#}"),
      ))),
    }
    self.apply(ConversationUpdate::Status(ConversationStatus::Idle));
    self.reload_list();
  }

  /// Reuses the agent unless the settings or the chosen model changed since it
  /// connected.
  fn connect<'a>(
    &self,
    connection: &'a mut Option<Connection>,
    config: &ConfigService,
  ) -> Result<&'a AgentHarness> {
    let config = config.get();
    let (id, model) = {
      let state = self.state();
      (state.snapshot.id.clone(), state.snapshot.model.clone())
    };

    let current = connection
      .take()
      .filter(|current| Arc::ptr_eq(&current.config, &config) && current.model == model);
    let connection = match current {
      Some(current) => connection.insert(current),
      None => {
        let resolved = config
          .as_ref()
          .as_ref()
          .map_err(|error| anyhow!("Settings failed to load: {error:#}"))?
          .resolve_model(model.as_deref())
          .context("No models configured")?;
        let mut agent = AgentHarness::new(AgentHarnessOptions {
          session_id: id.map(|id| id.0),
          working_dir: self.shared.working_dir.clone(),
          ..resolved.into()
        });
        if let Some(names) = agent.take_names() {
          forward_names(self.downgrade(), names);
        }
        connection.insert(Connection {
          agent,
          config: config.clone(),
          model,
        })
      }
    };
    Ok(&connection.agent)
  }

  fn save(
    &self,
    id: ConversationId,
  ) {
    self.apply(ConversationUpdate::Saved(id.clone()));
    if let Some(registry) = self.shared.registry.upgrade() {
      registry.register(id, self);
      registry.reload();
    }
  }
}

impl SharedState {
  fn apply(
    &mut self,
    update: ConversationUpdate,
  ) {
    self.snapshot.apply(&update);
    self
      .subscribers
      .retain(|subscriber| subscriber.send(update.clone()).is_ok());
  }
}

struct Connection {
  agent: AgentHarness,
  config: Arc<Result<Config>>,
  model: Option<String>,
}

// Holds the conversation only while working, so it ends once every handle is
// dropped, but a response already underway still finishes and is stored.
async fn run(
  shared: Weak<Shared>,
  config: ConfigService,
  mut prompts: UnboundedReceiver<String>,
) {
  if let Some(conversation) = Conversation::upgrade(&shared) {
    conversation.load().await;
  }

  let mut connection = None;
  while let Some(text) = prompts.recv().await {
    let Some(conversation) = Conversation::upgrade(&shared) else {
      return;
    };
    conversation.respond(&mut connection, &config, text).await;
  }
}

fn forward_names(
  shared: Weak<Shared>,
  mut names: UnboundedReceiver<String>,
) {
  goose::runtime().spawn(async move {
    while let Some(name) = names.recv().await {
      let Some(conversation) = Conversation::upgrade(&shared) else {
        return;
      };
      let name = name.trim();
      if !name.is_empty() {
        conversation.apply(ConversationUpdate::Title(name.to_string()));
        conversation.reload_list();
      }
    }
  });
}
