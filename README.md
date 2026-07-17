# Boxes

Single-player, offline desktop factory game for Linux. Manipulate a large 3D grid of cubes whose **state** changes over time — not their position. There is no player avatar; you build and tune fields of typed cells (generators, transformers, aggregators) viewed from orthographic faces (top, front, left).

**Status:** P5 shipped — factory UI overlay (palette, inspector, pause/step/speed) in `boxes_app`; P6 persistence is next.

## Concept

| | |
|--|--|
| **World** | 500 × 500 × 500 addressable grid (~45% non-empty target) |
| **Interaction** | Mouse and keyboard; 2D UI over 3D orthographic views |
| **Simulation** | 20 Hz discrete ticks, 8 staggered phases; event-driven neighbor updates |
| **Stack** | Rust simulation core (`boxes_sim`) + [Bevy](https://bevyengine.org/) rendering and input (`boxes_app`) |

See [docs/planning/initial-planning.md](docs/planning/initial-planning.md) for full rationale, architecture, and open decisions. [Possible next steps](docs/planning/next-steps.md) captures post-P5 design ideas (selection UX, theme, input remap, content expansion).

## Documentation

All design and delivery tracking lives under [`docs/`](docs/).

| Folder | Purpose |
|--------|---------|
| [`docs/planning/`](docs/planning/) | Vision, constraints, stack choices |
| [`docs/roadmap/`](docs/roadmap/) | Current and future phases |
| [`docs/specs/`](docs/specs/) | Per-slice implementation specs |

Start with [docs/README.md](docs/README.md) for workflow and naming conventions.

**Active work:** [docs/roadmap/active.md](docs/roadmap/active.md)

## Using the app

Run from the repo root after [setup](BUILD.md):

```bash
cargo run
```

A window opens with a seeded demo grid in an orthographic viewport. Close the window to exit.

### Views

The world is shown one orthographic face at a time, centered on the **selected cell**. The **visible surface** is the outermost cell per column at or behind the current depth slice (derived from the selection); cells between the slice and the camera are hidden.

| Input | Action |
|-------|--------|
| **Arrow keys** | Move selected cell on the face |
| **Ctrl + Arrow keys** | Rotate to an adjacent face (six orthographic views) |
| `T` | Snap to top view |
| **Ctrl + Mouse wheel** / **Ctrl + `[` / `]`** | Zoom in/out (8–64 cells visible) |
| `[` / `]` / **Mouse wheel** | Step selection along depth |

### Selection & tools

| Input | Action |
|-------|--------|
| **LMB** | Select cell (drag to track selection) |
| **RMB** | Apply the active tool on the cell under the cursor |
| **RMB drag** | Apply the active tool while dragging |
| `P` | Place tool (default) |
| `E` | Erase tool — removes the visible surface cell |
| `I` | Inspect tool — logs cell state to the console |
| `?` | Toggle help overlay (Esc to close) |

**Place** puts the selected palette cell type at the current depth slice in the column under the cursor. **Erase** clears the visible surface cell. The **inspector** always shows the selected cell.

### Depth slice

Slice depth follows the selected cell on the active view axis. Nudge with `[` / `]` or the mouse wheel before placing in a column.

| Key | Action |
|-----|--------|
| `[` | Decrease depth (toward camera) |
| `]` | Increase depth (away from camera) |

The depth axis depends on the view: **Y** in top/bottom, **Z** in front/back, **X** in left/right.

### Type palette

Hold **Shift** and press a digit to select a palette slot and switch to place mode:

| Slot | Cell type |
|------|-----------|
| `Shift`+`1` | Generator (standard, 1 s) |
| `Shift`+`2` | Generator (fast, 0.5 s) |
| `Shift`+`3` | Generator (slow, 5 s) |
| `Shift`+`4` | Transformer (+X input) |
| `Shift`+`5` | Transformer (+Z input) |
| `Shift`+`6` | Aggregator (sum) |
| `Shift`+`7` | Aggregator (max) |
| `Shift`+`8` | Transformer (−Y input) |
| `Shift`+`9` | Transformer (+Y input) |

The simulation runs continuously at 20 Hz in the background. Placed cells participate in generator, transformer, and aggregator behavior immediately.

### Factory UI

A 2D overlay complements keyboard tools:

| Control | Action |
|---------|--------|
| **Type palette** (left) | Click a slot to select cell type and switch to place mode |
| **Tool** (left) | Click Place, Erase, or Inspect to switch the active tool |
| **Inspector** (bottom left) | Live readout for the **selected** cell |
| **Pause / Resume** | Stops or resumes simulation stepping |
| **Step** | Advances exactly one simulation tick (works while paused) |
| **Speed** | Cycles sim speed: 0.5×, 1×, 2× |
| **Debug** | Toggles last-tick cell update and dirty-chunk readout |
| **Throughput HUD** (bottom right) | Current tick, cumulative cell updates, last-tick dirty chunks |
| **Depth readout** | Current depth slice (from selection) for the active view |

Press **`?`** for a full key and mouse binding reference.

Keyboard palette (`Shift`+digit) remains available alongside the UI.

Bindings will be configurable in a future release (P9).

## Build and development

See [BUILD.md](BUILD.md) for toolchain setup, Linux dependencies, and `cargo` commands.

## Repository layout

```text
boxes/
  BUILD.md                # toolchain setup, build, test, CI
  Cargo.toml              # workspace root
  rust-toolchain.toml
  crates/
    boxes/                # binary entrypoint (`cargo run`)
    boxes_app/            # Bevy app — render, input, factory UI
    boxes_sim/            # simulation kernel (sparse chunks, tick scheduler)
  docs/
    planning/
    roadmap/
    specs/
```

| Crate | Role |
|-------|------|
| `boxes` | Thin binary; wires Bevy `DefaultPlugins` + `BoxesAppPlugin` |
| `boxes_app` | Window, cameras, chunked GPU instancing, sim tick bridge, input tools, factory UI |
| `boxes_sim` | Sparse 32³ grid, 20 Hz `Simulation::step`, generator/transformer/aggregator cell types |

## License

[MIT](LICENSE) — Copyright (c) 2026 Daniel Hood
