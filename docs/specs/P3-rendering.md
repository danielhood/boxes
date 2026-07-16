# P3 — Rendering

**Status:** shipped  
**Last updated:** 2026-07-16  
**Roadmap:** [P3 — completed](../roadmap/completed.md)  
**Related:** [P1-simulation-core](P1-simulation-core.md), [P4-input-tools](P4-input-tools.md)

## Goal

Draw the visible subset of the grid from orthographic cameras using GPU instancing, updating only chunks marked dirty by the simulation.

## Scope

### In scope

- One cube mesh + instanced draws per visible chunk
- Orthographic cameras: **top**, **front**, **left** (active camera switchable)
- Top view: configurable slice or topmost-non-empty column policy (pick one in implementation; document in spec revision)
- Material/color per cell state or type
- Bridge: `boxes_sim` dirty chunk list → instance buffer rebuild
- Sim runs at 20 Hz; render interpolates or snaps on sim step (document choice)

### Out of scope

- Full 125M instance buffer
- Free camera orbit
- Advanced lighting / PBR

## Implementation notes (2026-07-16)

| Decision | Choice |
|----------|--------|
| Instancing | Bevy 0.16 automatic instancing — shared `Mesh3d` + per-type `StandardMaterial` |
| View surface | **Top:** max Y per (x,z); **Front:** max Z per (x,y); **Left:** min X per (y,z) |
| Sim/render sync | **Snap** on sim step (no interpolation) |
| Sim rate | 20 Hz fixed timestep, `max_steps_per_frame = 2` |
| View switch | `1`/`T` top, `2`/`F` front, `3`/`L` left |
| Dirty rebuild | View-dependent column expansion; only queued chunk coords rebuild instance entities |
| Demo world | ~64² footprint seeded near grid center on startup |

## Performance targets (initial)

| View | Max instances | Notes |
|------|---------------|-------|
| Single ortho face | ~250k | 500×500 |

Rebuild cost proportional to **dirty chunks** (expanded to view columns), not world size.

## Acceptance criteria

- [x] Switching views changes active orthographic camera
- [x] Placing/updating cells in sim reflects in render within 1 sim tick
- [x] Dirty chunk not rendered unchanged does not rebuild buffer
- [x] Stable 60 FPS with modest test world (e.g. 64³ active region) on dev hardware — demo seeds ~64² columns; profile on target GPU
