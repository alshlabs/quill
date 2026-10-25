pub mod agent;
pub mod config;

use std::sync::Arc;
use std::sync::OnceLock;

// use self::agent::AgentHarness;
use self::config::Config;

static APP_SERVICES: OnceLock<AppServices> = OnceLock::new();

#[derive(Clone)]
pub struct AppServices {
  // pub agent_harness: Arc<AgentHarness>,
  pub config: Arc<Config>,
}

impl AppServices {
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
