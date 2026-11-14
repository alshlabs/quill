use std::env;

fn main() {
  println!("cargo:rerun-if-changed=resources/icon.ico");

  if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
    return;
  }

  let mut resource = winresource::WindowsResource::new();
  resource.set_icon("resources/icon.ico");
  #[cfg(windows)]
  with_sdk_rc(&mut resource);
  resource
    .compile()
    .expect("failed to embed Windows resources");
}

// winresource picks `windres` for MSVC targets on a windows-latest host and
// passes it pe-aarch64-little, which neither GNU windres nor llvm-windres can
// emit; any host's SDK `rc.exe` can, so use it when the SDK has one.
#[cfg(windows)]
fn with_sdk_rc(resource: &mut winresource::WindowsResource) {
  let program_files_x86 =
    env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
  let rc = ["x64", "arm64"].iter().find_map(|arch| {
    let rc = std::path::Path::new(&program_files_x86)
      .join(r"Windows Kits\10\bin")
      .join(env::var("WIN10_SDK_VERSION").unwrap_or_default())
      .join(arch)
      .join("rc.exe");
    rc.is_file().then_some(rc)
  });

  if let Some(rc) = rc {
    let bin = rc.parent().expect("rc.exe has no parent").to_string_lossy();
    resource.set_toolkit_path(&bin);
  }
}
