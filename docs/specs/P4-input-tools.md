# P4 — Input and tools

**Status:** shipped  
**Last updated:** 2026-07-16  
**Roadmap:** [P4 — completed](../roadmap/completed.md)  
**Related:** [P3-rendering](P3-rendering.md), [P5-factory-ui](P5-factory-ui.md)

## Goal

Mouse and keyboard interaction with the world: pick cells, place/edit types, switch orthographic views.

## Scope

### In scope

- Ray pick from active orthographic camera → `(x, y, z)` cell
- Tool modes: place type, erase (empty), inspect (read state)
- Keyboard: switch top / front / left view; optional slice offset on held axis
- Click/drag placement for factory building

### Out of scope

- Undo/redo (future)
- Blueprint paste (P8)

## Default bindings (draft)

| Input | Action |
|-------|--------|
| `1` / `2` / `3` | Top / front / left view |
| LMB | Apply active tool |
| RMB | Inspect cell |
| `E` | Erase |
| `P` | Place |
| `I` | Inspect tool |
| `Shift` + `1`–`9` | Select cell type palette slot |
| `[` / `]` | Nudge depth slice on active view axis |
| `PageUp` / `PageDown` | Nudge depth slice (alternate) |
| `-` / `=` | Nudge depth slice (alternate) |

Bindings are configurable in P9.

## Implementation notes (2026-07-16)

| Decision | Choice |
|----------|--------|
| Pick math | Pure `input/pick.rs` — ray/plane intersect → view UV → surface or slice depth |
| Surface pick | Reuses P3 `visible_surface` policy per ortho face |
| Placement target | Always at current depth slice; erase/inspect use visible surface |
| Slice feedback | Console log on nudge; alt keys `PageUp`/`PageDown`, `-`/`=` |
| Palette | Nine presets (generators, transformers, aggregators) via `Shift`+digit |
| Inspect | RMB or inspect tool; logs cell state; `InspectedCell` resource for P5 UI |
| Render bridge | `queue_rebuild_for_positions` on place/erase |
| View keys | Kept in P3 `switch_view_system`; P4 adds tools only |

## Acceptance criteria

- [x] Pick hits correct cell in each ortho view (test vectors)
- [x] Place and erase update sim and render
- [x] View switch preserves world state
