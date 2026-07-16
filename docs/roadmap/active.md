# Active roadmap

**Status:** active  
**Last updated:** 2026-07-16  
**Related:** [initial planning](../planning/initial-planning.md), [future phases](future.md)

Phases listed here are **in progress** or **up next**. When a phase ships, move its row to [future.md](future.md) (if deferred) or a completed log; link the implementing spec from the Spec column.

## Current phase: P0 — Project foundation

| Slice | Summary | Spec | Status |
|-------|---------|------|--------|
| P0.1 | Rust workspace + Bevy app shell (window, ortho camera, empty scene) | [P0-project-foundation](../specs/P0-project-foundation.md) | in review |
| P0.2 | Documentation structure and planning baseline | — | **done** (this repo docs layout) |

**Exit criteria for P0:** Bevy binary runs on Linux; one orthographic viewport; docs/roadmap and docs/specs conventions in use.

---

## Up next (still active queue)

| Phase | Summary | Spec | Status |
|-------|---------|------|--------|
| P1 | Simulation core — sparse chunks, cell storage, 20 Hz / 8-phase scheduler | [P1-simulation-core](../specs/P1-simulation-core.md) | not started |
| P2 | Cell types — generator, transformer, aggregator + dirty propagation | [P2-cell-types](../specs/P2-cell-types.md) | not started |
| P3 | Rendering — chunked GPU instancing, ortho view switching, dirty buffer updates | [P3-rendering](../specs/P3-rendering.md) | not started |
| P4 | Input + tools — mouse pick, place/edit cells, keyboard view switch | [P4-input-tools](../specs/P4-input-tools.md) | not started |
| P5 | Factory UI — metrics, cell inspector, type palette | [P5-factory-ui](../specs/P5-factory-ui.md) | not started |

Implement in order unless a spec explicitly notes a dependency exception.
