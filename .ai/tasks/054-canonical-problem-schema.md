---
id: 054
title: Canonical Problem schema (ProblemFamily/ProblemInstance)
status: proposed
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

## What was built / tested / left out

(filled in when moving to review)

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

(filled in when moving to review — see spec §8 for known ones)
