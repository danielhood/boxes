# Active roadmap

**Status:** active  
**Last updated:** 2026-07-16  
**Related:** [initial planning](../planning/initial-planning.md), [completed phases](completed.md), [future phases](future.md)

Phases listed here are **in progress** or **up next**. When a phase ships, move its row to [completed.md](completed.md); link the implementing spec from the Spec column.

## Current phase: P2 — Cell types

| Slice | Summary | Spec | Status |
|-------|---------|------|--------|
| P2 | Generator, transformer, aggregator + dirty propagation | [P2-cell-types](../specs/P2-cell-types.md) | not started |

**Exit criteria for P2:** Generators fire on correct phased ticks; transformers and aggregators update only on neighbor changes; no infinite dirty loops on stable configs.

---

## Up next (still active queue)

| Phase | Summary | Spec | Status |
|-------|---------|------|--------|
| P3 | Rendering — chunked GPU instancing, ortho view switching, dirty buffer updates | [P3-rendering](../specs/P3-rendering.md) | not started |
| P4 | Input + tools — mouse pick, place/edit cells, keyboard view switch | [P4-input-tools](../specs/P4-input-tools.md) | not started |
| P5 | Factory UI — metrics, cell inspector, type palette | [P5-factory-ui](../specs/P5-factory-ui.md) | not started |

Implement in order unless a spec explicitly notes a dependency exception.
