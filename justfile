build:
  cargo build

build_release:
  cargo build --release

run:
  cargo build && QUILL_CONFIG_AUTO_RESOLVE=true ./target/debug/quill

run_release:
  cargo build --release && QUILL_CONFIG_AUTO_RESOLVE=true ./target/debug/quill

watch:
  watchexec -e rs -r -- 'cargo build && env QUILL_CONFIG_AUTO_RESOLVE=true ./target/debug/quill'

watch_release:
  watchexec -e rs -r -- 'cargo build --release && env QUILL_CONFIG_AUTO_RESOLVE=true ./target/debug/quill'
