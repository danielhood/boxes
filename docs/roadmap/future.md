# Future phases

**Status:** active  
**Last updated:** 2026-07-16  
**Related:** [active roadmap](active.md), [initial planning](../planning/initial-planning.md)

Backlog phases are **not started**. Promote a row to [active.md](active.md) when it becomes current work; add or refine a spec under `docs/specs/` before implementation.

## P6 — Persistence

| Slice | Summary | Spec |
|-------|---------|------|
| P6.1 | Save/load world — compressed chunk blobs, metadata header | _TBD: P6-persistence.md_ |
| P6.2 | Autosave + named snapshots | _TBD_ |

## P7 — Scale and performance

| Slice | Summary | Spec |
|-------|---------|------|
| P7.1 | Profiling harness — synthetic 1M / 10M active cell loads | _TBD: P7-performance.md_ |
| P7.2 | Tune update budgets; optional phase increase (8 → 16) | _TBD_ |
| P7.3 | Sim catch-up / slow-motion under overload | _TBD_ |

## P8 — Factory depth

| Slice | Summary | Spec |
|-------|---------|------|
| P8.1 | Additional cell types and composition rules | _TBD: P8-cell-catalog.md_ |
| P8.2 | Throughput analytics — rates, bottlenecks, utilization | _TBD_ |
| P8.3 | Blueprint copy/paste regions | _TBD_ |

## P9 — Polish and release

| Slice | Summary | Spec |
|-------|---------|------|
| P9.1 | Linux release build + packaging | _TBD: P9-release.md_ |
| P9.2 | Options, keybindings, accessibility pass | _TBD_ |

## Explicitly out of scope (v1)

- Multiplayer / networking
- Player avatar / character movement
- Free-orbit 3D camera (ortho faces only for v1)
- Mobile or web targets
- Live Scryfall-style external sync (N/A — not applicable to Boxes)
