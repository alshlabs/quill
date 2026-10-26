use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use self_update::ReleaseAsset;
use self_update::backends::github;

const BIN_NAME: &str = "http-server-rs";

pub const TARGET_REPO: &str = "alshdavid/http-server-rs";

pub async fn check_for_update(
  options: &UpgradeOptions<'_>
) -> anyhow::Result<Option<semver::Version>> {
  if is_prerelease(options.current_version)? {
    return Ok(None);
  }

  let releases = configure(options)?
    .build_async()?
    .get_latest_release_async()
    .await?;

  let Some(release) = releases.latest() else {
    return Ok(None);
  };

  let current_version = semver::Version::parse(options.current_version)?;
  let remote_version = normalize_tag(release.version())?;

  if current_version < remote_version {
    return Ok(Some(remote_version));
  }

  Ok(None)
}

#[derive(Debug)]
pub enum UpgradeOutcome {
  Skip,
  Success,
}

#[derive(Debug)]
pub struct UpgradeOptions<'a> {
  pub target_repo: &'a str,
  pub current_version: &'a str,
}

pub async fn try_upgrade(options: &UpgradeOptions<'_>) -> anyhow::Result<UpgradeOutcome> {
  if is_prerelease(options.current_version)? {
    return Ok(UpgradeOutcome::Skip);
  }

  let status = configure(options)?
    .progress_callback(progress())
    .build_async()?
    .update_async()
    .await?;

  match status {
    self_update::VersionStatus::Updated(_) => Ok(UpgradeOutcome::Success),
    _ => Ok(UpgradeOutcome::Skip),
  }
}

fn configure(options: &UpgradeOptions<'_>) -> anyhow::Result<github::UpdateBuilder> {
  let Some((owner, name)) = options.target_repo.split_once('/') else {
    anyhow::bail!("Invalid repo name");
  };

  let mut builder = github::Update::configure();
  builder
    .repo_owner(owner)
    .repo_name(name)
    .bin_name(BIN_NAME)
    .current_version(options.current_version)
    .asset_matcher(select_asset)
    .unattended();

  Ok(builder)
}

fn is_prerelease(version: &str) -> anyhow::Result<bool> {
  Ok(!semver::Version::parse(version)?.pre.is_empty())
}

fn normalize_tag(tag: &str) -> anyhow::Result<semver::Version> {
  let tag = tag.strip_prefix('v').unwrap_or(tag);
  Ok(semver::Version::parse(tag)?)
}

fn select_asset(assets: &[ReleaseAsset]) -> Option<ReleaseAsset> {
  let os = match std::env::consts::OS {
    "windows" => &["win32", "windows", "win"][..],
    "macos" => &["macos", "darwin"][..],
    "linux" => &["linux"][..],
    _ => return None,
  };

  let arch = match std::env::consts::ARCH {
    "x86_64" => &["amd64", "x86_64"][..],
    "aarch64" => &["arm64", "aarch64"][..],
    _ => return None,
  };

  assets
    .iter()
    .find(|asset| {
      let name = asset.name().to_ascii_lowercase();
      name.ends_with(".tar.gz")
        && os.iter().any(|s| name.contains(s))
        && arch.iter().any(|s| name.contains(s))
    })
    .cloned()
}

fn progress() -> impl Fn(u64, Option<u64>) + Send + Sync + 'static {
  let started = AtomicBool::new(false);
  let reported = AtomicU64::new(0);

  move |downloaded, total| {
    if !started.swap(true, Ordering::Relaxed) {
      println!("0%");
    }

    let Some(total) = total.filter(|total| *total > 0) else {
      return;
    };

    let step = (downloaded * 4 / total).min(4);
    if step > reported.load(Ordering::Relaxed) {
      reported.store(step, Ordering::Relaxed);
      println!("{}%", step * 25);
    }
  }
}
