# P4 — Input and tools

**Status:** draft  
**Last updated:** 2026-07-16  
**Roadmap:** [P4](../roadmap/active.md)  
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
| `1`–`9` (with modifier) | Select cell type palette slot |

Bindings are configurable in P9.

## Acceptance criteria

- [ ] Pick hits correct cell in each ortho view (test vectors)
- [ ] Place and erase update sim and render
- [ ] View switch preserves world state
