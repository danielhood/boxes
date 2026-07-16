# Completed phases

**Status:** active  
**Last updated:** 2026-07-16  
**Related:** [active roadmap](active.md), [initial planning](../planning/initial-planning.md)

Shipped work is logged here with links to specs and merge PRs. Promote the next row from [active.md](active.md) when starting new work.

## P0 — Project foundation (shipped 2026-07-16)

| Slice | Summary | Spec | Shipped |
|-------|---------|------|---------|
| P0.1 | Rust workspace + Bevy app shell (window, ortho camera, placeholder scene) | [P0-project-foundation](../specs/P0-project-foundation.md) | [PR #2](https://github.com/danielhood/boxes/pull/2) |
| P0.2 | Documentation structure and planning baseline | — | [PR #1](https://github.com/danielhood/boxes/pull/1) |

**Exit criteria met:** `cargo run` opens a Linux window with an orthographic viewport and placeholder cube; `boxes` / `boxes_app` / `boxes_sim` workspace layout in place; `docs/` planning, roadmap, and specs conventions established; CI runs `cargo build`, `cargo test`, and `cargo clippy -- -D warnings` on Ubuntu.

**Stack pinned:** Bevy 0.16, Rust stable (via `rust-toolchain.toml`).

## P1 — Simulation core (shipped 2026-07-16)

| Slice | Summary | Spec | Shipped |
|-------|---------|------|---------|
| P1 | Sparse 32³ chunks, 8-byte `Cell`, 20 Hz tick scheduler, dirty queue + hooks | [P1-simulation-core](../specs/P1-simulation-core.md) | _PR pending_ |

**Exit criteria met:** `boxes_sim::Simulation` with `step(ticks) -> Vec<ChunkCoord>`; sparse `ChunkMap` for 500³ world; phase `(x+y+z)%8`; `SimHooks` for P2; unit + benchmark stub tests.

## P2 — Cell types (shipped 2026-07-16)

| Slice | Summary | Spec | Shipped |
|-------|---------|------|---------|
| P2 | Generator, transformer, aggregator + type-aware dirty propagation | [P2-cell-types](../specs/P2-cell-types.md) | _PR pending_ |

**Exit criteria met:** `CellEngine` wired as default `Simulation::step`; phased generators; directional transformers; sum/max aggregators; stable dirty draining; activity-scaled update tests.

## P3 — Rendering (shipped 2026-07-16)

| Slice | Summary | Spec | Shipped |
|-------|---------|------|---------|
| P3 | Chunked GPU instancing, ortho views, dirty chunk rebuild bridge | [P3-rendering](../specs/P3-rendering.md) | _PR pending_ |

**Exit criteria met:** `GridRenderPlugin` draws visible surface per ortho face; `1`/`T`, `2`/`F`, `3`/`L` switch cameras; sim 20 Hz snap-sync; incremental chunk instance rebuild.

## P4 — Input and tools (shipped 2026-07-16)

| Slice | Summary | Spec | Shipped |
|-------|---------|------|---------|
| P4 | Ray pick, place/erase/inspect, depth slice with render clip, drag placement | [P4-input-tools](../specs/P4-input-tools.md) | [PR #8](https://github.com/danielhood/boxes/pull/8) |

**Exit criteria met:** Orthographic ray pick per view (unit test vectors); place at slice depth / erase+inspect on clipped surface; `visible_surface` hides cells between slice and camera; slice nudge rebuilds render; place/erase queue chunk rebuilds; view switch is render-only; `InputPlugin` + `BUILD.md` usage docs.
