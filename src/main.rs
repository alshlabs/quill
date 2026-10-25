mod app;
mod components;
#[rustfmt::skip]
mod icons;
mod panels;
mod services;
mod sidebars;
mod tabs;
mod theme;
mod window;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use gpui_kit::*;

use crate::services::AppServices;
use crate::services::config::Config;

fn main() -> anyhow::Result<()> {
  let config = Config::from_yaml_file(&match std::env::var("QUILL_CONFIG") {
    Ok(v) => PathBuf::from(v),
    Err(_) => Config::resolve_config(
      std::env::current_exe()?
        .parent()
        .context("Failed to get path to exe")?,
    )?,
  })?;

  dbg!(&config);

  services::agent::init(&config)?;

  AppServices {
    config: Arc::new(config),
  }
  .register_global()?;

  window::bootstrap(
    //
    app::init,
    |_cx, _options| async {},
    |window, cx| cx.new(|cx| app::AppRoot::new(window, cx)),
    |_window, _cx| {},
  );

  Ok(())
}
