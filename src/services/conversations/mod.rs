mod conversation;
mod goose;

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use std::sync::Weak;
use std::time::SystemTime;

use anyhow::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

pub use self::conversation::Conversation;
pub use self::conversation::ConversationCommand;
pub use self::conversation::ConversationMessage;
pub use self::conversation::ConversationSnapshot;
pub use self::conversation::ConversationStatus;
pub use self::conversation::ConversationUpdate;
pub use self::conversation::MessageRole;
use crate::services::config::ConfigService;
use crate::services::workspace::Workspace;

/// Identifies a stored conversation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConversationId(String);

#[derive(Clone)]
pub struct ConversationSummary {
  pub id: ConversationId,
  pub name: String,
  pub updated_at: SystemTime,
  pub workspace: Workspace,
}

/// Stored conversations with at least one message, most recently active first,
/// or why they could not be listed.
pub type ConversationList = Result<Vec<ConversationSummary>, String>;

#[derive(Clone)]
pub struct Conversations {
  registry: Arc<Registry>,
}

impl Conversations {
  /// Must run before anything else touches goose, which reads its environment
  /// once.
  pub fn new(config: ConfigService) -> Result<Self> {
    goose::init()?;
    Ok(Self {
      registry: Arc::new(Registry {
        config,
        state: Mutex::default(),
      }),
    })
  }

  /// Receives the workspace's whole list whenever it changes, starting with the
  /// current one.
  pub fn subscribe(
    &self,
    workspace: Workspace,
  ) -> UnboundedReceiver<ConversationList> {
    let (tx, rx) = unbounded_channel();
    let mut state = self.registry.state();
    if let Some(list) = &state.list {
      let _ = tx.send(in_workspace(list, &workspace));
    }
    state.subscribers.push((workspace, tx));
    let load = state.requested == 0;
    drop(state);

    if load {
      self.registry.reload();
    }
    rx
  }

  /// A new conversation, stored once its first prompt is sent.
  pub fn create(
    &self,
    workspace: &Workspace,
  ) -> Conversation {
    Conversation::spawn(&self.registry, None, workspace.folder().map(Into::into))
  }

  /// Everyone opening the same conversation shares it, and so shares one agent.
  pub fn open(
    &self,
    id: &ConversationId,
  ) -> Conversation {
    let mut state = self.registry.state();
    state
      .open
      .retain(|_, conversation| conversation.strong_count() > 0);
    if let Some(conversation) = state.open.get(id).and_then(Conversation::upgrade) {
      return conversation;
    }

    let conversation = Conversation::spawn(&self.registry, Some(id.clone()), None);
    state.open.insert(id.clone(), conversation.downgrade());
    conversation
  }

  pub fn rename(
    &self,
    id: &ConversationId,
    name: String,
  ) -> impl Future<Output = Result<()>> + use<> {
    let registry = self.registry.clone();
    let id = id.clone();
    let rename = goose::rename_chat(id.0.clone(), name.clone());
    async move {
      rename.await?;
      if let Some(conversation) = registry.find(&id) {
        conversation.apply(ConversationUpdate::Title(name));
      }
      registry.reload();
      Ok(())
    }
  }

  pub fn delete(
    &self,
    id: &ConversationId,
  ) -> impl Future<Output = Result<()>> + use<> {
    let registry = self.registry.clone();
    let id = id.clone();
    let delete = goose::delete_chat(id.0.clone());
    async move {
      delete.await?;
      if let Some(conversation) = registry.remove(&id) {
        conversation.apply(ConversationUpdate::Deleted);
      }
      registry.reload();
      Ok(())
    }
  }

  pub fn duplicate(
    &self,
    id: &ConversationId,
  ) -> impl Future<Output = Result<()>> + use<> {
    let registry = self.registry.clone();
    let duplicate = goose::clone_chat(id.0.clone());
    async move {
      duplicate.await?;
      registry.reload();
      Ok(())
    }
  }
}

struct Registry {
  config: ConfigService,
  state: Mutex<RegistryState>,
}

#[derive(Default)]
struct RegistryState {
  open: HashMap<ConversationId, Weak<conversation::Shared>>,
  list: Option<ConversationList>,
  subscribers: Vec<(Workspace, UnboundedSender<ConversationList>)>,
  requested: u64,
  published: u64,
}

impl Registry {
  fn state(&self) -> MutexGuard<'_, RegistryState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn find(
    &self,
    id: &ConversationId,
  ) -> Option<Conversation> {
    self.state().open.get(id).and_then(Conversation::upgrade)
  }

  fn remove(
    &self,
    id: &ConversationId,
  ) -> Option<Conversation> {
    self
      .state()
      .open
      .remove(id)
      .as_ref()
      .and_then(Conversation::upgrade)
  }

  fn register(
    &self,
    id: ConversationId,
    conversation: &Conversation,
  ) {
    self.state().open.insert(id, conversation.downgrade());
  }

  fn reload(self: &Arc<Self>) {
    let generation = {
      let mut state = self.state();
      state.requested += 1;
      state.requested
    };

    let list = goose::list_chats();
    let registry = self.clone();
    goose::runtime().spawn(async move {
      let list = list
        .await
        .map(|chats| {
          chats
            .into_iter()
            .map(|chat| ConversationSummary {
              id: ConversationId(chat.id),
              name: chat.name,
              updated_at: chat.updated_at,
              workspace: chat
                .working_dir
                .map_or(Workspace::Global, Workspace::Folder),
            })
            .collect()
        })
        .map_err(|error| format!("{error:#}"));
      registry.publish(generation, list);
    });
  }

  // Reloads can finish out of order; an older one must not replace a newer list.
  fn publish(
    &self,
    generation: u64,
    list: ConversationList,
  ) {
    let mut state = self.state();
    if generation < state.published {
      return;
    }
    state.published = generation;
    state
      .subscribers
      .retain(|(workspace, subscriber)| subscriber.send(in_workspace(&list, workspace)).is_ok());
    state.list = Some(list);
  }
}

fn in_workspace(
  list: &ConversationList,
  workspace: &Workspace,
) -> ConversationList {
  list.as_ref().map_err(Clone::clone).map(|chats| {
    chats
      .iter()
      .filter(|chat| chat.workspace == *workspace)
      .cloned()
      .collect()
  })
}
