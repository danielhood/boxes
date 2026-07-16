# P5 — Factory UI

**Status:** draft  
**Last updated:** 2026-07-16  
**Roadmap:** [P5](../roadmap/active.md)  
**Related:** [P4-input-tools](P4-input-tools.md), [initial planning](../planning/initial-planning.md)

## Goal

2D UI overlay for factory play: cell type palette, inspector, and basic throughput/readout so the player optimizes layouts over time.

## Scope

### In scope

- Type palette (generator / transformer / aggregator)
- Cell inspector: coordinates, type, state, `period_ticks` for generators
- Sim controls: pause, single-step tick, sim speed (0.5×, 1×, 2×)
- Chunk-level debug overlay (optional toggle): dirty count, updates last tick

### Out of scope

- Full analytics dashboard (P8)
- Theming / art pass

## Acceptance criteria

- [ ] Player can select type from UI and place via P4 tools
- [ ] Inspector shows live state for picked cell
- [ ] Pause stops `sim.step`; step advances exactly one tick
