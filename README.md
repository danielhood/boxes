# Boxes

Single-player, offline desktop factory game for Linux. Manipulate a large 3D grid of cubes whose **state** changes over time — not their position. There is no player avatar; you build and tune fields of typed cells (generators, transformers, aggregators) viewed from orthographic faces (top, front, left).

**Status:** Early planning — documentation and roadmap in place; implementation not started.

## Concept

| | |
|--|--|
| **World** | 500 × 500 × 500 addressable grid (~45% non-empty target) |
| **Interaction** | Mouse and keyboard; 2D UI over 3D orthographic views |
| **Simulation** | 20 Hz discrete ticks, 8 staggered phases; event-driven neighbor updates |
| **Stack (planned)** | Rust simulation core + [Bevy](https://bevyengine.org/) rendering and input |

See [docs/planning/initial-planning.md](docs/planning/initial-planning.md) for full rationale, architecture, and open decisions.

## Documentation

All design and delivery tracking lives under [`docs/`](docs/).

| Folder | Purpose |
|--------|---------|
| [`docs/planning/`](docs/planning/) | Vision, constraints, stack choices |
| [`docs/roadmap/`](docs/roadmap/) | Current and future phases |
| [`docs/specs/`](docs/specs/) | Per-slice implementation specs |

Start with [docs/README.md](docs/README.md) for workflow and naming conventions.

**Active work:** [docs/roadmap/active.md](docs/roadmap/active.md) — phase P0 (project foundation) is next.

## Building

The Rust workspace is not scaffolded yet. After [P0 — Project foundation](docs/specs/P0-project-foundation.md) lands:

```bash
cargo run
```

Requirements will be documented in that spec (Rust toolchain, Linux dependencies for Bevy).

## Repository layout (target)

```text
boxes/
  Cargo.toml           # workspace (planned)
  crates/
    boxes_app/         # Bevy app — render, input, UI
    boxes_sim/         # simulation kernel (no Bevy)
  docs/
    planning/
    roadmap/
    specs/
```

## License

[MIT](LICENSE) — Copyright (c) 2026 Daniel Hood
