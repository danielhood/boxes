# Future phases

**Status:** active  
**Last updated:** 2026-07-17  
**Related:** [active roadmap](active.md), [completed phases](completed.md), [initial planning](../planning/initial-planning.md), [possible next steps](../planning/next-steps.md)

Backlog phases are **not started**. Promote a row to [active.md](active.md) when it becomes current work; add or refine a spec under `docs/specs/` before implementation.

## Design backlog (pre-spec)

Ideas not yet phased — see [next-steps.md](../planning/next-steps.md) for rationale and dependencies.

| Area | Summary | Blocked by |
|------|---------|------------|
| UI | Multi-select for batch operations | Ops catalog defined |
| Design | Central theme, vitality/lose, progression, non-creature opposition | Design pass |
| Content | Derived cell types from v1 bases | Theme |
| Content | Themed world gen (replace demo seed) | Theme, cell catalog |

## P5.1 — Selection and view navigation

| Slice | Summary | Spec |
|-------|---------|------|
| P5.1 | Persistent selected cell, six-face view orbit, selection-centered zoom, pointer remap, help overlay | [P5.1-selection-view-nav.md](../specs/P5.1-selection-view-nav.md) |

**Exit criteria:** One cell always selected; views center on selection; Ctrl+arrows orbit faces; LMB select / RMB tool; inspector follows selection; `?` help overlay.

---

## P6 — Persistence

| Slice | Summary | Spec |
|-------|---------|------|
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
| P8.1 | Additional cell types and composition rules (base → derived) | _TBD: P8-cell-catalog.md_ |
| P8.2 | Throughput analytics — rates, bottlenecks, utilization | _TBD_ |
| P8.3 | Blueprint copy/paste regions | _TBD_ |
| P8.4 | Themed world generation (post-theme) | _TBD_ |

## P9 — Polish and release

| Slice | Summary | Spec |
|-------|---------|------|
| P9.1 | Linux release build + packaging | _TBD: P9-release.md_ |
| P9.2 | Options, keybindings, accessibility pass (incl. LMB/RMB remap) | _TBD_ |

## Explicitly out of scope (v1)

- Multiplayer / networking
- Player avatar / character movement
- Free-orbit 3D camera (ortho faces only for v1)
- Mobile or web targets
