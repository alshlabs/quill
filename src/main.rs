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

  services::agent::init()?;

  AppServices::new(config_path, config).register_global()?;

  window::bootstrap(
    //
    app::init,
    |_cx, _options| async {},
    |window, cx| cx.new(|cx| app::AppRoot::new(window, cx)),
    |_window, _cx| {},
  );

  Ok(())
}
