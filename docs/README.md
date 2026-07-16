# Boxes — documentation

This directory holds all project documentation. Code lives at the repo root; **design and delivery tracking live here**.

## Folder layout

| Path | Purpose | When to update |
|------|---------|----------------|
| [`planning/`](planning/) | Vision, constraints, stack choices, and early design rationale | During ideation and major pivots; treat as historical record once decisions ship |
| [`roadmap/`](roadmap/) | **What** we are building and **when** — phases, active work, future work | When starting, completing, or reprioritizing a phase or work slice |
| [`specs/`](specs/) | **How** each work slice is built — APIs, data layouts, algorithms, acceptance criteria | Before and during implementation of that slice; one spec per slice |

## Workflow

1. **Planning** — Capture goals, constraints, and rejected alternatives. Do not duplicate full implementation detail here; link to specs.
2. **Roadmap** — Break delivery into phases. Keep exactly one “current” view in [`roadmap/active.md`](roadmap/active.md). Move completed phases to [`roadmap/completed.md`](roadmap/completed.md) or archive with a date.
3. **Specs** — When a roadmap slice is ready to build, add or update a spec under `specs/`. Specs are the source of truth for engineers and agents implementing that slice.

```
planning (why)  →  roadmap (what/when)  →  specs (how)
```

## Naming conventions

- **Planning:** `planning/<topic>.md` — e.g. `initial-planning.md`
- **Roadmap:** `roadmap/active.md` (current), `roadmap/future.md` (backlog), optional `roadmap/completed.md`
- **Specs:** `specs/<slice-id>-<short-name>.md` — e.g. `specs/P1-simulation-core.md`

Use stable **slice IDs** (P1, P2, … or feature codes) in roadmap rows and spec filenames so cross-links stay valid.

## Document headers

Each doc should start with:

- **Status** — draft | active | superseded | shipped
- **Last updated** — date
- **Related** — links to planning, roadmap rows, or other specs

## What does not belong here

- Generated artifacts, build output, or assets
- Duplicate content that belongs in code comments or crate READMEs
- Changelog-style release notes (add `history/` later if needed)

## Quick links

- [Initial planning](planning/initial-planning.md)
- [Active roadmap](roadmap/active.md)
- [Future phases](roadmap/future.md)
