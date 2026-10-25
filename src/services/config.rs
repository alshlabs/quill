use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use serde::Deserialize;

const ENV_PREFIX: &str = "${env.";

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  #[serde(skip)]
  pub config_path: PathBuf,
  pub models: Vec<AgentConfig>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
  pub name: String,
  pub model_id: String,
  pub provider_host: String,
  pub provider_token: String,
  #[serde(default)]
  pub vision: bool,
  #[serde(default)]
  pub thinking: bool,
  pub max_input_tokens: usize,
  pub max_output_tokens: usize,
}

impl Config {
  pub fn from_yaml(
    content: &str,
    file_path: &Path,
  ) -> anyhow::Result<Self> {
    let mut config = serde_yaml::from_str::<Config>(&interpolate_env(content)?)?;
    config.config_path = file_path.to_path_buf();
    Ok(config)
  }

  pub fn from_yaml_file(file_path: &Path) -> anyhow::Result<Self> {
    Self::from_yaml(&fs::read_to_string(file_path)?, file_path)
  }

  pub fn resolve_config(root: &Path) -> anyhow::Result<PathBuf> {
    if std::env::var_os("QUILL_CONFIG_AUTO_RESOLVE").is_some() {
      return Self::search_config(root).context("Cannot find config");
    }

    let config_path = home::home_dir()
      .context("Failed to get home directory")?
      .join(".config/quill/config.yml");

    if !config_path.is_file() {
      fs::create_dir_all(config_path.parent().context("Invalid config path")?)?;
      fs::write(&config_path, "models: []\n")?;
    }

    Ok(config_path)
  }

  fn search_config(root: &Path) -> Option<PathBuf> {
    const CONFIG_FILE_NAMES: [&str; 9] = [
      "quill.yaml",
      "quill.yml",
      ".quill.yaml",
      ".quill.yml",
      ".quill/quill.yaml",
      ".quill/quill.yml",
      ".quill/.quill.yaml",
      ".quill/.quill.yml",
      ".config/quill/config.yml",
    ];

    fn find_config_in(dir: &Path) -> Option<PathBuf> {
      CONFIG_FILE_NAMES
        .iter()
        .map(|name| dir.join(name))
        .find(|candidate| candidate.is_file())
    }

    root
      .ancestors()
      .find_map(find_config_in)
      .or_else(|| home::home_dir().and_then(|home| find_config_in(&home)))
  }
}

fn interpolate_env(content: &str) -> anyhow::Result<String> {
  let mut interpolated = String::with_capacity(content.len());
  let mut rest = content;

  while let Some(start) = rest.find(ENV_PREFIX) {
    let after_prefix = &rest[start + ENV_PREFIX.len()..];
    let Some(end) = after_prefix.find('}') else {
      break;
    };

    let name = &after_prefix[..end];
    let value = std::env::var(name)
      .with_context(|| format!("Environment variable `{name}` referenced in config is not set"))?;

    interpolated.push_str(&rest[..start]);
    interpolated.push_str(&value);
    rest = &after_prefix[end + 1..];
  }

  interpolated.push_str(rest);
  Ok(interpolated)
}

/*
"contextWindow": 0,
"reasoningEffortFormat": "chat-completions", "messages" "responses"
"streaming": true,
"requestHeaders": {}
"supportsReasoningEffort": []
"zeroDataRetentionEnabled": false
*/
