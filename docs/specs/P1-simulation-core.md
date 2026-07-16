# P1 — Simulation core

**Status:** shipped  
**Last updated:** 2026-07-16  
**Roadmap:** [P1 — completed](../roadmap/completed.md)  
**Related:** [initial planning](../planning/initial-planning.md), [P2-cell-types](P2-cell-types.md)

## Goal

Implement the engine-agnostic simulation kernel: sparse chunked storage for a 500³ addressable world, 20 Hz tick loop, 8-phase stagger for periodic work, and dirty queues for event-driven updates.

## Parameters (from planning)

| Parameter | Value |
|-----------|--------|
| World extent | 500 × 500 × 500 |
| Target fill | ~45% non-empty |
| Tick rate | 20 Hz |
| Phases | 8 |
| `max_steps_per_frame` | 2 (initial) |

## Scope

### In scope

- `boxes_sim` crate with no Bevy dependency
- `ChunkCoord`, `Chunk` (proposed 32³ cells per chunk)
- Sparse `ChunkMap` — allocate chunks on first non-default cell
- Compact `Cell` record (type id, state, flags; exact layout TBD in implementation)
- Global tick counter `T`, fixed `Δt`
- Phase: `(x + y + z) % 8`
- Dirty cell queue (chunk-local or global with chunk affinity)
- Worker-ready API: `sim.step(ticks)` returns dirty chunk list
- Unit tests: phase schedule, chunk index math, dirty propagation stub

### Out of scope

- Concrete generator/transformer/aggregator logic (P2)
- Rendering integration (P3)
- Persistence (P6)

## Cell record (draft)

Target **8–16 bytes** per cell. Example layout (subject to implementation):

| Field | Size | Notes |
|-------|------|-------|
| `type_id` | u8 | 0 = empty |
| `flags` | u8 | dirty, pinned, etc. |
| `state` | u16 | type-specific payload |
| `reserved` | u32 | alignment / future timer |

Implemented as 8-byte `Cell` in `boxes_sim::cell`.

Empty cells should be representable without heap allocation per cell (default sentinel in dense chunk array).

## Tick algorithm

For each tick `T`:

1. **Drain dirty queue** (cap: `max_dirty_drain_per_tick`) — P2 attaches behavior; P1 invokes trait or callback stub.
2. **Schedule generators** — P1 exposes hook `on_phase_tick(T, phase)`; full logic in P2.
3. Collect **dirty chunks** for consumers.
4. Enforce **max_cell_updates_per_tick**.

## Acceptance criteria

- [x] Indexing: world `(x,y,z)` maps to chunk + local index; bounds 0..499
- [x] Sparse allocation: empty region uses no chunk memory
- [x] `step(1)` advances `T` and is deterministic given same inputs
- [x] Phase function matches `(x+y+z) % 8`
- [x] Tests cover chunk boundary neighbors
- [x] Benchmark stub: iterate 1M cell writes + 1000 ticks (no hard perf gate yet)

## Open questions

- Chunk size 32³ vs 64³ (default **32³** for parallelism granularity)
- Global vs per-chunk dirty queues
