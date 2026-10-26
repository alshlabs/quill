pub mod agent;
pub mod config;

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::PoisonError;
use std::sync::RwLock;

// use self::agent::AgentHarness;
use self::config::Config;

static APP_SERVICES: OnceLock<AppServices> = OnceLock::new();

pub struct AppServices {
  // pub agent_harness: Arc<AgentHarness>,
  config: RwLock<Arc<Config>>,
}

impl AppServices {
  pub fn new(config: Config) -> Self {
    Self {
      config: RwLock::new(Arc::new(config)),
    }
  }

  pub fn config(&self) -> Arc<Config> {
    self
      .config
      .read()
      .unwrap_or_else(PoisonError::into_inner)
      .clone()
  }

  pub fn set_config(
    &self,
    config: Config,
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
