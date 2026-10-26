fn main() {
  println!("cargo:rerun-if-changed=resources/icon.ico");

  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
    winresource::WindowsResource::new()
      .set_icon("resources/icon.ico")
      .compile()
      .expect("failed to embed Windows resources");
  }
}
