# P4 — Input and tools

**Status:** shipped  
**Last updated:** 2026-07-16  
**Roadmap:** [P4 — completed](../roadmap/completed.md)  
**Related:** [P3-rendering](P3-rendering.md), [P5-factory-ui](P5-factory-ui.md), [README usage](../../README.md#using-the-app)

## Goal

Mouse and keyboard interaction with the world: pick cells, place/edit types, switch orthographic views.

## Scope

### In scope

- Ray pick from active orthographic camera → `(x, y, z)` cell
- Tool modes: place type, erase (empty), inspect (read state)
- Keyboard: switch top / front / left view; depth slice on the axis perpendicular to the active face
- Click/drag placement for factory building
- Render clips cells between the current slice and the camera so the active layer is visible

### Out of scope

- Undo/redo (future)
- Blueprint paste (P8)
- On-screen slice plane / empty-column ghost (P5 UI may add inspector readout)

## Default bindings

| Input | Action |
|-------|--------|
| `1` / `T` | Top view |
| `2` / `F` | Front view |
| `3` / `L` | Left view |
| LMB | Apply active tool |
| LMB drag | Apply active tool on each new cell under the cursor |
| RMB | Inspect cell |
| `P` | Place tool (default) |
| `E` | Erase tool |
| `I` | Inspect tool |
| `Shift` + `1`–`9` | Select cell type palette slot (switches to place) |
| `[` / `]` | Nudge depth slice on active view axis |
| `PageUp` / `PageDown` | Nudge depth slice (alternate) |
| `-` / `=` | Nudge depth slice (alternate) |

View switch keys remain in P3 `switch_view_system`. User-facing copy lives in [README.md](../../README.md#using-the-app). Build/setup is in [BUILD.md](../../BUILD.md).

Bindings are configurable in P9.

## Implementation notes (2026-07-16)

| Decision | Choice |
|----------|--------|
| Plugin | `InputPlugin` in `boxes_app` — keyboard tools, pointer pick, slice nudge |
| Pick math | Pure `input/pick.rs` — ray/plane intersect → view UV → surface or slice depth |
| Surface pick | Reuses P3 `visible_surface` per ortho face, **with slice clipping** (see below) |
| Placement target | Always at current depth slice (`pick_slice_cell`); erase/inspect use clipped surface |
| Slice default | `ViewSlice` per view axis initialized to world midpoint (`250` on 500³ grid) |
| Slice clip (render) | `visible_surface(sim, view, slice_depth)` hides cells between slice and camera, then applies P3 surface rule on the remainder |
| Slice clip (per view) | **Top/Front:** keep cells with depth `≤ slice`; **Left:** keep cells with depth `≥ slice` |
| Slice rebuild | `mark_slice_change` queues full chunk rebuild when depth changes |
| Slice feedback | `info!` log on nudge (`depth slice Y=251 (Top)`); alt keys for layouts where `[`/`]` fail |
| Palette | Nine presets (generators, transformers, aggregators) via `Shift`+digit |
| Inspect | RMB or inspect tool; logs cell state; `InspectedCell` resource for P5 UI |
| Render bridge | `queue_rebuild_for_positions` on place/erase |
| View keys | Kept in P3 `switch_view_system`; P4 adds tools only |

### Module layout (`boxes_app`)

| Module | Role |
|--------|------|
| `input/pick.rs` | Orthographic ray pick, UV mapping, slice vs surface resolution (unit-tested) |
| `input/tools.rs` | `ToolState`, `ViewSlice`, `PalettePreset`, `slice_nudge_delta` |
| `input/mod.rs` | Bevy systems: tool keys, slice keys, pointer apply/drag |
| `render/surface.rs` | `is_cell_visible_at_slice`, `visible_surface(..., slice_depth)` |
| `render/chunk.rs` | `mark_slice_change` + rebuild with current slice |
| `sim_bridge.rs` | `queue_rebuild_for_positions` for editor writes |

### Depth slice behavior

1. Player nudges slice along the axis perpendicular to the active orthographic face (`Y` top, `Z` front, `X` left).
2. Render hides voxels **between** the slice plane and the camera so the edited layer is the visible face.
3. Place writes the selected palette cell at `(u, v, slice_depth)` for the column under the cursor.
4. Erase removes the **clipped** visible surface cell at that column (not necessarily at slice depth if a higher layer remains visible behind the clip).

## Acceptance criteria

- [x] Pick hits correct cell in each ortho view (test vectors in `input/pick.rs`)
- [x] Place and erase update sim and render
- [x] View switch preserves world state
- [x] Depth slice nudge changes visible layer (cells between slice and camera hidden)
- [x] Place targets current slice depth; erase/inspect target clipped surface
