# P0 — Project foundation

**Status:** draft  
**Last updated:** 2026-07-16  
**Roadmap:** [P0.1](../roadmap/active.md)  
**Related:** [initial planning](../planning/initial-planning.md)

## Goal

Establish a minimal Rust + Bevy application that runs on Linux and proves the rendering shell: window, single orthographic 3D camera, and a placeholder scene ready for chunked instancing.

## Scope

### In scope

- Cargo workspace at repo root (`boxes` binary crate)
- Bevy dependency pinned to a stable recent 0.1x release
- App plugin: clear color, orthographic `Camera3D`, basic lighting optional
- CI-friendly `cargo build` / `cargo test` (empty or smoke test)
- README at repo root with build/run instructions (update from placeholder)

### Out of scope

- Simulation crate
- Grid/chunk data structures
- UI beyond Bevy defaults
- Asset pipeline beyond primitive mesh

## Acceptance criteria

- [ ] `cargo run` opens a window on Linux
- [ ] Scene uses orthographic projection
- [ ] Project structure documents where sim vs render crates will live
- [ ] No warnings under `cargo clippy` with project `#![warn(...)]` baseline (optional for P0)

## Suggested layout

```text
boxes/
  Cargo.toml          # workspace
  crates/
    boxes_app/        # Bevy app, rendering, input (later)
    boxes_sim/        # simulation (stub crate in P0)
  docs/
```

## Notes

P0.2 (documentation) is complete via `docs/` layout and planning capture.
