# Fedora

```
sudo dnf install gcc \
  pkgconf-pkg-config \
  fontconfig-devel \
  freetype-devel \
  libxcb-devel \
  libxkbcommon-devel \
  libxkbcommon-x11-devel \
  openssl-devel \
  zstd-devel \
  vulkan-loader-devel

cargo build
```

# Ubuntu

```
sudo apt update
sudo apt install build-essential \
  pkg-config \
  libfontconfig1-dev \
  libfreetype6-dev \
  libxcb1-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libssl-dev \
  libzstd-dev \
  libvulkan-dev
cargo build
```