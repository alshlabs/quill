pub mod config;
pub mod conversations;
pub mod updates;
pub mod workspace;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

// use self::agent::AgentHarness;
use self::config::Config;
use self::config::ConfigService;
use self::conversations::Conversations;

static APP_SERVICES: OnceLock<AppServices> = OnceLock::new();

pub struct AppServices {
  // pub agent_harness: Arc<AgentHarness>,
  config_path: PathBuf,
  config: ConfigService,
  conversations: Conversations,
}

impl AppServices {
  pub fn new(
    config_path: PathBuf,
    config: ConfigService,
    conversations: Conversations,
  ) -> Self {
    Self {
      config_path,
      config,
      conversations,
    }
  }

  pub fn config_path(&self) -> &Path {
    &self.config_path
  }

  pub fn config(&self) -> Arc<anyhow::Result<Config>> {
    self.config.get()
  }

  pub fn set_config(
    &self,
    config: anyhow::Result<Config>,
  ) {
    self.config.set(config);
  }

  pub fn conversations(&self) -> &Conversations {
    &self.conversations
  }

  pub fn register_global(self) -> anyhow::Result<()> {
    if APP_SERVICES.set(self).is_err() {
      return Err(anyhow::anyhow!("already initialized"));
    }
    Ok(())
  }

  pub fn obtain() -> &'static Self {
    APP_SERVICES.get().expect("Services not initialized")
  }
}
