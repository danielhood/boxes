# Building Boxes

How to set up a Linux dev environment, build, test, and run the Boxes workspace.

Tested on Ubuntu/Debian Linux. Other distros need equivalent Bevy/winit dev packages.

## Local environment setup

### 1. Rust toolchain

Install [rustup](https://rustup.rs/) if you do not already have `cargo`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

The repo pins stable via [`rust-toolchain.toml`](rust-toolchain.toml); the first `cargo` command in this directory will install that toolchain automatically.

Check:

```bash
rustc --version
cargo --version
```

### 2. Linux system libraries

Bevy links against ALSA, Wayland/X11, and Mesa. On Ubuntu/Debian:

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libasound2-dev \
  libudev-dev \
  libwayland-dev \
  libxkbcommon-dev \
  libegl1-mesa-dev \
  libgles2-mesa-dev
```

`pkg-config` is required for native dependency discovery; without it, `cargo build` fails on crates such as `alsa-sys` and `wayland-sys`.

### 3. Verify the workspace

From the repo root:

```bash
cargo build --workspace
cargo test --workspace
cargo run
```

`cargo run` opens a window with the seeded demo grid. See [README.md](README.md#using-the-app) for controls.

## Building and running

From the repo root after setup:

```bash
cargo run
```

Other useful commands:

```bash
cargo build          # debug build
cargo test           # workspace smoke tests
cargo clippy         # lint (CI runs with -D warnings)
```

CI (`.github/workflows/ci.yml`) runs on every push and PR: `cargo build`, `cargo test`, and `cargo clippy -- -D warnings` on Ubuntu with Bevy system libraries installed.
