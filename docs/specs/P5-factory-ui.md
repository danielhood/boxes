# P5 — Factory UI

**Status:** active  
**Last updated:** 2026-07-16  
**Roadmap:** [P5 — current](../roadmap/active.md)  
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
- Replacing P4 keyboard palette (UI complements `ToolState` / `Shift`+digit)

## P4 integration (shipped)

P5 UI reads and drives existing P4 resources — do not duplicate pick/placement logic.

| P4 artifact | P5 use |
|-------------|--------|
| `InspectedCell` | Inspector panel for RMB / inspect-tool picks |
| `ToolState` + `PalettePreset` | UI palette selection; placement still via P4 pointer system |
| `ViewSlice` | Optional on-screen depth readout; slice nudge may stay on keyboard |
| Clipped `visible_surface` | Inspector/pick must match what the player sees ([P4 spec](P4-input-tools.md)) |

Default controls: [README usage](../../README.md#using-the-app).

## Acceptance criteria

- [ ] Player can select type from UI and place via P4 tools
- [ ] Inspector shows live state for picked cell
- [ ] Pause stops `sim.step`; step advances exactly one tick
