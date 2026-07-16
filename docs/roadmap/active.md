# Active roadmap

**Status:** active  
**Last updated:** 2026-07-16  
**Related:** [initial planning](../planning/initial-planning.md), [completed phases](completed.md), [future phases](future.md)

Phases listed here are **in progress** or **up next**. When a phase ships, move its row to [completed.md](completed.md); link the implementing spec from the Spec column.

## Current phase: P3 — Rendering

| Slice | Summary | Spec | Status |
|-------|---------|------|--------|
| P3 | Chunked GPU instancing, ortho view switching, dirty buffer updates | [P3-rendering](../specs/P3-rendering.md) | not started |

**Exit criteria for P3:** Visible orthographic grid renders from `boxes_sim` dirty chunks; view switching between top/front/left; chunk instance buffers update incrementally.

---

## Up next (still active queue)

| Phase | Summary | Spec | Status |
|-------|---------|------|--------|
| P4 | Input + tools — mouse pick, place/edit cells, keyboard view switch | [P4-input-tools](../specs/P4-input-tools.md) | not started |
| P5 | Factory UI — metrics, cell inspector, type palette | [P5-factory-ui](../specs/P5-factory-ui.md) | not started |

Implement in order unless a spec explicitly notes a dependency exception.
