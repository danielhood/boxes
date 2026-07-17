# Initial planning — Boxes

**Status:** active  
**Last updated:** 2026-07-17  
**Related:** [roadmap/active.md](../roadmap/active.md), [roadmap/completed.md](../roadmap/completed.md), [roadmap/future.md](../roadmap/future.md)

## Implementation status

| Phase | Status | Notes |
|-------|--------|-------|
| P0 — Project foundation | **shipped** (2026-07-16) | [completed log](../roadmap/completed.md) — Bevy 0.16 app shell, CI, docs layout |
| P1 — Simulation core | **shipped** (2026-07-16) | [completed log](../roadmap/completed.md) — `boxes_sim` kernel |
| P2 — Cell types | **shipped** (2026-07-16) | [completed log](../roadmap/completed.md) — generator / transformer / aggregator |
| P3 — Rendering | **shipped** (2026-07-16) | [completed log](../roadmap/completed.md) — chunked instancing + ortho views |
| P4 — Input + tools | **shipped** (2026-07-16) | [completed log](../roadmap/completed.md) — `InputPlugin`, slice clip render, [PR #8](https://github.com/danielhood/boxes/pull/8) |
| P5 — Factory UI | **shipped** (2026-07-16) | [completed log](../roadmap/completed.md) — `FactoryUiPlugin`, sim playback controls |
| P6 — Persistence | **active** | [active roadmap](../roadmap/active.md) |
| P7+ | backlog | [future roadmap](../roadmap/future.md) |

## Summary

**Boxes** is a single-player, offline desktop game (Linux-first) where the player manipulates a large 3D grid of cubes. Cubes change **state**, not position. There is no player avatar and no moving characters. Interaction is mouse and keyboard. The UI is 2D; the world is shown as axis-aligned cubes from one orthographic perspective at a time (top, front, left, etc.).

Gameplay is **factory-oriented**: long-running fields of typed cells that execute state changes in the background, with throughput and topology mattering more than instant puzzle chains.

## Product constraints

| Constraint | Decision |
|------------|----------|
| Platform | Desktop; Linux compatibility required |
| Multiplayer | None — single player, offline |
| Player representation | None — direct manipulation of the world |
| Motion | No character or cube movement; state-only updates |
| View | One orthographic face at a time; switchable |
| Input | Mouse + keyboard |
| World size (addressable) | 500 × 500 × 500 |
| Fill rate (initial target) | ~45% non-empty cells (~56M active cells) |

## Core mechanics

- Each cell has a **type** and **state**.
- Types execute behavior independently in the background, e.g.:
  - **Generators** — periodic output on a schedule
  - **Transformers** — react when input neighbors change
  - **Aggregators** — combine neighbor states into own state
- Interactions are **local** (neighbors / neighborhood rules), not physics-based.
- The player builds and tunes layouts for steady-state throughput.

## Tech stack (recommended)

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Simulation | **Rust** | Chunked grid, parallel workers, 56M-scale incremental updates |
| Rendering + input | **Bevy** | Orthographic 3D, GPU instancing, desktop/Linux, single codebase |
| UI | **Bevy UI** or **egui** | 2D overlays, inspectors, factory metrics |
| Storage model | **Sparse chunks** (e.g. 32³) | ~45% fill; avoid allocating empty space |
| Rendering strategy | **View-dependent instancing** | Only visible subset per ortho face; chunk-dirty GPU buffer updates |

### Rejected or deferred

| Option | Why not (for v1) |
|--------|------------------|
| Godot-only (GDScript sim) | Hot path at 56M scale needs native sim; GDExtension adds dual-runtime cost |
| One entity per cell (ECS/scene graph) | 125M entities not viable |
| Full-grid scan each step | CPU-prohibitive at target scale |
| Continuous per-cell timers (primary model) | Ordering/determinism harder; neighbor logic wants phased reads/writes |
| 60 Hz simulation | Wasteful for factory tempo; decouple sim from display refresh |

## Simulation model

### Time: discrete ticks (not continuous timers)

- **Tick rate:** 20 Hz (`Δt_sim = 50 ms`)
- **Stagger phases:** 8 (`P = 8`)
- **Style:** Factory — background throughput over click-instant chains

Continuous timers were considered; discrete ticks won for determinism, debuggability, neighbor semantics, and chunk-parallel batching. Generator rate variety is expressed as `period_ticks` (seconds × 20).

### Scheduling rules

**Phase assignment** (fixed, deterministic):

```text
phase = (x + y + z) % 8
```

**Generator** with period `N` fires when:

```text
(T + phase) % N == 0
```

**Transformers / aggregators:** event-driven — run only when **dirty** (neighbor/input changed), not on a global phase schedule.

**Per render frame:**

```text
steps = min(floor(elapsed * 20), max_steps_per_frame)   // initial cap: 2
```

### Tick pipeline (each sim tick T)

1. Drain **dirty queue** — transformers and aggregators (priority).
2. Run **phase-gated generators** matching `(T + phase) % period_ticks == 0`.
3. **Propagate** — enqueue neighbor dirties; mark render chunks dirty.
4. Enforce **per-tick update budget**; defer overflow to later ticks.

### Initial budgets (to tune in profiling)

| Parameter | Starting value |
|-----------|----------------|
| `max_steps_per_frame` | 2 |
| `max_cell_updates_per_tick` | 100k–250k |
| `max_dirty_drain_per_tick` | ~50% of update budget |

### Factory timing reference (20 Hz)

| Real time | `period_ticks` |
|-----------|----------------|
| 0.25 s | 5 |
| 0.5 s | 10 |
| 1 s | 20 |
| 2 s | 40 |
| 5 s | 100 |

Typical factory cycles: **10–100 ticks** (0.5–5 s). With 8 phases, same-period generators spread peak load ≈ **1/8** per tick on average.

## Architecture

```text
┌──────────────────────────────────────────────────┐
│ UI (2D): tools, inspectors, view switcher        │
├──────────────────────────────────────────────────┤
│ Renderer: ortho cameras → GPU instanced cubes    │  visible subset only
├──────────────────────────────────────────────────┤
│ Simulation (Rust):                               │
│   ChunkMap (sparse 32³ chunks)                     │
│   CellType registry (generator/transform/agg)    │
│   Dirty queues + phase scheduler                 │
│   Worker pool                                    │
└──────────────────────────────────────────────────┘
```

- **Simulation ≠ rendering** — grid model is source of truth; view reads dirty chunks.
- **ECS thinking at chunk level** — not one Bevy entity per cell.
- **Orthographic views** — top/front/left cameras; switch active camera on input.
- **Mouse pick** — raycast → cell index; tools apply state/type changes.

## Scale notes

| Metric | Value |
|--------|-------|
| Addressable cells | 125,000,000 |
| Non-empty (45%) | ~56,250,000 |
| Dense 1-byte state (if allocated) | ~125 MB minimum |
| Richer cell (8–16 B) | ~0.5–2 GB |

Success depends on **incremental updates** (dirty/event queues), not scanning all non-empty cells each tick.

## Rendering notes

- One ortho face ≈ up to **500 × 500 = 250k** visible cells — feasible with GPU instancing.
- On cell change: mark chunk dirty → rebuild that chunk’s instance buffer only.
- Visual refresh can run at display Hz; sim stays at 20 Hz.

## Open decisions (next specs)

Resolved in P1–P5 specs where noted. Still open:

### Technical

- **Save/load** format (compressed chunk blobs) — P6
- **Bevy UI vs egui** for heavy analytics — factory overlay uses Bevy UI (P5)

### Product & theme

Captured in detail in [next-steps.md](next-steps.md):

- **Central game theme** — fiction, tone, visual identity; current build is unthemed factory nav
- **Vitality / lose condition** — environment should stay “alive”; without player interaction it tends to “die” (exact rules TBD)
- **Progression** — how sessions or campaigns advance; win state TBD
- **Opposition model** — how the system pushes back **without moving creatures** (field decay, competing subsystems, etc.)
- **Selection UX** — always-one selected cell; views center on selection; future multi-select for batch ops
- **Pointer remap** — LMB select; RMB context menu vs tool-apply (TBD)
- **Cell catalog** — expand beyond v1 base types; derived types and themed world gen after theme lock

## Glossary

| Term | Meaning |
|------|---------|
| **Tick** | Fixed simulation step at 20 Hz |
| **Phase** | Stagger bucket 0..7 to spread periodic generator work |
| **Dirty** | Cell needs re-evaluation due to local change |
| **Chunk** | Fixed-size subvolume (e.g. 32³) for storage and parallelism |
