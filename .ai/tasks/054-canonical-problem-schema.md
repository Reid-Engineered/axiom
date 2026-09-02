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

(filled in by reviewer)

## Follow-ups

- Implement generator functions and domain-validity property tests.
- Add `math.verify`, Practice Core integration, Study Session UI integration, and the
  offline acceptance test in the follow-up sub-projects listed by the design spec.
