pub mod agent;
pub mod config;
pub mod updates;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::PoisonError;
use std::sync::RwLock;

// use self::agent::AgentHarness;
use self::config::Config;

static APP_SERVICES: OnceLock<AppServices> = OnceLock::new();

pub struct AppServices {
  // pub agent_harness: Arc<AgentHarness>,
  config_path: PathBuf,
  // A config that fails to load is kept as its error so the app still starts
  // and chats can report it.
  config: RwLock<Arc<anyhow::Result<Config>>>,
}

impl AppServices {
  pub fn new(
    config_path: PathBuf,
    config: anyhow::Result<Config>,
  ) -> Self {
    Self {
      config_path,
      config: RwLock::new(Arc::new(config)),
    }
  }

  pub fn config_path(&self) -> &Path {
    &self.config_path
  }

  pub fn config(&self) -> Arc<anyhow::Result<Config>> {
    self
      .config
      .read()
      .unwrap_or_else(PoisonError::into_inner)
      .clone()
  }

  pub fn set_config(
    &self,
    config: anyhow::Result<Config>,
  ) {
    *self.config.write().unwrap_or_else(PoisonError::into_inner) = Arc::new(config);
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
