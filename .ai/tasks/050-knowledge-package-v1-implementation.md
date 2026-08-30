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

- 2026-08-30 (codex, plan Task 4): Task 3 is accepted and Task 4 authorized. Read only
  plan Task 4 in full. Its current Step 1 contains eight mandatory tests—the original six
  cases plus the two structural `schema_version` cases added during review—so the green
  total will be 16 rather than the prompt's stale count of 14. Added all eight tests before
  implementation and registered the private `package` module so Cargo compiles the red
  tests; no Task 5+ or `knowledge-package/` work is in scope.
- 2026-08-30 (codex, plan Task 4): The red run,
  `cargo test --locked knowledge::package`, failed as intended because
  `parse_package_toml`, `parse_sources_toml`, `PackageIdentity`, and the five new error
  variants were undefined. Applied Task 4 Step 3's full `KnowledgeError` replacement and
  added exactly `PackageIdentity` plus the two private parsing/validation functions, with
  no later-task conversion or loader work.
- 2026-08-30 (codex, plan Task 4): Green/gates complete. The final
  `cargo test --locked knowledge::` run passed 16 tests—the prior 8 plus all 8 tests in the
  corrected Step 1—with 0 failures and 40 filtered out.
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings, and
  `cargo fmt --all --check` passed after applying rustfmt's mechanical wrapping and module
  sorting. Rustfmt places `mod package;` before `mod raw;`, rather than literally after it
  as Step 4 says; this is the repository's canonical formatter behavior and does not change
  module privacy or semantics. Task 5+ and `knowledge-package/` remain untouched.
- 2026-08-30 (codex, plan Task 3): Task 2 is accepted and Task 3 authorized. Read only
  plan Task 3 in full. Its interface and implementation code consistently prescribe eight
  raw structs, so I am treating the prompt's “seven” as a counting typo. Added the four
  exact parse-layer tests before the structs and registered the private `raw` module so
  Cargo compiles the red tests; no Task 4+ or `knowledge-package/` work is in scope.
- 2026-08-30 (codex, plan Task 3): The red run, `cargo test --locked knowledge::raw`,
  failed as intended because `RawKnowledgePackage`, `RawConceptFrontmatter`, and
  `RawSource` were undefined. Added exactly the eight prescribed crate-internal
  deserialization structs, with the plan's field types, defaults, and unknown-field
  rejection attributes; added no conversions, validation, relationships, or error variants.
- 2026-08-30 (codex, plan Task 3 blocker): `cargo test --locked knowledge::` passes all
  8 expected tests, but the exact prescribed code emits `dead_code` warnings because all
  eight raw structs are `pub(crate)` and their production consumers are deliberately
  deferred to Tasks 4, 6, and 8. Consequently,
  `cargo clippy --all-targets --locked -- -D warnings` fails with eight dead-code errors.
  The available fixes each exceed or contradict Task 3's locked scope: add an unprescribed
  lint exemption, export parse-layer types the plan says must never be public, or implement
  later-task consumers. `cargo fmt --all --check` passes. Stopped without choosing a new
  contract and without committing; Task 4+ and `knowledge-package/` remain untouched.
- 2026-08-30 (codex, plan Task 3 resumed): Re-read corrected Task 3 after commit `ee7d9e5`
  authorized a temporary knowledge-module-root `#![allow(dead_code)]`, inherited by the
  incremental crate-private implementation and explicitly scheduled for removal once the
  loader provides production callers. Applied Task 3 Step 4's exact full `mod.rs` shape.
  `cargo test --locked knowledge::` passed all 8 expected tests with 0 failures and 40
  filtered out; `cargo clippy --all-targets --locked -- -D warnings` passed cleanly; and
  `cargo fmt --all --check` passed. The prior blocker is resolved without exporting raw
  types or implementing Task 4+; `knowledge-package/` remains untouched.
- 2026-08-30 (codex, plan Task 2): Task 1 is accepted and Task 2 authorized. Read only
  plan Task 2 in full and confirmed its domain-type contract is consistent. Added the
  prescribed `Example` JSON round-trip test before the types, with the final `mod types;`
  declaration so Cargo compiles the red test; no Task 3+ files or `knowledge-package/`
  changes are in scope.
- 2026-08-30 (codex, plan Task 2): The red run, `cargo test --locked knowledge::types`,
  failed as intended because `Example`, `ProvenanceRef`, `SourceLocator`, and
  `ProvenanceKind` were undefined. Added exactly the eight prescribed serde domain types
  and Task 2's exact `knowledge/mod.rs` exports, without constructors, validation, or new
  `KnowledgeError` variants.
- 2026-08-30 (codex, plan Task 2): Green/gates complete. `cargo test --locked knowledge::`
  passed 4 tests (the 3 Task 1 identifier tests plus the new `Example` JSON round trip),
  with 0 failures and 40 filtered out. `cargo clippy --all-targets --locked -- -D warnings`
  passed with zero warnings, and `cargo fmt --all --check` passed without changes. Task 2
  required no unspecified implementation choice and exposed no blocker or spec/plan
  contradiction; Task 3+ and `knowledge-package/` remain untouched.
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

- 2026-08-30 (claude-code): Confirmed Codex's Task 3 blocker as real, then found it was
  broader than reported — reproduced the exact clippy failure (`cargo clippy --all-targets`
  → 8 "field is never read" errors, both lib and lib-test targets), and checked why: Task
  3's own tests construct only 3 of the 8 raw structs and don't read every field even on
  those. Traced the same shape (`pub(crate)` item, no production caller until a later task)
  across the rest of the plan and confirmed it recurs through Task 11 — Tasks 1–2 are exempt
  because their items are `pub` and reachable from the crate's public surface, which the
  compiler can't prove unreachable; `pub(crate)` items have no such exemption. A per-file
  `#[allow(dead_code)]` (my first attempt, on `raw.rs` alone) would only have deferred the
  identical failure onto Task 4, then 5, then 6... one at a time. Replaced it with a single
  `#![allow(dead_code)]` at the `knowledge` module root, added in Task 3 (first task that
  needs it), lint-inherited by every descendant module for the rest of the build-out, removed
  explicitly in Task 12 once the loader wires every `pub(crate)` item to a real caller. Also
  found and fixed four stale "Task 13 (loader)" references in the plan (loader is Task 12;
  these were leftover from an earlier numbering pass) plus one resulting off-by-one ("Tasks
  9–12" → "Tasks 9–11"). Plan re-committed; nothing in `src-tauri/` touched by this fix.

## What was built / tested / left out

Filled in when moving to `review`, after plan Task 17 (the plan's last task) is accepted.

## Review

Per-plan-Task review notes accumulate here as each one is accepted, before the final
task-level review.

### Plan Task 1 — Knowledge error taxonomy and identifier types (commit `e30ebc3`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced rather than trusted: `cargo test --locked` → 43 passed (40
pre-existing `modules::` + 3 new `knowledge::`, no regression), `cargo clippy --all-targets
--locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git status` → clean.
Diffed all six changed files against the corrected plan's Task 1 line-by-line — content
matches exactly. `ModuleId`/`CapabilityId`'s public API is untouched (confirmed via diff —
only the one `mod`/`fn` visibility widening each, nothing else in either `modules/` file).
`KnowledgeError` has exactly the one `InvalidIdentifier` variant the corrected plan calls
for, not the original 34. No duplicate-entity-ID variant added. No Task 2+ or
`knowledge-package/` scope creep.

One reported deviation, accepted: `pub mod knowledge;` placed before `pub mod modules;` in
`lib.rs`, not after as the plan's Step 5 literally said. Correct call, not a shortcut —
rustfmt's `reorder_modules` default (true) alphabetically sorts contiguous same-visibility
`mod` declarations, so "after modules" was fighting a formatter default that would just
reintroduce churn on the next `cargo fmt`. Smallest change consistent with the plan's actual
intent (register the module), reported with reasoning.

**Verdict: accepted.** Plan Task 2 (domain types) authorized.

### Plan Task 2 — Domain types (commit `58b3303`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 4 passed (the 3 from Task 1
plus this task's round-trip test), `cargo clippy --all-targets --locked -- -D warnings` →
clean, `cargo fmt --all --check` → clean, `git status` → clean. Diffed `types.rs` field by
field against the plan's Task 2 Step 3 — all eight structs/enum match exactly (field names,
types, `Option`/`Vec` wrapping, derives including `Eq` present only on `SourceLocator`/
`ProvenanceKind` as specified, `ProvenanceKind`'s `#[serde(rename_all = "lowercase")]`
matching spec §11's `"direct"`/`"derived"` serialization requirement). `mod.rs`'s re-export
block matches Step 4 exactly, existing `error`/`identifier` re-exports preserved. No
constructors, no validation, no new `KnowledgeError` variants, no Task 3+ or
`knowledge-package/` scope creep.

**Verdict: accepted.** Plan Task 3 (raw serialization structs) authorized.

### Plan Task 3 — Raw serialization structs (commit `a5b0f08`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 8 passed, `cargo clippy
--all-targets --locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git
status` → clean. Diffed `mod.rs` and `raw.rs` against the corrected plan — `#![allow(dead_
code)]` placement/comment and all eight `Raw*` structs match exactly, `#[serde(deny_unknown_
fields)]` present on every one, no `pub use` for `raw` (confirmed via diff — only `mod raw;`
added). This is the task that surfaced the dead-code gap in the first place, now resolved at
the plan level rather than patched around; nothing further needed here.

**Verdict: accepted.** Plan Task 4 (`package.toml`/`sources.toml` parsing) authorized.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
