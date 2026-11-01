use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::Result;

/// The folder chats run in. Global chats have no folder.
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub enum Workspace {
  #[default]
  Global,
  Folder(PathBuf),
}

impl Workspace {
  /// Canonical, so the same folder always compares equal however it was picked.
  pub fn open(dir: &Path) -> Result<Self> {
    let dir = dunce::canonicalize(dir).with_context(|| format!("Cannot open {}", dir.display()))?;
    Ok(Self::Folder(dir))
  }

  pub fn folder(&self) -> Option<&Path> {
    match self {
      Self::Global => None,
      Self::Folder(dir) => Some(dir),
    }
  }

  pub fn name(&self) -> String {
    match self {
      Self::Global => "Global workspace".to_string(),
      Self::Folder(dir) => dir
        .file_name()
        .unwrap_or(dir.as_os_str())
        .to_string_lossy()
        .into_owned(),
    }
  }
}
