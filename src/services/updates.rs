use std::future::Future;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use self_update::ReleaseAsset;
use self_update::backends::github;

const BIN_NAME: &str = "quill";
const APP_BUNDLE: &str = "Quill.app";
const APP_IMAGE: &str = "Quill.AppImage";

pub const TARGET_REPO: &str = "alshlabs/quill";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum UpgradeOutcome {
  Skip,
  Success,
}

pub fn check_for_update() -> impl Future<Output = anyhow::Result<Option<semver::Version>>> {
  spawn(async {
    if !is_installed_release()? {
      return Ok(None);
    }

    let releases = configure()?
      .build_async()?
      .get_latest_release_async()
      .await?;

    let Some(release) = releases.latest() else {
      return Ok(None);
    };

    let current_version = semver::Version::parse(CURRENT_VERSION)?;
    let remote_version = normalize_tag(release.version())?;

    if current_version < remote_version {
      return Ok(Some(remote_version));
    }

    Ok(None)
  })
}

pub fn try_upgrade() -> impl Future<Output = anyhow::Result<UpgradeOutcome>> {
  spawn(async {
    if !is_installed_release()? {
      return Ok(UpgradeOutcome::Skip);
    }

    let status = configure()?.build_async()?.update_async().await?;

    match status {
      self_update::VersionStatus::Updated(_) => Ok(UpgradeOutcome::Success),
      _ => Ok(UpgradeOutcome::Skip),
    }
  })
}

/// Starts the freshly installed release. The caller is expected to quit right
/// after, so the old process does not linger beside the new one.
pub fn relaunch() -> anyhow::Result<()> {
  let mut command = match std::env::consts::OS {
    "macos" => {
      let mut command = Command::new("open");
      // `-n` because this instance is still running when the new one starts.
      command.arg("-n").arg(app_bundle_path()?);
      command
    }
    "linux" => Command::new(app_image_path()?),
    _ => Command::new(std::env::current_exe()?),
  };
  command.spawn()?;
  Ok(())
}

fn configure() -> anyhow::Result<github::UpdateBuilder> {
  let Some((owner, name)) = TARGET_REPO.split_once('/') else {
    anyhow::bail!("Invalid repo name");
  };

  let mut builder = github::Update::configure();
  builder
    .repo_owner(owner)
    .repo_name(name)
    .bin_name(BIN_NAME)
    .current_version(CURRENT_VERSION)
    .asset_matcher(select_asset)
    .unattended();

  match std::env::consts::OS {
    "macos" => {
      // The whole bundle is swapped, so the code signature stays consistent with
      // the executable; replacing only the inner binary would break it.
      builder
        .bundle_path_in_archive(APP_BUNDLE)
        .bundle_install_path(app_bundle_path()?);
    }
    "linux" => {
      // The running executable lives inside the AppImage's read-only mount; the
      // file to replace is the AppImage itself.
      builder
        .bin_path_in_archive(APP_IMAGE)
        .bin_install_path(app_image_path()?);
    }
    _ => {}
  }

  Ok(builder)
}

/// Local builds carry a prerelease version and are not packaged, so they never
/// update themselves.
fn is_installed_release() -> anyhow::Result<bool> {
  if !semver::Version::parse(CURRENT_VERSION)?.pre.is_empty() {
    return Ok(false);
  }

  Ok(match std::env::consts::OS {
    "macos" => app_bundle_path().is_ok(),
    "linux" => app_image_path().is_ok(),
    _ => true,
  })
}

fn app_bundle_path() -> anyhow::Result<PathBuf> {
  let exe = std::env::current_exe()?;
  exe
    .ancestors()
    .find(|path| path.extension().is_some_and(|ext| ext == "app"))
    .map(Path::to_path_buf)
    .context("Not running from an app bundle")
}

fn app_image_path() -> anyhow::Result<PathBuf> {
  std::env::var_os("APPIMAGE")
    .map(PathBuf::from)
    .context("Not running from an AppImage")
}

fn normalize_tag(tag: &str) -> anyhow::Result<semver::Version> {
  let tag = tag.strip_prefix('v').unwrap_or(tag);
  Ok(semver::Version::parse(tag)?)
}

fn select_asset(assets: &[ReleaseAsset]) -> Option<ReleaseAsset> {
  let (os, extension) = match std::env::consts::OS {
    "windows" => ("windows", ".zip"),
    "macos" => ("macos", ".tar.gz"),
    "linux" => ("linux", ".tar.gz"),
    _ => return None,
  };

  let arch = match std::env::consts::ARCH {
    "x86_64" => "amd64",
    "aarch64" => "arm64",
    _ => return None,
  };

  let name = format!("{BIN_NAME}-{os}-{arch}{extension}");

  assets
    .iter()
    .find(|asset| asset.name().eq_ignore_ascii_case(&name))
    .cloned()
}

/// The GitHub client needs a tokio reactor, which the UI executor does not
/// provide. Update checks are rare, so each runs on its own short-lived runtime.
fn spawn<T: Send + 'static>(
  future: impl Future<Output = anyhow::Result<T>> + Send + 'static
) -> impl Future<Output = anyhow::Result<T>> {
  let (tx, rx) = futures::channel::oneshot::channel();
  std::thread::spawn(move || {
    let result = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .map_err(anyhow::Error::from)
      .and_then(|runtime| runtime.block_on(future));
    let _ = tx.send(result);
  });
  async move { rx.await? }
}
