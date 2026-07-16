# Boxes

Single-player, offline desktop factory game for Linux. Manipulate a large 3D grid of cubes whose **state** changes over time — not their position. There is no player avatar; you build and tune fields of typed cells (generators, transformers, aggregators) viewed from orthographic faces (top, front, left).

**Status:** P0 foundation — Rust workspace and Bevy app shell in place.

## Concept

| | |
|--|--|
| **World** | 500 × 500 × 500 addressable grid (~45% non-empty target) |
| **Interaction** | Mouse and keyboard; 2D UI over 3D orthographic views |
| **Simulation** | 20 Hz discrete ticks, 8 staggered phases; event-driven neighbor updates |
| **Stack** | Rust simulation core (`boxes_sim`) + [Bevy](https://bevyengine.org/) rendering and input (`boxes_app`) |

See [docs/planning/initial-planning.md](docs/planning/initial-planning.md) for full rationale, architecture, and open decisions.

## Documentation

All design and delivery tracking lives under [`docs/`](docs/).

| Folder | Purpose |
|--------|---------|
| [`docs/planning/`](docs/planning/) | Vision, constraints, stack choices |
| [`docs/roadmap/`](docs/roadmap/) | Current and future phases |
| [`docs/specs/`](docs/specs/) | Per-slice implementation specs |

Start with [docs/README.md](docs/README.md) for workflow and naming conventions.

**Active work:** [docs/roadmap/active.md](docs/roadmap/active.md)

## Requirements

- **Rust:** stable toolchain (see [`rust-toolchain.toml`](rust-toolchain.toml))
- **Linux:** system libraries for Bevy/winit (Ubuntu/Debian example):

```bash
sudo apt-get install -y \
  libasound2-dev \
  libudev-dev \
  libwayland-dev \
  libxkbcommon-dev \
  libegl1-mesa-dev \
  libgles2-mesa-dev
```

## Building and running

From the repo root:

```bash
cargo run
```

This opens a window with an orthographic 3D viewport and a placeholder cube. Close the window or press the OS close shortcut to exit.

Other useful commands:

```bash
cargo build          # debug build
cargo test           # workspace smoke tests
cargo clippy         # lint (CI runs with -D warnings)
```

## Repository layout

```text
boxes/
  Cargo.toml              # workspace root
  rust-toolchain.toml
  crates/
    boxes/                # binary entrypoint (`cargo run`)
    boxes_app/            # Bevy app — render, input, UI (later)
    boxes_sim/            # simulation kernel (stub until P1)
  docs/
    planning/
    roadmap/
    specs/
```

| Crate | Role |
|-------|------|
| `boxes` | Thin binary; wires Bevy `DefaultPlugins` + `BoxesAppPlugin` |
| `boxes_app` | Window, cameras, scene, and future input/UI |
| `boxes_sim` | Grid, cell types, tick scheduler — no Bevy dependency |

## License

[MIT](LICENSE) — Copyright (c) 2026 Daniel Hood
