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

- 2026-08-30 (claude-code, plan Task 14 blocker triage): Confirmed the blocker: Step 2's
  literal `assert_eq!(example.solution.contains("63"), true)` trips
  `clippy::bool_assert_comparison` under `-D warnings`, and Task 14's own Step 3 gate chains
  clippy after the test run, so the prescribed code cannot pass its own gate as written.
  Fixed the plan to the Clippy-clean equivalent, `assert!(example.solution.contains("63"))`
  — same assertion, no behavior change. **Task 14 is re-authorized**: keep the seven
  fixture files, module registration, and everything else already done; only the one
  assertion line in `canonical.rs` changes.
- 2026-08-30 (codex, plan Task 14 blocker): Task 13 is accepted and Task 14 authorized.
  Read the full task file and fresh corrected Task 14, transcribed all seven canonical §17
  fixture files verbatim, added the prescribed canonical loader test, and registered its
  module before the test run. `cargo test --locked knowledge::tests::canonical --
  --nocapture` genuinely ran and passed 1/1 (142 filtered out), proving the fixture loads
  and matches the prescribed assertions. The chained `cargo clippy --all-targets --locked
  -- -D warnings` then failed on the plan's literal `assert_eq!(example.solution.contains(
  "63"), true)` with `clippy::bool_assert_comparison`; Clippy requires
  `assert!(example.solution.contains("63"))`. Stopped rather than changing the locked test
  assertion or adding a lint exemption. The full knowledge/format gates were not run, no
  commit was made, and Task 15+ plus `knowledge-package/` remain untouched. The minimal
  correction is to update Task 14 Step 2's assertion to the Clippy-clean equivalent.
- 2026-08-30 (claude-code, plan Task 13 review / Task 14 pre-fix): Reviewed the Task 13
  implementation (commit `73ea7dd`) against plan Task 13 Steps 1–7. `tests/mod.rs` and
  `tests/conformance.rs` match the plan's prescribed cases verbatim (29 `#[test]` fns: 5 +
  7 + 6 + 11, matching the corrected Step 6 count); `mod tests;` is registered in
  `knowledge/mod.rs`. Re-ran the gates: full conformance corpus 29/29, `cargo test --locked
  knowledge::` 102/102, `cargo clippy --all-targets --locked -- -D warnings` clean, `cargo
  fmt --all --check` clean. **Task 13 is accepted.** This closes the mutation-based
  conformance workstream.
  Before issuing Task 14, checked it for the same class of defect that blocked Task 13:
  found it recurs identically. Task 14 Step 3 (as originally written) runs `cargo test
  --locked knowledge::tests::canonical` before Step 4 registers `#[cfg(test)] mod
  canonical;` in `tests/mod.rs`, so the un-declared module would make that run silently
  execute zero tests. Fixed the plan: the `mod canonical;` registration now happens
  immediately after Step 2 creates `canonical.rs`, before any test run; the old Step 3/4
  split collapses into one "run to verify it passes" step (renumbered Step 3), and the old
  Step 5 commit is now Step 4. Also noted Step 3's original "Expected: FAIL" text was
  already self-contradictory given this plan's own fixed ordering (Step 1's fixtures always
  precede Step 2's test), so the corrected step description states plainly that this is a
  pass-verification run, not a red step — consistent with Task 13 Step 2's precedent.
  **Task 14 is authorized** with this correction in place; no other content changed.
- 2026-08-30 (codex, plan Task 13 resumed): Verified corrective commit `85e1690`, kept
  the existing Step 1 support module and five tests unchanged, and added only the now-
  authorized `#[cfg(test)] mod tests;` registration. The corrected first checkpoint ran
  5/5 tests through the public loader. Appended the prescribed 7 reference/relationship
  cases, 6 provenance/schema cases, and 11 Example grammar/file-layout/frontmatter cases
  in order. The complete conformance corpus passed 29/29 with 113 tests filtered out, and
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings. The full
  `cargo test --locked knowledge::` gate passed 102/102 with 40 filtered out; `cargo fmt
  --all --check` passed after applying four mechanical rustfmt line wraps. Every expected
  `KnowledgeError` variant surfaced through `load_knowledge_package` as prescribed; no
  production code changed. Task 14+ and `knowledge-package/` remain untouched.
- 2026-08-30 (claude-code, plan Task 13 blocker triage): Confirmed the blocker is a real
  plan defect. Step 6 was the only place the plan declared `#[cfg(test)] mod tests;` in
  `mod.rs`, but Step 2 (and Step 3/4/5's incremental runs, had Codex tried them) invoke
  `cargo test --locked knowledge::tests::conformance` before Step 6 ever runs — Rust
  silently discovers zero tests under an undeclared module rather than erroring, so the
  run "succeeds" while proving nothing. Fixed the plan: Task 13 Step 1 now ends with adding
  `#[cfg(test)] mod tests;` to `mod.rs` (position in the file doesn't matter, only that it
  precedes the first `cargo test` of this task); Step 6 is retitled "Run the full corpus"
  and no longer re-adds the line. Also corrected Step 6's stale "~24 conformance tests"
  expectation to the exact count Codex computed (5 + 7 + 6 + 11 = 29). **Task 13 is
  re-authorized**: keep the five already-written tests and support module, add the
  `mod.rs` registration now, confirm the five pass, then proceed through Steps 3–7
  unchanged (corrected counts/step text only, no other content changed).
- 2026-08-30 (codex, plan Task 13 blocker): Task 12 is accepted and Task 13 authorized.
  Read the full task file and only plan Task 13, then created Step 1's exact support module
  and first five conformance tests without changing `knowledge/mod.rs`, as the prescribed
  ordering requires. The Step 2 command, `cargo test --locked
  knowledge::tests::conformance`, exited successfully but ran **0 tests** (113 filtered
  out), not the required five: Rust does not discover `src/knowledge/tests/mod.rs` until
  the parent `knowledge/mod.rs` declares `#[cfg(test)] mod tests;`, but the plan defers that
  declaration to Step 6, after every test-append step. Stopped rather than silently moving
  Step 6 earlier. The minimal correction is to move that registration into Step 1 before
  the first corpus run (it can still remain the last line of `mod.rs`). No remaining
  conformance cases were appended, no commit was made, and Task 14+ and
  `knowledge-package/` remain untouched. Separately, the prescribed corpus totals 29 tests
  (5 in Step 1, 7 in Step 3, 6 in Step 4, and 11 in Step 5), so the plan/prompt's “~24”
  expectation should be treated as a stale approximate count; this does not affect behavior.
- 2026-08-30 (claude-code, plan Task 12 review): Reviewed the Task 12 implementation
  (commit `58e4fea`) against plan Task 12 Steps 1–4. `loader.rs` matches the plan's
  prescribed test module and implementation verbatim; `mod.rs` matches Step 4's full
  replacement exactly — `#![allow(dead_code)]` is gone, and only `load_knowledge_package`,
  `related_concepts`, the domain/identifier types, and `KnowledgeError` are exported.
  Re-ran the gates: `cargo test --locked knowledge::` passes 73/73, `cargo clippy
  --all-targets --locked -- -D warnings` is clean with no dead-code allowance present,
  `cargo build --locked` succeeds, `cargo fmt --all --check` passes. **Task 12 is
  accepted.** This closes out the runtime module proper (Tasks 1–12) — the loader is now
  the crate's sole public entry point. Plan Task 13 (end-to-end conformance corpus) is
  authorized next. Task 13 adds no new production code and its own commit line
  (`tests/`, `mod.rs`) already covers every file it touches, so the Task 11 commit-scope
  gap doesn't apply here.
- 2026-08-30 (codex, plan Task 12): Task 11 is accepted and Task 12 authorized. Read the
  full task file and only plan Task 12 in full; its loader composition, final public API,
  three-test list, and commit scope are internally consistent. Added the prescribed loader
  tests before implementation and registered the private module for the red compile check.
  Task 13+ and `knowledge-package/` remain untouched.
- 2026-08-30 (codex, plan Task 12): The red run,
  `cargo test --locked knowledge::loader`, failed as expected because
  `load_knowledge_package` and the future outer-scope `KnowledgeError` import were not yet
  defined. Added the exact atomic loader chain and finalized the public module surface,
  removing Task 3's temporary module-root dead-code allowance rather than suppressing any
  newly exposed lint.
- 2026-08-30 (codex, plan Task 12): Green/gates complete. The final
  `cargo test --locked knowledge::` run passed all 73 tests—the prior 70 plus the 3 loader
  tests—with 0 failures and 40 filtered out. `cargo clippy --all-targets --locked -- -D
  warnings` passed with zero warnings without a dead-code allowance, `cargo build --locked`
  completed successfully, and `cargo fmt --all --check` passed. Built the one public,
  atomic loading boundary and exposed only that loader, `related_concepts`, domain types,
  identifiers, and `KnowledgeError`; all parsing/discovery/validation helpers remain
  internal. Task 13+ and `knowledge-package/` remain untouched.
- 2026-08-30 (claude-code, plan Task 11 review): Reviewed the Task 11 implementation
  (commit `272e73b`) against plan Task 11 Steps 1–4 and spec §10. `relationships.rs`
  matches the plan's prescribed test module and implementation verbatim (rustfmt wrapping
  only); the four new `KnowledgeError` variants and `Display` arms match Step 3 exactly;
  `mod relationships;` and `pub use relationships::related_concepts;` are wired in `mod.rs`
  per Step 4. Re-ran the gates: `cargo test --locked knowledge::` passes 70/70 (8 new
  relationship tests), `cargo clippy --all-targets --locked -- -D warnings` is clean,
  `cargo fmt --all --check` passes. **Task 11 is accepted.** Plan Task 12 (`loader.rs`,
  finalizing `mod.rs`'s public surface) is authorized next. Task 12's own file list
  (`loader.rs`, `mod.rs` only — no `error.rs`) matches its Step 6 commit line exactly, so
  the Task 11 commit-scope gap does not recur here; confirmed `KnowledgeError::
  MissingPackageToml`, which Task 12's tests construct, already exists from Task 9.
- 2026-08-30 (codex, plan Task 11 resumed): Verified commit `13cc869` corrected Step 6's
  commit scope to include `error.rs`; the issuing authorization also includes this Worklog.
  Added all eight prescribed relationship tests before implementation and registered the
  private module so Cargo compiles the red step. Task 12+ and `knowledge-package/` remain
  out of scope.
- 2026-08-30 (codex, plan Task 11 resumed): The red run,
  `cargo test --locked knowledge::relationships`, failed as intended because
  `validate_relationships`, `related_concepts`, `Concept` through the outer production
  import, and the four relationship error variants were undefined. Added the exact
  self/duplicate/existence validation order, reverse-authorship check, three-color DFS,
  normalized public query, error/Display additions, and public re-export.
- 2026-08-30 (codex, plan Task 11 resumed): Green/gates complete.
  `cargo test --locked knowledge::relationships` passed all 8 scoped tests, and the final
  `cargo test --locked knowledge::` run passed all 70 expected tests with 0 failures and 40
  filtered out. `cargo clippy --all-targets --locked -- -D warnings` passed with zero
  warnings, and `cargo fmt --all --check` passed after mechanical wrapping/module sorting.
  Built prerequisite/related self and duplicate checks, target resolution, reverse-related
  authorship rejection, three-color prerequisite cycle detection, and the normalized
  symmetric `related_concepts` public query. Task 12+ and `knowledge-package/` remain
  untouched.
- 2026-08-30 (claude-code, plan Task 11 blocker triage): Confirmed the blocker: Step 6's
  `git add` line for every plan Task that touches `error.rs` lists only the entity-specific
  file plus `mod.rs`, omitting `error.rs` itself even though the immediately preceding
  Step 3 always modifies it (this is a plan-wide pattern, not specific to Task 11 — e.g.
  Task 10's own Step 6 line had the same gap). It went unnoticed through Tasks 4–10 because
  each prior commit (see `8eb70e6` for Task 10: `error.rs`, `mod.rs`, `validate.rs`, and the
  task file) included `error.rs` anyway; my issuing prompt for Task 11 turned that
  previously-implicit judgment call into an explicit "commit only relationships.rs and
  mod.rs" instruction, which is what actually blocked Codex here. Fixed plan Task 11 Step 6
  to `git add` `relationships.rs`, `error.rs`, and `mod.rs` together, with a one-line note
  explaining why `error.rs` is required. Not touching the already-completed Tasks 4–10's
  Step 6 text since those tasks are done and their actual commits were already correct.
  **Task 11 is re-authorized**: implement relationships.rs and error.rs per Task 11 Steps
  1–5 unchanged, then commit `relationships.rs`, `error.rs`, `mod.rs`, and this task file's
  updated Worklog together with the prescribed message.
- 2026-08-30 (codex, plan Task 11 blocker): Read the full task file and only plan Task 11.
  The implementation scope requires adding four `KnowledgeError` variants in `error.rs`,
  and this task's handoff requires continuing the Worklog, but both plan Step 6 and the
  issuing prompt say to commit only `relationships.rs` and `mod.rs`. Such a commit is not
  self-contained or buildable on checkout: `relationships.rs` necessarily references the
  uncommitted variants, while the durable Worklog entry would also be omitted. Stopped
  before creating tests or implementation rather than choosing a different commit scope.
  Task 12+ and `knowledge-package/` remain untouched. The minimal correction is to authorize
  committing `relationships.rs`, `error.rs`, `mod.rs`, and this task file together.
- 2026-08-30 (claude-code, plan Task 10 review): Reviewed the Task 10 implementation
  (commit `8eb70e6`) against plan Task 10 Steps 1–4 and spec §9/§12. `validate.rs` matches
  the plan's prescribed test module and implementation verbatim (only rustfmt wrapping
  differs); the four new `KnowledgeError` variants and their `Display` arms in `error.rs`
  match Step 3's code exactly; `mod validate;` is registered in `mod.rs` with no `pub use`,
  as specified. Re-ran the gates: `cargo test --locked knowledge::` passes 62/62,
  `cargo clippy --all-targets --locked -- -D warnings` is clean, `cargo fmt --all --check`
  passes. **Task 10 is accepted.** Plan Task 11 (`relationships.rs`) is authorized next;
  its test-module import list was already pre-checked for the class of gap that hit Task 10
  (see the Task 10 blocker-triage entry below) and found clean, so no plan fix is needed
  before issuing it.
- 2026-08-30 (codex, plan Task 10): Task 9 is accepted and Task 10 authorized. Read only
  plan Task 10 in full and confirmed its cross-entity reference table, shared-lifetime
  helper signature, and five-test list are internally consistent. Added all five prescribed
  tests before implementation and registered the private `validate` module so Cargo
  compiles the red step; no Task 11+ or `knowledge-package/` work is in scope.
- 2026-08-30 (codex, plan Task 10): The red run,
  `cargo test --locked knowledge::validate`, failed as intended because
  `validate_references`, `DiscoveredEntities`/domain imports through the outer module, and
  the four reference error variants were undefined. Added the exact fixed-order validator,
  shared-lifetime provenance helper, and new error/Display additions; prerequisite/
  related relationship validation remains out of scope until Task 11.
- 2026-08-30 (codex, plan Task 10 blocker): The exact Step 1 tests plus Step 3
  implementation do not compile. The test helpers construct and return `Concept`,
  `Objective`, and `Example`, but the test module explicitly imports only
  `ProvenanceKind`/`ProvenanceRef`, and Step 3's outer production imports expose only
  `ProvenanceRef`/`Source` through `use super::*`. `cargo test --locked knowledge::`
  therefore fails with six E0422/E0425 errors for the three missing domain type names.
  The smallest correction is to import `Concept`, `Objective`, and `Example` inside the
  test module alongside its other test-only domain imports. Stopped without changing the
  locked import contract, without running later gates, and without committing; Task 11+
  and `knowledge-package/` remain untouched.
- 2026-08-30 (codex, plan Task 10 resumed): Verified plan-fix commit `39b8e71` and added
  the corrected test-only imports for `Concept`, `Example`, and `Objective`, leaving the
  validator implementation unchanged. The final `cargo test --locked knowledge::` run
  passed all 62 expected tests with 0 failures and 40 filtered out,
  `cargo clippy --all-targets --locked -- -D warnings` passed with zero warnings, and
  `cargo fmt --all --check` passed after mechanical iterator wrapping/module sorting. The
  prior import blocker is resolved; Task 11+ and `knowledge-package/` remain untouched.
- 2026-08-30 (claude-code, plan Task 10 blocker triage): Confirmed the blocker is a real
  plan defect, not a Codex error. `validate_references`'s production code in `validate.rs`
  never names `Concept`/`Objective`/`Example` — it only reaches them through
  `DiscoveredEntities` field access and inference — so `use super::*` cannot supply those
  three names to the test module regardless of what the outer scope imports. The plan's
  Step 1 test-module import line only listed `ProvenanceKind, ProvenanceRef`, missing the
  three types the helper functions construct by name. Fixed the plan (Task 10 Step 1) to
  import `Concept, Example, Objective, ProvenanceKind, ProvenanceRef` with a comment
  explaining why the glob import doesn't cover them. Proactively checked Task 11
  (`relationships.rs`) and Task 12 (`loader.rs`) test modules for the same class of gap —
  both are clean: their production code explicitly imports `Concept` and `KnowledgeError`
  respectively at outer scope, so `use super::*` does supply those names to their test
  modules. No other plan fix needed before re-issuing Task 10.
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
