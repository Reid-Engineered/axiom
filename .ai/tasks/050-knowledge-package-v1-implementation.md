---
id: 050
title: Knowledge Package v1 — runtime implementation and Calc II migration
status: proposed
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
- `src-tauri/src/modules/identifier.rs` (widen `validate_identifier` visibility only)
- `src-tauri/src/lib.rs` (register `pub mod knowledge;`)
- `knowledge-package/` (migrated to v1 format — plan Tasks 15–17 only, not before)
- `ARCHITECTURE.md`, `knowledge-package/synthesis-report.md` (plan Task 17 only)

## Worklog

- 2026-08-30 (claude-code): Task created and proposed. Plan Task 1 (error taxonomy +
  identifier types) prompt issued to Codex.

## What was built / tested / left out

Filled in when moving to `review`, after plan Task 17 (the plan's last task) is accepted.

## Review

Per-plan-Task review notes accumulate here as each one is accepted, before the final
task-level review.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
