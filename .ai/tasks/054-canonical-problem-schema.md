---
id: 054
title: Canonical Problem schema (ProblemFamily/ProblemInstance)
status: review
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

## What was built / tested / left out

PR: [#3 — feat: Canonical Problem schema (#054)](https://github.com/Reid-Engineered/axiom/pull/3)

Built the canonical `ProblemFamily` authored schema, `ProblemInstance` runtime schema,
constraint expression parser, TOML/frontmatter and Markdown-body parsing, structural and
cross-entity validation, package discovery/loading, public exports, and the migrated
canonical shell-method fixture.

Tested with the WSL Rust toolchain against the Windows checkout:
- `cargo check --locked` — pass
- `cargo test --locked` — pass, 164 tests
- `cargo clippy --all-targets --locked -- -D warnings` — pass
- `cargo fmt --all --check` — pass

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

## Follow-ups

- Review finding 6: share the parameter-reference and prerequisite cycle detection instead
  of maintaining separate White/Gray/Black DFS implementations in `problem_family.rs` and
  `relationships.rs`. Preserve both error contexts and cycle-path diagnostics.
- Review finding 7: replace `conformance_problem_family.rs`'s recursive `copy_tree()`
  fixture setup with the existing `conformance.rs` / `write_base_package` pattern,
  retaining canonical-fixture coverage in the canonical suite.
- Implement generator functions and domain-validity property tests.
- Add `math.verify`, Practice Core integration, Study Session UI integration, and the
  offline acceptance test in the follow-up sub-projects listed by the design spec.
