# Completed phases

**Status:** active  
**Last updated:** 2026-07-16  
**Related:** [active roadmap](active.md), [initial planning](../planning/initial-planning.md)

Shipped work is logged here with links to specs and merge PRs. Promote the next row from [active.md](active.md) when starting new work.

## P0 — Project foundation (shipped 2026-07-16)

| Slice | Summary | Spec | Shipped |
|-------|---------|------|---------|
| P0.1 | Rust workspace + Bevy app shell (window, ortho camera, placeholder scene) | [P0-project-foundation](../specs/P0-project-foundation.md) | [PR #2](https://github.com/danielhood/boxes/pull/2) |
| P0.2 | Documentation structure and planning baseline | — | [PR #1](https://github.com/danielhood/boxes/pull/1) |

**Exit criteria met:** `cargo run` opens a Linux window with an orthographic viewport and placeholder cube; `boxes` / `boxes_app` / `boxes_sim` workspace layout in place; `docs/` planning, roadmap, and specs conventions established; CI runs `cargo build`, `cargo test`, and `cargo clippy -- -D warnings` on Ubuntu.

**Stack pinned:** Bevy 0.16, Rust stable (via `rust-toolchain.toml`).
