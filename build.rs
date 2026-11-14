use std::env;
#[cfg(windows)]
use std::path::PathBuf;

fn main() {
  println!("cargo:rerun-if-changed=resources/icon.ico");

  if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
    return;
  }

  if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") && is_msvc_aarch64() {
    compile_with_sdk_rc();
    return;
  }

  winresource::WindowsResource::new()
    .set_icon("resources/icon.ico")
    .compile()
    .expect("failed to embed Windows resources");
}

fn is_msvc_aarch64() -> bool {
  let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
  let triple = env::var("TARGET").unwrap_or_default();
  arch == "aarch64" || triple.starts_with("aarch64-")
}

// winresource hands an x86 host's `windres` the aarch64 target, which it can't
// emit; the SDK's ARM64 `rc.exe` can.
#[cfg(windows)]
fn compile_with_sdk_rc() {
  let program_files_x86 =
    env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
  let bin_root = PathBuf::from(program_files_x86).join(r"Windows Kits\10\bin");

  let mut candidates: Vec<PathBuf> = std::fs::read_dir(&bin_root)
    .into_iter()
    .flatten()
    .flatten()
    .map(|entry| entry.path().join("arm64").join("rc.exe"))
    .filter(|path| path.is_file())
    .collect();
  candidates.sort();

  let rc = candidates
    .pop()
    .expect("could not find the Windows SDK's arm64 rc.exe");

  let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
  let out = out_dir.join("icon.res");
  let status = std::process::Command::new(&rc)
    .arg(format!("/fo{}", out.display()))
    .arg("resources/icon.ico")
    .status()
    .expect("failed to run rc.exe");
  if !status.success() {
    panic!("rc.exe failed to compile resources/icon.ico");
  }

  println!("cargo:rustc-link-arg={}", out.display());
}

#[cfg(not(windows))]
fn compile_with_sdk_rc() {
  panic!("embedding Windows resources on an ARM64 MSVC target requires a Windows host");
}
