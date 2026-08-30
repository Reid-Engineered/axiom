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

- 2026-08-30 (codex, plan Task 9): Task 8 is accepted and Task 9 authorized. Read only
  corrected plan Task 9 in full and confirmed its filesystem discovery contract, sorting,
  rejection logic, and eight-test list are internally consistent. Added all eight
  prescribed temp-directory tests before implementation and registered the private
  `discover` module so Cargo compiles the red step; no Task 10+ or `knowledge-package/`
  work is in scope.
- 2026-08-30 (codex, plan Task 9): The red run,
  `cargo test --locked knowledge::discover`, failed as intended because
  `discover_entities`, both root-file readers, `DiscoveredEntities`, and the six filesystem
  error variants were undefined. Added the exact deterministic discovery/read helpers,
  filename-ID enforcement, new errors/Display arms, and the `Io` arm in `Error::source`;
  no Task 10 reference validation or Task 12 loader assembly was added.
- 2026-08-30 (codex, plan Task 9): Green/gates complete. The final
  `cargo test --locked knowledge::` run passed all 57 expected tests—the prior 49 plus the
  8 real-filesystem discovery tests—with 0 failures and 40 filtered out.
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings, and
  `cargo fmt --all --check` passed after mechanical wrapping. Tests cover optional entity
  directories, deterministic filename sorting, filename/ID enforcement, invalid entries,
  and missing required root TOML files. Task 10+ and `knowledge-package/` remain untouched.
- 2026-08-30 (codex, plan Task 8): Task 7 is accepted and Task 8 authorized. Read only
  plan Task 8 in full and confirmed its composition-only parser contract, production
  imports, and three-test list are internally consistent. Added all three prescribed tests
  before the parser and registered the private `example` module so Cargo compiles the red
  step; no Task 9+ or `knowledge-package/` work is in scope.
- 2026-08-30 (codex, plan Task 8): The red run,
  `cargo test --locked knowledge::example`, failed as intended because
  `parse_example_file` was undefined. Added the exact composition-only implementation,
  reusing Tasks 5–7 without adding error variants, reference resolution, cross-concept
  validation, or discovery logic.
- 2026-08-30 (codex, plan Task 8): Green/gates complete. The final
  `cargo test --locked knowledge::` run passed all 49 expected tests—the prior 46 plus the
  3 Example entity tests—with 0 failures and 40 filtered out.
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings, and
  `cargo fmt --all --check` passed after mechanical test-expression wrapping. The parser
  remains a single-entity composition layer; Task 9+ and `knowledge-package/` remain
  untouched.
- 2026-08-30 (codex, plan Task 7): Task 6 is accepted and Task 7 authorized. Read only
  plan Task 7 in full and confirmed its closed-heading grammar, helper contract, and
  fifteen-test list are internally consistent. Added all 15 prescribed tests before the
  parser and registered the private `example_body` module so Cargo compiles the red step;
  no Task 8+ or `knowledge-package/` work is in scope.
- 2026-08-30 (codex, plan Task 7): The red run,
  `cargo test --locked knowledge::example_body`, failed as intended because
  `parse_example_body`, `ParsedExampleBody`, and the seven grammar error variants were
  undefined. Added the exact error/Display additions and parser with its
  `section_name`/`section_text`/`parse_hints` helpers; no Example frontmatter or entity-file
  parsing from Task 8 was added.
- 2026-08-30 (codex, plan Task 7): Green/gates complete. The final
  `cargo test --locked knowledge::` run passed all 46 expected tests—the prior 31 plus the
  15 body-grammar tests—with 0 failures and 40 filtered out.
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings, and
  `cargo fmt --all --check` passed after mechanical wrapping. Exact `## Problem`/
  `## Solution`/`## Hints` recognition, opaque deeper headings, optional ordered hints, and
  all prescribed rejection cases are covered. Task 8+ and `knowledge-package/` remain
  untouched.
- 2026-08-30 (codex, plan Task 6): Task 5 is accepted and Task 6 authorized. Read only
  plan Task 6 in full. The corrected import placement is internally consistent, but the
  current Step 1 contains six Concept tests rather than the prompt's five; together with
  Step 5's two Objective tests, the green total should be 31 rather than 30. Following all
  eight locked tests. Added the six Concept tests first and registered the private module
  so Cargo compiles the Concept red step; no Task 7+ or `knowledge-package/` work is in
  scope.
- 2026-08-30 (codex, plan Task 6): The Concept red run,
  `cargo test --locked knowledge::concept`, failed as intended because
  `parse_concept_file` and the four provenance error variants were undefined. Added the
  exact shared `convert_provenance_refs` helper and error/Display additions, then the
  Concept parser. The production import list contains `Concept` alone; `ProvenanceKind`
  remains inside the test module as required.
- 2026-08-30 (codex, plan Task 6): The six Concept tests passed after their green step.
  Added the two prescribed Objective tests before `parse_objective_file` and registered the
  private `objective` module so Cargo compiles the second red step independently.
- 2026-08-30 (codex, plan Task 6): The Objective red run,
  `cargo test --locked knowledge::objective`, failed as intended because
  `parse_objective_file` was undefined. Added the exact Objective parser, reusing the shared
  provenance converter and Task 4's `EmptyField`; no cross-entity resolution or discovery
  work was added.
- 2026-08-30 (codex, plan Task 6): Green/gates complete. The final
  `cargo test --locked knowledge::` run passed 31 tests—the prior 23 plus the six Concept
  and two Objective tests in the current plan—with 0 failures and 40 filtered out.
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings, including
  the corrected test-only `ProvenanceKind` import placement, and
  `cargo fmt --all --check` passed after mechanical wrapping/module sorting. Task 7+ and
  `knowledge-package/` remain untouched.
- 2026-08-30 (codex, plan Task 5): Task 4 is accepted and Task 5 authorized. Read only
  plan Task 5 in full and confirmed its splitter contract and seven-test list are
  consistent. Added all seven prescribed tests before implementation and registered the
  private `frontmatter` module so Cargo compiles the red tests. The function remains pure
  string input/output with `Path` used only for error context; no Task 6+ or
  `knowledge-package/` work is in scope.
- 2026-08-30 (codex, plan Task 5): The red run,
  `cargo test --locked knowledge::frontmatter`, failed as intended because
  `split_frontmatter` and the three frontmatter error variants were undefined. Added the
  exact enum/Display additions and the specified pure string splitter; it normalizes CRLF,
  scans only the supplied text, and uses `Path` solely when constructing errors.
- 2026-08-30 (codex, plan Task 5 blocker): The exact Step 3 implementation contradicts
  three locked Step 1 assertions. Rust `str::split('\n')` retains a final empty element for
  newline-terminated input; joining `remaining[body_start..]` and then appending another
  newline makes `"Body text.\n"`, `"Body.\n"`, and the two-paragraph body each return with
  two trailing newlines. `cargo test --locked knowledge::` therefore produced 20 passes
  and 3 failures (`splits_toml_and_body`, `accepts_crlf_line_endings`, and
  `trims_leading_blank_lines_from_body_but_preserves_internal_blank_lines`). Fixing this
  requires changing the locked implementation—for example, using `split_terminator` or
  conditionally avoiding the extra newline. Stopped without selecting a new contract,
  without running later gates, and without committing; Task 6+ and `knowledge-package/`
  remain untouched.
- 2026-08-30 (codex, plan Task 5 resumed/blocker): Re-read corrected Step 3 after commit
  `9a3e8f7` and changed only `split('\n')` to `split_terminator('\n')`. The full
  `cargo test --locked knowledge::` run then passed all 23 expected tests with 0 failures
  and 40 filtered out, resolving the trailing-newline defect. A second locked-code/gate
  conflict surfaced: Step 3 explicitly declares `split_frontmatter<'a>` with `raw: &'a str`,
  but the function returns owned strings and does not otherwise use the lifetime, so
  `cargo clippy --all-targets --locked -- -D warnings` fails on
  `clippy::needless_lifetimes`. Fixing it requires eliding the prescribed lifetime or adding
  an unprescribed Clippy exemption. `cargo fmt --all --check` also reports only mechanical
  wrapping/module-order changes, but no formatting was applied after the Clippy blocker.
  Stopped without selecting another contract and without committing; Task 6+ and
  `knowledge-package/` remain untouched.
- 2026-08-30 (codex, plan Task 5 resumed): Verified commit `cb057ba` corrected Step 3's
  signature to `split_frontmatter(path: &Path, raw: &str)` and applied that exact lifetime
  elision. After rustfmt's mechanical Display-arm wrapping and alphabetical private-module
  ordering, `cargo test --locked knowledge::` passed all 23 expected tests with 0 failures
  and 40 filtered out, `cargo clippy --all-targets --locked -- -D warnings` passed with zero
  warnings, and `cargo fmt --all --check` passed. Both prior Task 5 plan blockers are now
  resolved; Task 6+ and `knowledge-package/` remain untouched.
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

- 2026-08-30 (claude-code): Confirmed Codex's Task 5 blocker by hand-tracing the exact Step
  3 code against all three failing cases. `"...Body text.\n".split('\n')` yields a trailing
  empty-string element because the input ends in the delimiter, not merely contains it; that
  element survives into `remaining`, folds into `body_text` via `.join("\n")`, and collides
  with the `+ "\n"` appended after it — `"Body text.\n" + "\n"` = `"Body text.\n\n"`, exactly
  the double-trailing-newline Codex reported. Fixed `docs/superpowers/plans/2026-08-30-
  knowledge-package-v1.md` Task 5 Step 3: `normalized.split('\n')` → `normalized.split_
  terminator('\n')`, which drops that spurious trailing element. Re-traced all three
  originally-failing tests plus the CRLF and internal-blank-line-preservation cases by hand
  against the corrected version — all come out exactly as each test expects. One-line fix,
  no other line in Step 3 needed to change. Nothing in `src-tauri/` touched by this fix;
  Codex's uncommitted Task 5 work stays as-is for it to pick back up.

- 2026-08-30 (claude-code): Confirmed Codex's second Task 5 blocker. `split_frontmatter`'s
  Step 3 code block declared `<'a>` and `raw: &'a str`, but `'a` appears nowhere else in the
  signature — the return type (`Result<(String, String), KnowledgeError>`) is fully owned,
  and the function body only ever uses `raw` to build a fresh owned `normalized` String, never
  returning anything borrowed from it. Confirmed this really is needless (not a considered
  choice this task's Interfaces line already contradicted it): the "Produces" line for this
  same task, written earlier, already states the correctly-elided signature `raw: &str` with
  no lifetime — only the literal Step 3 code block had the stray explicit one. Fixed
  `docs/superpowers/plans/2026-08-30-knowledge-package-v1.md` Task 5 Step 3 to match: removed
  `<'a>` and changed `&'a str` to `&str`. Purely syntactic — Rust's elision rules produce an
  identical effective signature, nothing about behavior changes. Nothing in `src-tauri/`
  touched by this fix; Codex's uncommitted Task 5 work (including its already-correct
  `split_terminator` fix from the previous round) stays as-is for it to pick back up.

- 2026-08-30 (claude-code): Before issuing Task 6, traced its code by hand rather than
  waiting for another Codex round-trip, given the last two tasks each surfaced a real bug in
  the plan's literal code. Found one: `concept.rs` Step 4 imports `use super::types::
  {Concept, ProvenanceKind};`, but `ProvenanceKind` is never referenced in
  `parse_concept_file`'s body — only in Step 1's test assertion
  (`concept.provenance_refs[0].kind, ProvenanceKind::Direct`), which doesn't exist in a
  non-test compilation. `#![allow(dead_code)]` (the Task 3 fix) does not cover this — `unused
  _imports` is a separate lint, and `cargo clippy -- -D warnings` denies it the same way.
  Fixed `docs/superpowers/plans/2026-08-30-knowledge-package-v1.md`: removed `ProvenanceKind`
  from Step 4's outer-scope import, added `use crate::knowledge::ProvenanceKind;` inside
  Step 1's test module instead (via the existing `pub use` re-export from Task 2, not a new
  path). Then checked every other remaining task (7 through 12) for the same shape by hand —
  every import in each is genuinely used by that task's own production code, not just its
  tests; no further instances found. Nothing in `src-tauri/` touched by this fix.

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

### Plan Task 4 — `package.toml`/`sources.toml` parsing and validation (commit `2876ea2`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 16 passed, `cargo clippy
--all-targets --locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git
status` → clean. Diffed `error.rs` and `package.rs` against the plan — the five new variants,
the now-real `Error::source()` handling `TomlSyntax`, and `parse_package_toml`/`parse_
sources_toml` all match exactly, field for field. The "14 vs 16" test-count mismatch in the
issuing prompt was my own arithmetic error (miscounted the plan's already-6+2=8-test Step 1
as "six"), not a plan defect — Codex correctly followed the plan's actual test list rather
than my prompt's summary, which is the right call when the two disagree. `mod package;`
landing before `mod raw;` in `mod.rs` is the same well-reasoned rustfmt-driven deviation
pattern as Task 1's `lib.rs` ordering — noted, not an issue.

**Verdict: accepted.** Plan Task 5 (TOML frontmatter splitter) authorized.

### Plan Task 5 — TOML frontmatter splitter (commit `b4e6dad`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 23 passed, `cargo clippy
--all-targets --locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git
status` → clean. Diffed `frontmatter.rs`, `error.rs`, `mod.rs` against the twice-corrected
plan — `split_terminator`, the elided lifetime, all three new error variants and their
`Display` arms, and the module registration all match exactly. This task took two review
rounds to land (a real trailing-newline bug, then a real needless-lifetime lint failure),
both genuine plan defects Codex correctly refused to route around unilaterally — both fixed
at the plan level and verified by hand before re-issuing, same discipline as Tasks 1 and 3.

**Verdict: accepted.** Plan Task 6 (Concept/Objective entity parsers) authorized.

### Plan Task 6 — Concept/Objective entity parsers (commit `2a3b813`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 31 passed, `cargo clippy
--all-targets --locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git
status` → clean. This confirms in practice, not just by inspection, that the pre-emptive
`ProvenanceKind` import fix actually works — no `unused_imports` regression. Diffed
`concept.rs`, `objective.rs`, `provenance.rs`, `error.rs`, `mod.rs` against the plan — all
match exactly, including the `O(n²)` exact-duplicate-provenance scan and the empty-locator
rule. First task in a while with no blocker at all; the pre-trace before issuing paid for
itself.

**Verdict: accepted.** Plan Task 7 (Example body grammar parser) authorized.

### Plan Task 7 — Example body grammar parser (commit `b714c50`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 46 passed, `cargo clippy
--all-targets --locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git
status` → clean. Diffed `example_body.rs` against the plan — matches verbatim, including the
`section_name`/`section_text`/`parse_hints` helpers. Confirms the pre-trace (heading
detection, ordering, `&&str` handling in `parse_hints`, `starts_with("## ")` correctly
treating `### sub-headings` as opaque) held up in practice — no blocker this round, second
task in a row with none.

**Verdict: accepted.** Plan Task 8 (Example entity parser) authorized.

### Plan Task 8 — Example entity parser (commit `11c4662`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 49 passed, `cargo clippy
--all-targets --locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git
status` → clean. Diffed `example.rs`/`mod.rs` against the plan — matches verbatim. Confirms
the pre-trace held. Third task in a row with no blocker.

**Verdict: accepted.** Plan Task 9 (package discovery) authorized.

### Plan Task 9 — Package discovery (commit `5fef187`)

Reviewer: claude-code. Date: 2026-08-30.

Independently reproduced: `cargo test --locked knowledge::` → 57 passed, `cargo clippy
--all-targets --locked -- -D warnings` → clean, `cargo fmt --all --check` → clean, `git
status` → clean. Diffed `discover.rs`/`error.rs`/`mod.rs` against the plan — matches exactly,
including the `Io` second match arm in `Error::source()`. Fourth task in a row with no
blocker; the pre-trace on the first real-filesystem task held.

**Verdict: accepted.** Plan Task 10 (cross-entity reference validation) authorized.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
