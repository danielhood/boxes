# P3 — Rendering

**Status:** draft  
**Last updated:** 2026-07-16  
**Roadmap:** [P3](../roadmap/active.md)  
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

## Performance targets (initial)

| View | Max instances | Notes |
|------|---------------|-------|
| Single ortho face | ~250k | 500×500 |

Rebuild cost proportional to **dirty chunks**, not world size.

## Acceptance criteria

- [ ] Switching views changes active orthographic camera
- [ ] Placing/updating cells in sim reflects in render within 1 sim tick
- [ ] Dirty chunk not rendered unchanged does not rebuild buffer
- [ ] Stable 60 FPS with modest test world (e.g. 64³ active region) on dev hardware
