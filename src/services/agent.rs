use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::SystemTime;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use futures::StreamExt;
use goose::agents::Agent;
use goose::agents::AgentConfig as GooseAgentConfig;
use goose::agents::AgentEvent as GooseEvent;
use goose::agents::ExtensionConfig;
use goose::agents::GoosePlatform;
use goose::agents::SessionConfig;
use goose::agents::extension::PLATFORM_EXTENSIONS;
use goose::agents::state_machine;
use goose::builtin_extension::get_builtin_extension_names;
use goose::builtin_extension::register_builtin_extensions;
use goose::config::Config as GooseConfig;
use goose::config::GooseMode;
use goose::config::PermissionManager;
use goose::conversation::message::Message;
use goose::model_config::model_config_from_user_config;
use goose::providers::create_with_named_model;
use goose::session::Session;
use goose::session::SessionManager;
use goose::session::SessionNameUpdate;
use goose::session::session_manager::SessionType;
use rmcp::model::Role;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

use crate::services::config::AgentConfig;
use crate::services::config::Config;

const PROVIDER: &str = "openai";
const SESSION_NAME: &str = "quill";
const UNTITLED_CHAT: &str = "Untitled chat";
const AGENT_STACK_SIZE: usize = 32 * 1024 * 1024;
const GOOSE_ROOT: &str = ".config/quill";
const SCRATCH_DIR: &str = "scratch";
const NO_DIRECTORY_PROMPT_KEY: &str = "quill_no_directory";
const NO_DIRECTORY_PROMPT: &str = "This chat is not associated with any directory or \
  project. The working directory in your turn context is an internal scratch space, \
  not the user's location or project: never present it as where the user is. If \
  asked what directory you are in, say this chat has no directory.";

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Configures goose and starts the runtime every agent and storage call runs on.
/// Must run before any other goose code: goose reads its environment once.
pub fn init(config: &Config) -> Result<()> {
  // goose resolves the OpenAI-compatible endpoint and key from the process
  // environment; edition 2024 marks env mutation unsafe.
  unsafe {
    std::env::set_var("GOOSE_DISABLE_KEYRING", "1");
    std::env::set_var("GOOSE_PROVIDER", PROVIDER);

    if let Some(model) = config.models.first() {
      std::env::set_var("GOOSE_MODEL", &model.model_id);
      std::env::set_var("OPENAI_HOST", &model.provider_host);
      std::env::set_var("GOOSE_PROVIDER__HOST", &model.provider_host);
      std::env::set_var("OPENAI_API_KEY", &model.provider_token);
      std::env::set_var("GOOSE_PROVIDER__API_KEY", &model.provider_token);
    }

    // goose reads its storage root once, on first use, and only accepts an
    // absolute path; anything else falls back to goose's own directories.
    if let Some(root) = goose_root() {
      std::env::set_var("GOOSE_PATH_ROOT", root);
    }
  }

  register_builtin_extensions(goose_mcp::BUILTIN_EXTENSIONS.clone());

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .thread_name("goose")
    .thread_stack_size(AGENT_STACK_SIZE)
    .build()?;

  RUNTIME
    .set(runtime)
    .map_err(|_| anyhow!("goose is already initialized"))
}

fn runtime() -> &'static Runtime {
  RUNTIME.get().expect("goose is not initialized")
}

fn spawn<T: Send + 'static>(
  future: impl Future<Output = Result<T>> + Send + 'static
) -> impl Future<Output = Result<T>> {
  let task = runtime().spawn(future);
  async move { task.await? }
}

pub struct ChatSummary {
  pub id: String,
  pub name: String,
  pub updated_at: SystemTime,
}

pub enum HistoryRole {
  User,
  Agent,
}

pub struct HistoryMessage {
  pub role: HistoryRole,
  pub message_id: Option<String>,
  pub text: String,
}

pub struct ChatHistory {
  pub name: String,
  pub messages: Vec<HistoryMessage>,
}

/// Chats with at least one message, most recently active first.
pub fn list_chats() -> impl Future<Output = Result<Vec<ChatSummary>>> {
  spawn(async {
    let mut sessions = SessionManager::instance()
      .list_sessions_by_types(&[SessionType::Hidden])
      .await?;
    sessions.retain(|session| session.message_count > 0);
    sessions.sort_by_key(|session| std::cmp::Reverse(last_active(session)));

    Ok(
      sessions
        .into_iter()
        .map(|session| ChatSummary {
          name: display_name(&session),
          updated_at: last_active(&session),
          id: session.id,
        })
        .collect(),
    )
  })
}

/// The messages a chat shows: user-visible text only, so goose's turn context,
/// tool calls, tool results and thinking are left out.
pub fn load_history(session_id: String) -> impl Future<Output = Result<ChatHistory>> {
  spawn(async move {
    let session = SessionManager::instance()
      .get_session(&session_id, true)
      .await?;

    let messages = session
      .conversation
      .iter()
      .flat_map(|conversation| conversation.messages().iter())
      .filter(|message| message.is_user_visible())
      .filter_map(|message| {
        let text = message.as_concat_text();
        if text.is_empty() {
          return None;
        }
        let role = match message.role {
          Role::User => HistoryRole::User,
          Role::Assistant => HistoryRole::Agent,
        };
        Some(HistoryMessage {
          role,
          message_id: message.id.clone(),
          text,
        })
      })
      .collect();

    Ok(ChatHistory {
      name: display_name(&session),
      messages,
    })
  })
}

fn last_active(session: &Session) -> SystemTime {
  session.last_message_at.unwrap_or(session.updated_at).into()
}

fn display_name(session: &Session) -> String {
  let name = session.name.trim();
  if name.is_empty() || name == SESSION_NAME {
    UNTITLED_CHAT.to_string()
  } else {
    name.to_string()
  }
}

pub struct AgentHarnessOptions {
  pub model: String,
  pub working_dir: Option<PathBuf>,
  /// Resume this goose session instead of starting a new one.
  pub session_id: Option<String>,
}

impl From<&AgentConfig> for AgentHarnessOptions {
  fn from(config: &AgentConfig) -> Self {
    Self {
      model: config.model_id.clone(),
      working_dir: None,
      session_id: None,
    }
  }
}

pub enum AgentEvent {
  Session {
    id: String,
  },
  Text {
    message_id: Option<String>,
    delta: String,
  },
  Error(String),
}

// One agent and session per harness; prompts are queued to it so the session
// accumulates the conversation history across turns. The session is only
// opened on the first prompt so unused chats don't leave empty sessions behind.
pub struct AgentHarness {
  prompts: UnboundedSender<Prompt>,
  names: Option<UnboundedReceiver<String>>,
}

impl AgentHarness {
  pub fn new(options: AgentHarnessOptions) -> Self {
    let (prompts, queue) = unbounded_channel();
    let (names_tx, names) = unbounded_channel();

    runtime().spawn(run(options, queue, names_tx));

    Self {
      prompts,
      names: Some(names),
    }
  }

  /// Names goose generates for the session, sent in parallel with the first
  /// few prompts. Can only be taken once.
  pub fn take_names(&mut self) -> Option<UnboundedReceiver<String>> {
    self.names.take()
  }

  pub fn prompt(
    &self,
    message: &str,
  ) -> UnboundedReceiver<AgentEvent> {
    let (tx, rx) = unbounded_channel();
    let _ = self.prompts.send(Prompt {
      text: message.to_string(),
      tx,
    });
    rx
  }
}

async fn run(
  options: AgentHarnessOptions,
  mut queue: UnboundedReceiver<Prompt>,
  names: UnboundedSender<String>,
) {
  let agent = Arc::new(create_agent(names));
  let mut session: Option<Session> = None;

  while let Some(prompt) = queue.recv().await {
    if session.is_none() {
      match open_session(&agent, &options).await {
        Ok(opened) => {
          let _ = prompt.tx.send(AgentEvent::Session {
            id: opened.id.clone(),
          });
          session = Some(opened);
        }
        Err(error) => {
          fail(&prompt.tx, error);
          continue;
        }
      }
    }

    if let Some(session) = &session {
      respond(&agent, session, prompt).await;
    }
  }
}

fn create_agent(names: UnboundedSender<String>) -> Agent {
  let (name_updates_tx, mut name_updates) = unbounded_channel::<SessionNameUpdate>();
  tokio::spawn(async move {
    while let Some(update) = name_updates.recv().await {
      if names.send(update.name).is_err() {
        return;
      }
    }
  });

  // Mirrors `Agent::new`, except session naming is forced on because the
  // generated names are forwarded to the UI.
  Agent::with_config(
    GooseAgentConfig::new(
      Arc::new(SessionManager::instance()),
      PermissionManager::instance(),
      None,
      GooseConfig::global().get_goose_mode().unwrap_or_default(),
      false,
      GoosePlatform::GooseCli,
    )
    .with_session_name_update_tx(Some(name_updates_tx)),
  )
}

async fn open_session(
  agent: &Arc<Agent>,
  options: &AgentHarnessOptions,
) -> Result<Session> {
  let scratch = scratch_dir()?;
  let session = match &options.session_id {
    Some(id) => resume_session(agent, id).await?,
    None => {
      let working_dir = options.working_dir.clone().unwrap_or(scratch.clone());
      create_session(agent, &options.model, working_dir).await?
    }
  };

  if session.working_dir == scratch {
    agent
      .extend_system_prompt(
        NO_DIRECTORY_PROMPT_KEY.to_string(),
        NO_DIRECTORY_PROMPT.to_string(),
      )
      .await;
  }

  Ok(session)
}

async fn resume_session(
  agent: &Arc<Agent>,
  id: &str,
) -> Result<Session> {
  let session = agent.config.session_manager.get_session(id, false).await?;
  agent.restore_provider_from_session(&session).await?;
  agent.load_extensions_from_session(&session).await;
  Ok(session)
}

async fn create_session(
  agent: &Agent,
  model: &str,
  working_dir: PathBuf,
) -> Result<Session> {
  let provider = create_with_named_model(PROVIDER, Vec::new()).await?;
  let model_config = model_config_from_user_config(PROVIDER, model)?;

  let session = agent
    .config
    .session_manager
    .create_session(
      working_dir,
      SESSION_NAME.to_string(),
      SessionType::Hidden,
      GooseMode::default(),
    )
    .await?;

  agent
    .update_provider(provider, model_config, &session.id)
    .await?;

  let mut extensions: Vec<_> = PLATFORM_EXTENSIONS
    .values()
    .filter(|definition| definition.default_enabled)
    .collect();
  extensions.sort_unstable_by_key(|definition| definition.name);
  for definition in extensions {
    agent
      .add_extension(
        ExtensionConfig::Platform {
          name: definition.name.to_string(),
          description: definition.description.to_string(),
          display_name: Some(definition.display_name.to_string()),
          bundled: Some(true),
          available_tools: Vec::new(),
        },
        &session.id,
      )
      .await
      .with_context(|| format!("failed to add {}", definition.name))?;
  }

  let mut builtins = get_builtin_extension_names();
  builtins.sort_unstable();
  for name in builtins {
    agent
      .add_extension(
        ExtensionConfig::Builtin {
          name: name.to_string(),
          description: builtin_description(name).to_string(),
          display_name: None,
          timeout: None,
          bundled: Some(true),
          available_tools: Vec::new(),
        },
        &session.id,
      )
      .await
      .with_context(|| format!("failed to add {name}"))?;
  }

  Ok(session)
}

fn goose_root() -> Option<PathBuf> {
  home::home_dir().map(|home| home.join(GOOSE_ROOT))
}

// goose needs a real directory for every session (shell cwd, turn context and
// AGENTS.md hint lookup), so chats without one get an empty, project-free one.
fn scratch_dir() -> Result<PathBuf> {
  let dir = goose_root()
    .unwrap_or_else(|| std::env::temp_dir().join("quill"))
    .join(SCRATCH_DIR);
  std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
  Ok(dir)
}

fn builtin_description(name: &str) -> &'static str {
  match name {
    "autovisualiser" => "Data visualisation and UI generation tools",
    "computercontroller" => "Read and write Word, PDF, and Excel documents",
    "memory" => "Tools to save and retrieve durable memories",
    "tutorial" => "Access interactive tutorials and guides",
    _ => "Bundled goose extension",
  }
}

async fn respond(
  agent: &Agent,
  session: &Session,
  prompt: Prompt,
) {
  let session_config = SessionConfig {
    id: session.id.clone(),
    schedule_id: None,
    max_turns: None,
    retry_config: None,
  };

  let mut stream = match agent
    .reply(
      Message::user().with_text(&prompt.text),
      session_config,
      state_machine::enabled(),
      None,
    )
    .await
  {
    Ok(stream) => stream,
    Err(error) => {
      fail(&prompt.tx, error);
      return;
    }
  };

  while let Some(event) = stream.next().await {
    match event {
      Ok(GooseEvent::Message(message)) => {
        let delta = message.as_concat_text();
        if delta.is_empty() {
          continue;
        }
        let event = AgentEvent::Text {
          message_id: message.id,
          delta,
        };
        if prompt.tx.send(event).is_err() {
          return;
        }
      }
      Ok(_) => {}
      Err(error) => {
        fail(&prompt.tx, error);
        return;
      }
    }
  }
}

fn fail(
  tx: &UnboundedSender<AgentEvent>,
  error: anyhow::Error,
) {
  let _ = tx.send(AgentEvent::Error(error.to_string()));
}

struct Prompt {
  text: String,
  tx: UnboundedSender<AgentEvent>,
}

/*

let &AppServices { agent_harness, .. } = &AppServices::obtain();

let mut rx = agent_harness.prompt("Hello");

while let Some(msg) = rx.recv().await {
  println!("{}", msg);
}

*/
