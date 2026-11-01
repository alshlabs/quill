#![deny(unused_crate_dependencies)]

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

use anyhow::Context;
use gpui_kit::*;

use crate::services::AppServices;
use crate::services::config::Config;
use crate::services::config::ConfigService;
use crate::services::conversations::Conversations;

fn main() -> anyhow::Result<()> {
  let config_path = match std::env::var("QUILL_CONFIG") {
    Ok(v) => PathBuf::from(v),
    Err(_) => Config::resolve_config(
      std::env::current_exe()?
        .parent()
        .context("Failed to get path to exe")?,
    )?,
  };
  let config = Config::from_yaml_file(&config_path);

  dbg!(&config);

  let config = ConfigService::new(config);
  let conversations = Conversations::new(config.clone())?;

  AppServices::new(config_path, config, conversations).register_global()?;

  window::bootstrap(
    //
    app::init,
    |_cx, _options| async {},
    |window, cx| cx.new(|cx| app::AppRoot::new(window, cx)),
    |_window, _cx| {},
  );

  Ok(())
}
