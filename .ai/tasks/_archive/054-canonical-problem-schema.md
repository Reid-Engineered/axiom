---
id: 054
title: Canonical Problem schema (ProblemFamily/ProblemInstance)
status: done
owner: codex
stage: 8
depends_on: []
---

## Scope

Add `ProblemFamily` (authored, generative-problem-template Knowledge Package content) and
`ProblemInstance` (runtime generation-output shape) to `src-tauri/src/knowledge/`, following
every convention that module already established for `Concept`/`Objective`/`Example`.
Sub-project 1 of 6 in the "Practice engine" initiative — the remaining scope of `ROADMAP.md`
Stage 8, after the module/capability runtime (tasks 045–048, done).

Full design: `docs/superpowers/specs/2026-09-01-canonical-problem-schema-design.md`.
Full task breakdown: `docs/superpowers/plans/2026-09-01-canonical-problem-schema.md`.

Does not build: `math.verify` (sub-project 2), the actual generator functions or domain-
validity property tests (sub-project 3), the Practice Core Utility (sub-project 4), Study
Session UI integration (sub-project 5), or the offline acceptance test (sub-project 6) — all
tracked as follow-ups in the spec's §8.

## Plan

Files to be created or touched (all within `src-tauri/src/knowledge/`, no frontend or
command-layer change — see the plan's 7 tasks for exact per-file, per-step detail):
- Create: `constraint.rs`, `problem_family.rs`, `problem_family_body.rs`
- Modify: `identifier.rs`, `types.rs`, `raw.rs`, `discover.rs`, `validate.rs`, `loader.rs`,
  `mod.rs`, `error.rs`
- Create fixtures/tests: `tests/fixtures/canonical/problems/problem.shell_y_poly.md`, a
  conformance test file (exact location determined by Task 7 Step 1 — read the existing
  conformance suite's structure before adding to it)

## Worklog

- 2026-09-01 — spec and plan written by Claude (architect role per AGENTS.md), handed to
  Codex for implementation
- 2026-09-01 — Codex began implementation on `agent/codex/054-canonical-problem-schema`.
  Windows has no native Cargo installation; WSL access requires an escalated sandbox call.
  The Rust toolchain is healthy in Ubuntu and tests run against this Windows checkout via
  `/mnt/c`, with disposable build output in WSL `/tmp`. The unrelated untracked `archify/`
  directory was preserved.
- 2026-09-01 — Deviated from the plan's sample generator IDs (`gen-shell-y-poly`/`gen-a`):
  they violate the identifier grammar the same plan requires. Fixtures/tests use the valid
  `gen.shell_y_poly`/`gen.a` forms instead.
- 2026-09-01 — `ResolvedSolution` uses adjacent Serde tagging (`kind` + `value`) because
  Serde's internally tagged representation does not support the spec's tuple variants;
  this preserves the intended explicit variant discriminator and compiles cleanly.
- 2026-09-01 — Codex self-review found and fixed incomplete constraint parsing/wiring,
  missing parameter-cycle and constraint-reference checks, permissive body-section parsing,
  and missing conformance coverage. Final self-check found no remaining blocking findings.
- 2026-09-01 — Initial CI found the documented `GoalEditingSheet.test.tsx` frontend flake
  on Ubuntu and a Windows-only CRLF assumption in the new hint-count conformance mutation.
  Made the mutation line-ending agnostic; no production code changed.
- 2026-09-02 — Addressed review findings 1–5 on the same branch: bounded parenthesis and
  unary-minus recursion at 64 levels, added unary negation with atom precedence, required
  at least one objective, rejected zero hint levels, and rejected non-finite parameter
  literals and reference offsets in value/min/max conversion. Added contextual error
  variants and preserved the review recorded on `origin/master` below.
- 2026-09-02 — Regression coverage includes accepted nesting at the limit, rejection just
  past the limit and at 10,000 levels, unary-minus precedence and malformed operands,
  empty/omitted objectives, zero hint levels, and all signed TOML nan/inf forms across
  value/min/max literals and reference offsets. Finite bounds and negative constraints
  remain accepted. Recorded findings 6–7 as follow-ups without refactoring them here.
- 2026-09-02 — WSL validation against this checkout passed: `cargo test --locked --quiet`
  (171 tests), `cargo check --locked --quiet`,
  `cargo clippy --all-targets --locked --quiet -- -D warnings`,
  `cargo fmt --all --check`, and `git diff --check`. Native E2E was not run locally:
  WSL lacks `WebKitWebDriver` and `tauri-driver`; PR #3's required CI E2E job remains the
  merge gate for that flow.

- 2026-09-02 — Merged `origin/master` (`3f05bd7`) into the PR branch and resolved the
  task-document conflict. Kept master's original Claude review as the base and appended
  the branch's second-review Codex outcomes unchanged. Retained `status: review`, the
  complete worklog/follow-ups, and the current 178-test summary; 164 and 171 were earlier
  revision counts. No Rust files changed. Re-ran `cargo test --locked` (178 passed),
  `cargo clippy --all-targets --locked -- -D warnings`, `cargo fmt --all --check`, and
  `git diff --check`; all passed.

## What was built / tested / left out

PR: [#3 — feat: Canonical Problem schema (#054)](https://github.com/Reid-Engineered/axiom/pull/3)

Built the canonical `ProblemFamily` authored schema, `ProblemInstance` runtime schema,
constraint expression parser, TOML/frontmatter and Markdown-body parsing, structural and
cross-entity validation, package discovery/loading, public exports, and the migrated
canonical shell-method fixture.

Latest validation (2026-09-02, second review fixes) with the WSL Rust toolchain against
the Windows checkout; earlier worklog counts describe their respective revisions:
- `cargo check --locked` — pass
- `cargo test --locked` — pass, 178 tests
- `cargo clippy --all-targets --locked -- -D warnings` — pass
- `cargo fmt --all --check` — pass
- `git diff --check` — pass

The conformance coverage includes every planned full-loader rejection class. Generator
execution, `math.verify`, Practice Core, UI integration, and offline acceptance remain out
of scope as specified.

## Review

Reviewed by Claude (`/code-review`, high effort, 8 finder angles + verification) against
PR #3 (`agent/codex/054-canonical-problem-schema`, commit `00d252d`). 7 findings, all
confirmed by direct reading of the implementation, not agent speculation.

**Correctness — blocking, should be fixed before merge:**

1. `src-tauri/src/knowledge/constraint.rs:210` — `parse_atom`'s `LParen` branch recurses
   into `parse_term` with no depth limit. A deeply nested parenthesized constraint string
   overflows the stack and crashes the process instead of returning a parse error.
2. `src-tauri/src/knowledge/constraint.rs:206` — `parse_atom` has no case for a leading
   `Token::Minus`. Constraints like `"b >= -5"` fail to parse even though `Term::Literal(f64)`
   can represent negative values — tokenizes to `[..., Ge, Minus, Number(5.0)]` and the
   right-hand `parse_atom` hits the `other => Err(...)` catch-all.
3. `src-tauri/src/knowledge/problem_family.rs:29` — `objective_ids` is never checked for
   non-emptiness, despite the design spec §3 explicitly documenting the field as non-empty.
   `raw.rs`'s `#[serde(default)]` lets it default to an empty `Vec`; no `KnowledgeError`
   variant even models this case.
4. `src-tauri/src/knowledge/problem_family.rs:95` — hint `level` is checked for uniqueness
   (`seen_levels`) but never for positivity, despite spec §6 requiring "unique, positive
   integers." `RawHint.level: u32` accepts `0`.
5. `src-tauri/src/knowledge/raw.rs:103` — `RawBound::Literal(f64)` / `Bound::Literal` never
   rejects `NaN`/`Infinity`, which TOML's float grammar allows authors to write directly as
   `nan`/`inf`/`-inf`. No downstream check in `convert_bound` or
   `validate_parameter_references` catches it.

**Reuse — non-blocking, worth a follow-up:**

6. `src-tauri/src/knowledge/problem_family.rs:194` — `validate_parameter_references`
   reimplements `relationships.rs`'s White/Gray/Black three-color DFS cycle detection inline
   instead of sharing it. Two independent finder angles flagged this independently.
7. `src-tauri/src/knowledge/tests/conformance_problem_family.rs:12` — `copy_tree()` hand-rolls
   a full recursive directory copy, run once per test (11×), instead of following the
   existing `conformance.rs` suite's lighter in-memory `write_base_package` pattern.

**Explicitly checked and NOT findings** (ruled out during verification, noted so they aren't
re-litigated): `provenance_refs` non-emptiness *is* enforced correctly via the shared
`convert_provenance_refs` helper; the manual `String → enum` matching for `response_type`/
`status`/provenance `kind` matches this module's own established convention, not a deviation;
the design spec's numbered-list hint example (§5) is a stale doc inconsistency — the actual
fixture and parser agree on dash-bullet format, so no code bug there.

### 2026-09-02 — Codex outcomes for the second review (against `bad967b`)

The numbered outcomes below correspond to the eight findings in Marcus's follow-up.
They are implementation responses, not a replacement for the existing review or an
independent reviewer verdict.

1. Fixed flat operator chains in `constraint.rs`. Each parsed subtree carries its actual
   depth; every binary operation checks `1 + max(left, right)` against 64 before building
   a new node. The separate recursion guard still bounds parentheses/unary minus. This
   bounds validation and recursive drop as well as parsing. Regression:
   `long_flat_operator_chains_are_rejected_end_to_end` rejects 200,000-term chains for
   addition, subtraction, multiplication, and division through the package loader.
   `bounds_actual_tree_depth_across_operators_and_grouping` checks the accepted boundary
   and rejection when another operator, grouped subtree, or unary minus exceeds it.
2. Fixed non-finite numeric canonical solutions in `convert_canonical_solution`, returning
   `NonFiniteCanonicalSolution` with the family ID. Regression:
   `numeric_canonical_solution_must_be_finite_end_to_end` rejects all signed TOML nan/inf
   forms and verifies finite negative, zero, and positive solutions remain accepted.
3. Fixed unknown reference-bound fields with Serde's container-level
   `#[serde(untagged, deny_unknown_fields)]` on `RawBound`, which applies to the Reference
   struct variant. Regression: `unknown_bound_reference_fields_are_rejected_end_to_end`
   rejects both `offst` and an extra key alongside a correctly spelled `offset`.
4. Fixed unreferenceable parameter names by validating through the constraint tokenizer:
   exactly one identifier token, preserving the full authored name. This admits
   `[A-Za-z_][A-Za-z0-9_]*` except the reserved `and`, rather than imposing dotted entity
   IDs on existing parameter names. Regression:
   `parameter_names_must_be_referenceable_end_to_end` rejects keywords, empty names,
   leading digits, punctuation, whitespace, and Unicode; accepted names are exercised in
   real constraints, including conjunctions.
5. Fixed hint ordering with `OutOfOrderHintLevel` after positivity and uniqueness checks.
   Spec §3 says hints are ordered by level and levels ascend; body pairing is preserved
   without sorting the metadata separately. Regression:
   `hint_levels_must_ascend_in_body_order_end_to_end` rejects descending metadata and
   verifies ascending levels with gaps retain their original bullet text.
6. Deferred shared section-parser extraction to a dedicated refactor, as permitted for
   this non-blocking finding. Both parsers retain their current tested behavior; the
   extraction must preserve their distinct error mapping and empty-Hints behavior.
   Tracked explicitly under Follow-ups below; no production change claimed here.
7. Replaced manual generator validation with `knowledge_id!(GeneratorId)`, exported it,
   and used it in `GeneratorRef` and family construction. Regression:
   `generator_ids_are_validated_end_to_end`, the extended identifier acceptance/rejection
   tests, and `problem_family_round_trips_through_json` cover grammar and unchanged JSON
   string representation. All existing consumers compile with the typed ID.
8. Reconciled the current validation summary to **178 tests**. The earlier 171-test
   worklog remains historical evidence for `bad967b`, explicitly distinguished from the
   latest run above. Verified the summary against the actual `cargo test --locked` result;
   no code test was added for this documentation-only correction.

Validation after these changes: `cargo test --locked` passed all 178 tests;
`cargo check --locked`, `cargo clippy --all-targets --locked -- -D warnings`,
`cargo fmt --all --check`, and `git diff --check` passed. Native E2E remains assigned to
PR #3's required CI job because the local WSL environment lacks `WebKitWebDriver` and
`tauri-driver`. No directory layout or data-flow rule changed. The section-parser
extraction and the earlier reuse follow-ups remain deliberately deferred.

- 2026-09-02 — Merged `agent/codex/054-canonical-problem-schema` directly into `master`
  (no PR — repo has switched to direct-merge workflow until there's a working app end to
  end). Clean merge, no conflicts. Re-verified independently on `master` post-merge:
  `cargo test --locked` (178 passed), `cargo clippy --all-targets --locked -- -D warnings`,
  `cargo fmt --all --check`, and `git diff --check` all pass. All 5 blocking findings from
  the original review confirmed fixed by direct code reading (not just worklog claims).
  Marking done and archiving.

## Follow-ups

- Second review finding 6: extract a shared section parser for `problem_family_body.rs`
  and `example_body.rs`, parameterized by recognized/required headings and error mapping.
  Preserve error precedence, section order, whitespace/content handling, and the distinct
  empty-Hints rules; run both existing body suites and add cross-parser regression cases.
- Review finding 6: share the parameter-reference and prerequisite cycle detection instead
  of maintaining separate White/Gray/Black DFS implementations in `problem_family.rs` and
  `relationships.rs`. Preserve both error contexts and cycle-path diagnostics.
- Review finding 7: replace `conformance_problem_family.rs`'s recursive `copy_tree()`
  fixture setup with the existing `conformance.rs` / `write_base_package` pattern,
  retaining canonical-fixture coverage in the canonical suite.
- Implement generator functions and domain-validity property tests.
- Add `math.verify`, Practice Core integration, Study Session UI integration, and the
  offline acceptance test in the follow-up sub-projects listed by the design spec.
