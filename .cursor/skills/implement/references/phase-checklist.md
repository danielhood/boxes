# Phase implementation checklist

Copy into the agent todo list or PR body when running the **implement** skill.

## Pre-flight

- [ ] On `main`, pulled latest
- [ ] Current phase identified in `docs/roadmap/active.md`
- [ ] Spec read end-to-end
- [ ] Gaps/ambiguities resolved and recorded in spec **Implementation notes**
- [ ] Out-of-scope items explicitly deferred

## Build

- [ ] Branch `cursor/implement-<name>-dd82` created
- [ ] Code in correct crate (`boxes_sim` vs `boxes_app`)
- [ ] Acceptance criteria mapped to tests
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes

## Docs

- [ ] Spec: criteria `[x]`, status `shipped`, implementation notes
- [ ] `docs/roadmap/completed.md` updated
- [ ] `docs/roadmap/active.md` — next phase promoted
- [ ] `docs/planning/initial-planning.md` — status table
- [ ] `README.md` — status / usage if needed; `BUILD.md` if build setup changed
- [ ] Next phase spec set to `active`

## Ship

- [ ] Committed with conventional message
- [ ] `git push -u origin <branch>`
- [ ] PR opened (ManagePullRequest, or `gh pr create` if that tool is unavailable) with test plan
