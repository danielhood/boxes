# P2 — Cell types

**Status:** shipped  
**Last updated:** 2026-07-16  
**Roadmap:** [P2 — completed](../roadmap/completed.md)  
**Related:** [P1-simulation-core](P1-simulation-core.md)

## Goal

Implement the first three cell behaviors and wire them into the P1 scheduler: generators (periodic), transformers (neighbor-input), aggregators (neighbor-reduce).

## Type catalog (v1)

| Type | Trigger | Behavior (summary) |
|------|---------|-------------------|
| **Empty** | — | No op; default cell |
| **Generator** | `(T + phase) % period_ticks == 0` | Sets/emits configured output state |
| **Transformer** | Dirty when any input neighbor changes | Maps input neighbor pattern → new self state |
| **Aggregator** | Dirty when any neighbor changes | Reduces neighbor states (e.g. sum, max, vote) → self state |

Exact state machines and neighbor masks are defined during implementation; keep rules **local** and **deterministic**.

## Scope

### In scope

- `CellType` registry in `boxes_sim`
- Per-type `period_ticks` for generators (config table or embedded in type def)
- Dirty propagation: on state change, enqueue affected neighbors per type rules
- Cycle policy: document read/write phase (read neighbors at T, write at end of tick)
- Tests: isolated 3×3×3 chunk fixtures, generator stagger, transformer chain

### Out of scope

- Visual representation of states (P3)
- Player placement tools (P4)
- Additional types beyond the four above

## Acceptance criteria

- [x] Generator fires on correct ticks for all 8 phases
- [x] Transformer updates only when input neighbors change
- [x] Aggregator matches specified reduce rule in tests
- [x] No infinite dirty loop on stable configuration
- [x] Update count scales with activity, not total non-empty cells (instrumented test)

## Cell encoding (implemented)

| Type | `state` | `reserved` |
|------|---------|------------|
| Generator | output pulse counter | `period_ticks` |
| Transformer | copied value | input `Direction` (0–5) |
| Aggregator | reduced value | `ReduceMode` (`Sum` / `Max`) |

## Read/write cycle

Neighbors are read at the start of a dirty-cell evaluation; the cell writes once at the end of that evaluation. Dirty propagation to transformers/aggregators runs only when `state` or `type_id` changes.

## Factory tuning defaults

| Generator profile | `period_ticks` |
|-------------------|----------------|
| Fast | 10 (0.5 s) |
| Standard | 20 (1 s) |
| Slow | 100 (5 s) |
