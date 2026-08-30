---
id: 050
title: Knowledge Package v1 — runtime implementation and Calc II migration
status: in-progress
owner: codex
stage: 8
depends_on: []
---

## Scope

Implements the Rust loader/validator for Knowledge Package v1 and migrates the existing
`knowledge-package/` (Calc II reference content) against it, per the fully-specified
implementation plan:

```text
docs/superpowers/specs/2026-08-30-knowledge-package-v1-design.md   (brainstorm)
docs/superpowers/specs/2026-08-30-knowledge-package-v1-spec.md     (formal spec, commit 5d3f283)
docs/superpowers/plans/2026-08-30-knowledge-package-v1.md          (implementation plan, 17 tasks)
```

This task file tracks the whole 17-task plan as one unit of work (matching this repo's own
task granularity precedent — 046/047/048 each bundled several TDD cycles into one task file),
but execution is per-plan-Task, not one shot: Codex implements one plan Task at a time,
Claude reviews it against the spec/plan/prerequisites before the next plan Task is issued.
No plan Task is issued until the previous one is reviewed and accepted. This is a deliberate
working-mode variant of this repo's usual "one task, one review at the end" pattern — the
review checkpoints happen *inside* this task, once per plan Task, not once at the end.

Explicitly out of scope, same as the plan itself: Canonical Problem, `math.verify`, Practice,
Tutor integration, UI, Docling ingestion, and any `knowledge.query` capability. If executing
a plan Task seems to require touching any of those, that is a specification contradiction to
report, not a decision to make unilaterally — see `.ai/tasks/TEMPLATE.md`'s framing and
`CLAUDE.md`'s "architectural" escalation rule.

## Plan

Files expected to change, per the implementation plan's own file lists:
- `src-tauri/src/knowledge/` (new module: `error.rs`, `identifier.rs`, `types.rs`, `raw.rs`,
  `package.rs`, `frontmatter.rs`, `concept.rs`, `objective.rs`, `provenance.rs`,
  `example_body.rs`, `example.rs`, `discover.rs`, `validate.rs`, `relationships.rs`,
  `loader.rs`, `mod.rs`, `tests/`)
- `src-tauri/src/modules/identifier.rs` (widen `validate_identifier` visibility)
- `src-tauri/src/modules/mod.rs` (widen the `identifier` module's own visibility — required
  alongside the function widening; module privacy in Rust is transitive along the path)
- `src-tauri/src/lib.rs` (register `pub mod knowledge;`)
- `knowledge-package/` (migrated to v1 format — plan Tasks 15–17 only, not before)
- `ARCHITECTURE.md`, `knowledge-package/synthesis-report.md` (plan Task 17 only)

## Worklog

- 2026-08-30 (codex, plan Task 1): Claimed task 050 after re-reading spec §§1–2,
  `modules/identifier.rs`, and only the corrected plan preamble/Global Constraints/Task 1.
  The prior error-taxonomy contradiction is resolved: this step will add exactly
  `InvalidIdentifier` and the five distinct wrapper types, with no Task 2+ implementation
  and no changes to `knowledge-package/`. The corrected plan directory is currently
  untracked user content and will remain unchanged and excluded from this step's commit.
- 2026-08-30 (codex, plan Task 1): Added the prescribed identifier tests before the
  implementation. Because the plan wires the module after its red-test step, I temporarily
  exposed `knowledge::identifier` so Cargo would actually compile the tests rather than run
  zero tests. `cargo test --locked knowledge::identifier` then failed as expected with the
  five wrapper types and `KnowledgeError` undefined. Final module wiring will match the plan
  exactly.
- 2026-08-30 (codex, plan Task 1 blocker): The implemented plan code does not compile:
  `crate::modules::identifier::validate_identifier` fails with Rust E0603 because
  `modules/mod.rs` declares `mod identifier;` privately. Making the function `pub(crate)` is
  insufficient across sibling modules when its containing module remains private. Resolving
  this requires a contract-authorized change not present in Task 1's exact file list: either
  `pub(crate) mod identifier;`, or a `pub(crate) use identifier::validate_identifier;`
  re-export plus a different import path. Stopped without choosing between them, without
  running later gates, and without touching `knowledge-package/` or plan Task 2+.
- 2026-08-30 (codex, plan Task 1 resumed): Applied the now-authorized
  `pub(crate) mod identifier;` change. `knowledge/mod.rs` already matched Task 1 Step 5's
  final `error`/`identifier` wiring exactly, so no temporary wiring remained to remove.
  `cargo test --locked knowledge::identifier` passed all 3 tests and
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings.
  `cargo fmt --all --check` initially required `pub mod knowledge;` before
  `pub mod modules;`; I adopted rustfmt's canonical ordering because the plan's requested
  "after modules" placement is semantically irrelevant and cannot coexist with the required
  formatting gate. The subsequent format check passed. Plan Task 1 is ready for its internal
  review checkpoint; Task 2 and `knowledge-package/` remain untouched.
- 2026-08-30 (claude-code): Task created and proposed. Plan Task 1 (error taxonomy +
  identifier types) prompt issued to Codex.

- 2026-08-30 (claude-code): Confirmed Codex's blocker report as a genuine gap in the plan,
  not a Codex error — verified directly against `modules/mod.rs:1`'s `mod identifier;`
  (private) and Rust's transitive module-privacy rule (a private module is reachable only
  from its defining module and that module's descendants; `knowledge` is a sibling of
  `modules`, not a descendant, so `E0603` is the correct compiler behavior, not a mistake on
  Codex's part). Fixed `docs/superpowers/plans/2026-08-30-knowledge-package-v1.md` Task 1
  Step 1 to also widen `mod identifier;` → `pub(crate) mod identifier;` in `modules/mod.rs`,
  and added that file to Task 1's Files list. Re-issuing Task 1 to Codex with the corrected
  plan; still nothing committed for this task.

## What was built / tested / left out

Filled in when moving to `review`, after plan Task 17 (the plan's last task) is accepted.

## Review

Per-plan-Task review notes accumulate here as each one is accepted, before the final
task-level review.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
