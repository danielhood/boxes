---
name: implement
description: Implement the current Boxes roadmap phase from its spec — review for gaps, code, test, update docs, and open a PR. Use when the user says implement, proceed with, or ship a phase (P0–P9), or when starting work from docs/roadmap/active.md.
---

# Implement a roadmap phase

End-to-end workflow for shipping a **single phase slice** in the Boxes repo. Derived from how P0–P3 were delivered.

## When to use

- User says **implement**, **proceed with**, or **ship** a phase (e.g. “implement P4”).
- `docs/roadmap/active.md` has a **not started** row and a linked spec under `docs/specs/`.
- Do **not** use for doc-only updates, bugfixes outside a phase, or work with no spec.

## Overview

```
sync main → spec review → branch → implement → test → docs → commit/push → PR
```

Complete every step in order. Do not open a PR until `cargo test` and `cargo clippy` pass.

---

## Step 1 — Sync and identify the slice

1. `git fetch origin main && git checkout main && git pull origin main`
2. Read [`docs/roadmap/active.md`](../../../docs/roadmap/active.md) — find the **Current phase** row.
3. Open the linked spec in `docs/specs/` (e.g. `P4-input-tools.md`).
4. Skim dependencies: [`docs/planning/initial-planning.md`](../../../docs/planning/initial-planning.md), prior phase specs, and **Out of scope** sections in the current spec.

**Crate ownership (Boxes convention):**

| Layer | Crate | Depends on |
|-------|-------|------------|
| Binary entry | `crates/boxes` | `boxes_app` |
| Render / input / UI | `crates/boxes_app` | `boxes_sim`, Bevy |
| Simulation kernel | `crates/boxes_sim` | std only — **no Bevy** |

---

## Step 2 — Spec review (gaps and ambiguities)

Before writing code, produce a short mental (or scratch) checklist:

### 2a. Read for completeness

| Check | Action if missing |
|-------|-------------------|
| Goal and **In scope** / **Out of scope** | Stop; ask user or add spec section before coding |
| **Acceptance criteria** (testable bullets) | Add criteria to spec or confirm with user |
| Parameters (sizes, rates, budgets) | Default from `initial-planning.md` or document choice |
| API / data layout | Sketch types and public functions |
| Dependencies on prior phases | Read shipped specs; reuse existing APIs |

### 2b. Resolve ambiguities explicitly

Prior phases left decisions **in the spec** rather than guessing silently. For each open choice:

1. Pick the simplest option that satisfies acceptance criteria and planning constraints.
2. Record it under a new **`## Implementation notes (YYYY-MM-DD)`** table in the spec (see P3 rendering spec for format).
3. If the choice changes behavior a future phase assumes, note it in the spec **Related** section.

**Common ambiguity patterns in this repo:**

- “TBD in implementation” → decide, implement, document in spec (P1 cell layout, P2 state machines).
- “Pick one in implementation” → choose one, document (P3 topmost vs slice).
- “Document choice” (e.g. snap vs interpolate) → state the choice in Implementation notes.
- Overlap with next phase (e.g. view keys in P3 vs P4) → implement only what **this** phase’s acceptance criteria require; leave hooks for the next spec.

### 2c. Confirm exit criteria

Match spec acceptance criteria to `docs/roadmap/active.md` **Exit criteria for PN**. They must align before implementation.

---

## Step 3 — Branch and implement

```bash
git checkout -b cursor/implement-<short-phase-name>-dd82
```

Branch rules: prefix `cursor/`, suffix `-dd82`, lowercase.

### Implementation rules

- **Minimize scope** — only what the spec’s *In scope* requires; no drive-by refactors.
- **Match existing style** — module layout, naming, and patterns from neighboring code.
- **Respect boundaries** — simulation logic in `boxes_sim`; Bevy/rendering in `boxes_app`.
- **Wire incrementally** — e.g. P2 used `SimHooks` / `CellEngine`; P3 used `GridSimulation` + `GridRenderPlugin`.
- **Document non-obvious choices** in code only when the spec does not already cover them.

### Typical deliverables by phase type

| Phase type | Usual touchpoints |
|------------|-------------------|
| Foundation / CI | `Cargo.toml`, `.github/workflows/`, `README.md` |
| Simulation | `crates/boxes_sim/src/` |
| Rendering / input / UI | `crates/boxes_app/src/` |
| Cross-cutting | `boxes_app` bridge + `boxes_sim` API |

---

## Step 4 — Author tests

Map tests **directly to acceptance criteria**:

| Criterion type | Test approach |
|----------------|---------------|
| Data structure / indexing | Unit tests in owning module (`#[cfg(test)]`) |
| Tick / phase / determinism | `boxes_sim` unit tests with small fixtures |
| Cell behavior | Isolated 3×3 or column fixtures; avoid full 500³ grids |
| Performance / scale | Benchmark **stub** tests (no hard CI gate unless spec says so) |
| Bevy plugin wiring | `App::new().add_plugins(...)` smoke test in `boxes_app` |
| Render / surface logic | Pure functions tested without GPU where possible |

Add tests in the same PR as implementation. Do not mark spec acceptance criteria `[x]` until tests exist and pass.

---

## Step 5 — Run verification (required)

From repo root (Linux; install Bevy deps per `README.md` if needed):

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

CI runs the same on Ubuntu (`.github/workflows/ci.yml`). Fix all failures before proceeding.

Optional when the phase affects the window:

```bash
cargo run   # manual smoke — ortho viewport, no panics
```

---

## Step 6 — Update documentation

Update in the **same commit series** as code. Checklist:

### Spec (`docs/specs/P{N}-*.md`)

- [ ] Check all **Acceptance criteria** boxes `[x]`.
- [ ] Set **Status** to `shipped`.
- [ ] Set **Last updated** to today.
- [ ] Point **Roadmap** link to `../roadmap/completed.md`.
- [ ] Add **Implementation notes** for decisions made in Step 2.

### Roadmap

- [ ] [`docs/roadmap/completed.md`](../../docs/roadmap/completed.md) — add phase section with slice summary, spec link, PR link (use `_PR pending_` until merged).
- [ ] [`docs/roadmap/active.md`](../../docs/roadmap/active.md) — promote **next** phase to *Current phase*; remove shipped phase from current table.

### Planning and README

- [ ] [`docs/planning/initial-planning.md`](../../docs/planning/initial-planning.md) — update **Implementation status** table.
- [ ] [`README.md`](../../README.md) — **Status** line, crate table, and `cargo run` notes if behavior changed.

### Next phase spec

- [ ] Set next spec **Status** to `active` and **Roadmap** to current.

Do **not** edit `docs/roadmap/future.md` unless reprioritizing backlog.

---

## Step 7 — Commit, push, and open PR

```bash
git add -A
git commit -m "<type>(<scope>): implement P<N> <short title>

<1–3 sentence body describing what shipped and why.>"
git push -u origin cursor/implement-<short-phase-name>-dd82
```

Use conventional commits: `feat(sim):`, `feat(render):`, `docs:`, etc.

### Pull request

Use the **ManagePullRequest** tool (not `gh pr create`):

- **branch_name** — your `cursor/...-dd82` branch
- **base_branch** — `main`
- **title** — mirrors commit subject
- **body** — Summary (what/why), bullet deliverables, **Test plan** with checked items for `cargo test`, `cargo clippy`, and any manual steps

Mark draft only if explicitly requested; otherwise open ready for review.

---

## Definition of done

A phase implementation is complete when **all** are true:

1. Spec acceptance criteria are met and checked.
2. `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` pass.
3. Docs updated (spec, roadmap, planning, README as applicable).
4. Branch pushed; PR opened with a test plan.
5. No scope creep into the next phase’s spec.

---

## Reference — phases shipped to date

| Phase | Crate(s) | Notable pattern |
|-------|----------|-----------------|
| P0 | workspace, `boxes_app`, `boxes_sim` stub | CI + ortho shell |
| P1 | `boxes_sim` | Sparse chunks, `Simulation::step`, hooks |
| P2 | `boxes_sim` | `CellEngine`, type-aware dirty propagation |
| P3 | `boxes_app` | `GridRenderPlugin`, sim bridge, dirty chunk rebuild |

See [`references/phase-checklist.md`](references/phase-checklist.md) for a copy-paste checklist.
