#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP_NAME="Quill"
BIN_NAME="quill"
ARCH="$(uname -m)"
TARGET="${TARGET:-}"
ICON="${ICON:-$ROOT/resources/icon.svg}"
ICON_EXT="${ICON##*.}"

if [ -n "$TARGET" ]; then
  cargo build --release --target "$TARGET"
  RELEASE_DIR="$ROOT/target/$TARGET/release"
  ARCH="${TARGET%%-*}"
else
  cargo build --release
  RELEASE_DIR="$ROOT/target/release"
fi

TOOLS_DIR="$ROOT/target/appimage-tools"
LINUXDEPLOY="$TOOLS_DIR/linuxdeploy-$ARCH.AppImage"
if [ ! -x "$LINUXDEPLOY" ]; then
  mkdir -p "$TOOLS_DIR"
  curl -fsSL -o "$LINUXDEPLOY" \
    "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-$ARCH.AppImage"
  chmod +x "$LINUXDEPLOY"
fi

BUILD_DIR="$RELEASE_DIR/appimage"
APP_DIR="$BUILD_DIR/$APP_NAME.AppDir"
rm -rf "$BUILD_DIR"
mkdir -p "$APP_DIR"

cat > "$BUILD_DIR/$BIN_NAME.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Exec=$BIN_NAME
Icon=$BIN_NAME
Categories=Development;Utility;
Terminal=false
EOF

cp "$ICON" "$BUILD_DIR/$BIN_NAME.$ICON_EXT"

OUTPUT="$RELEASE_DIR/$APP_NAME.AppImage"
rm -f "$OUTPUT"

# Extract-and-run avoids needing FUSE on the build host (CI runners and containers lack it).
APPIMAGE_EXTRACT_AND_RUN=1 \
ARCH="$ARCH" \
LDAI_OUTPUT="$OUTPUT" \
  "$LINUXDEPLOY" \
    --appdir "$APP_DIR" \
    --executable "$RELEASE_DIR/$BIN_NAME" \
    --desktop-file "$BUILD_DIR/$BIN_NAME.desktop" \
    --icon-file "$BUILD_DIR/$BIN_NAME.$ICON_EXT" \
    --output appimage

echo "$OUTPUT"
