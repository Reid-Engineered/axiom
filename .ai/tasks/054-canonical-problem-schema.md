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

(filled in by reviewer)

## Follow-ups

(filled in when moving to review — see spec §8 for known ones)
