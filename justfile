build:
  cargo build

run:
  cargo build && QUILL_CONFIG_AUTO_RESOLVE=true ./target/debug/quill

watch:
  watchexec -e rs -r -- 'cargo build && QUILL_CONFIG_AUTO_RESOLVE=true ./target/debug/quill'
