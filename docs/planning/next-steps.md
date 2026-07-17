# Possible next steps

**Status:** active  
**Last updated:** 2026-07-17  
**Related:** [initial planning](initial-planning.md), [future roadmap](../roadmap/future.md), [P5 factory UI spec](../specs/P5-factory-ui.md)

Exploratory notes captured after P5 shipped. **Not scheduled** — promote items into [active.md](../roadmap/active.md) and a `docs/specs/` file before implementation.

---

## UI refinement — selection model

**Spec:** [P5.1 — Selection and view navigation](../specs/P5.1-selection-view-nav.md) (shipped 2026-07-17)

### Always-selected cell

- Maintain **one cell always selected** in the UI (distinct from transient inspect readout).
- **All orthographic views center on the selected cell** when switching views or when selection changes.
- Implies a persistent `SelectedCell` (or similar) resource, camera framing per view, and inspector/panel binding to selection rather than only RMB picks.
- Initial selection: random non-empty cell from demo world gen.

### Multi-select (later)

- Introduce **multi-select** for batch operations once those operations are defined (copy region, bulk type change, delete column, etc.).
- Depends on a clear ops catalog; selection UX should not block single-select + view-centering work.

---

## Input remap — pointer roles

**Resolved in [P5.1 spec](../specs/P5.1-selection-view-nav.md):**

| Pointer | Role |
|---------|------|
| **LMB** | **Select** cell (update persistent selection; primary navigation) |
| **RMB** | **Apply active tool** (place / erase / inspect mode) |

Context menu on RMB is deferred. View switching moves to **Ctrl + arrow keys**; `T` snaps to Top. Slice step on `[` / `]` / wheel; zoom on **Ctrl + wheel**. Help overlay on `?`.

---

## Game theme, progression, and opposition (open)

Today the app is a **cell-based factory editor** with no central theme, win/lose conditions, or progression arc. The following direction is under consideration — **not decided**.

### Core fantasy (draft)

- The player tends a large **environment** that must stay **alive**.
- **Without interaction, the environment tends toward “death”** (decay, stall, collapse — exact mechanic TBD).
- There is **no player avatar**; the player’s personal survival is **not** a factor. Stakes are the **system / ecology**, not a character HP bar.

### Open design questions

| Question | Notes |
|----------|--------|
| **Central theme** | Visual identity, fiction, and tone are undecided. Current UI is functional nav only. |
| **Win / lose** | What counts as “alive” vs “dead”? Time limits, vitality metric, critical cell networks? |
| **Progression** | How does a session or campaign advance? Unlocks, larger regions, new pressures, scenarios? |
| **Opposition without creatures** | No moving “creatures” in current concept. How does the **system fight back**? Examples to explore: spreading decay types, resource drain, topological faults, scheduled failures, competing autonomous subsystems — all **cell-local / field dynamics**, not chase AI. |
| **Player role** | Gardener, engineer, stabilizer — language should follow once theme is fixed. |

Decisions here should land in planning before major content work (cell catalog, world gen, analytics).

---

## Cell types and world generation (after theme)

### Type catalog expansion

- Present v1 types (generator, transformer, aggregator) may become **base types** that **derived types** extend or compose.
- New behaviors, visuals, and neighbor rules follow the chosen theme (e.g. vitality sources, rot, conduits, seals).

### Themed world generation

- Replace or supplement the current **demo seed** with **theme-aligned world gen**: starting vitality, pressure gradients, tutorial pockets, failure modes.
- World gen spec should follow cell-catalog and theme docs, not precede them.

Suggested dependency order:

```text
theme + win/lose sketch  →  cell catalog (P8)  →  themed world gen  →  batch ops / multi-select UI
```

---

## Suggested roadmap hooks

| Idea | Likely home | Depends on |
|------|-------------|------------|
| Persistent selection + view centering | [P5.1](../specs/P5.1-selection-view-nav.md) | Camera framing API |
| Pointer remap (LMB select) | [P5.1](../specs/P5.1-selection-view-nav.md) | — |
| View orbit (Ctrl+arrows), zoom, help overlay | [P5.1](../specs/P5.1-selection-view-nav.md) | Selection model |
| Multi-select + batch ops | P8 factory depth | Ops catalog |
| Theme + progression doc | Planning | Design pass |
| Derived cell types | P8.1 cell catalog | Theme |
| Themed world gen | P8+ or new phase | Theme, types |

See [future roadmap](../roadmap/future.md) for phased backlog rows.
