# P5 — Factory UI

**Status:** shipped  
**Last updated:** 2026-07-16  
**Roadmap:** [P5 — completed](../roadmap/completed.md)  
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

## Implementation notes (2026-07-16)

| Decision | Choice |
|----------|--------|
| Plugin | `FactoryUiPlugin` in `boxes_app::ui` — Bevy UI overlay |
| Sim playback | `SimPlayback` resource — pause, `step_pending` (exactly one tick), speed cycle |
| Step semantics | `step_pending` runs one `run_sim_tick` and skips accumulator; works while paused |
| Speed | `SimSpeed` multiplies frame delta before fixed-timestep accumulator (0.5× / 1× / 2×) |
| Inspector | `refresh_inspected_cell` re-reads sim each frame; `format_inspector` for panel text |
| Palette | Nine buttons set `ToolState.selected_slot` + `ActiveTool::Place`; highlight selected slot |
| Throughput HUD | Tick count, cumulative `total_cell_updates`, last-tick dirty chunk count |
| Debug overlay | Toggle shows last-tick cell updates + dirty chunks (`SimTickStats`) |
| Depth readout | HUD line from `ViewSlice` + `ActiveView` (keyboard nudge unchanged) |

### Module layout (`boxes_app`)

| Module | Role |
|--------|------|
| `ui/factory.rs` | `FactoryUiPlugin`, palette/sim buttons, panel updates |
| `ui/format.rs` | Inspector and throughput string formatters (unit-tested) |
| `sim_bridge.rs` | `SimPlayback`, `SimTickStats`, pause/speed/step in `sim_step_system` |
| `input/tools.rs` | `palette_slot_label`, `direction_label`, `reduce_mode_label` |

## Acceptance criteria

- [x] Player can select type from UI and place via P4 tools
- [x] Inspector shows live state for picked cell
- [x] Pause stops `sim.step`; step advances exactly one tick
