---
id: 056
title: Problem generation engine (deterministic seeded sampling + domain-validity property test)
status: proposed
owner: codex
stage: 8
depends_on: [054, 055]
---

## Scope

Add `generate_problem_instance(family: &ProblemFamily, seed: u64) -> Result<ProblemInstance,
GenerationError>` under a new `src-tauri/src/generation/` module: given an authored
`ProblemFamily` (`src-tauri/src/knowledge/types.rs`) and a seed, deterministically sample
parameters, substitute them into the prompt/hints/canonical solution, and produce a concrete
`ProblemInstance`. Sub-project 3 of 6 in the "Practice engine" initiative — `ROADMAP.md`
Stage 8, following the canonical Problem schema
(`.ai/tasks/_archive/054-canonical-problem-schema.md`) and `math.verify`
(`.ai/tasks/_archive/055-math-verify.md`), both merged.

Full design: `docs/superpowers/specs/2026-09-02-problem-generation-design.md`.
Full task breakdown: `docs/superpowers/plans/2026-09-02-problem-generation.md`.

Does not build (per brainstorming — engine-only scope decision): authoring real
`ProblemFamily` content into `knowledge-package/`'s reference package (proven against the
existing test fixture instead); the Practice Core Utility that will call this engine
(sub-project 4); bespoke non-generic generator logic for hypothetical future families beyond
the one generic pipeline; any Tauri command, frontend, or UI (sub-projects 5–6).

## Plan

Files to be created or touched (all new except two small additions to existing files — see
the plan's 8 tasks for exact per-file, per-step detail):
- Create: `src-tauri/src/generation/{mod.rs,error.rs,rng.rs,sampling.rs,template.rs}`,
  `src-tauri/src/generation/tests/mod.rs`.
- Modify: `src-tauri/src/lib.rs` (add `pub mod generation;`).
- Modify: `src-tauri/src/knowledge/constraint.rs` (add `Term::evaluate`/
  `ConstraintExpr::holds` — the module currently only parses constraints, never evaluates
  them; this is a genuine gap this task fills rather than duplicating in `generation/`).
- Modify: `src-tauri/src/knowledge/mod.rs` (export `parse_constraint` so `generation/`'s
  tests can construct constraints without hand-writing AST literals).
- No new Cargo dependency — hand-rolled deterministic PRNG (SplitMix64) and a hand-rolled
  seed-loop property test, per the brainstorming decision not to add `rand` or `proptest`.

## Worklog

- 2026-09-02 — Brainstormed with Marcus. Confirmed engine-only scope (no new reference-
  package content), hand-rolled PRNG over `rand`, and a hand-rolled property-test loop over
  `proptest` — consistent with this project's pattern of avoiding new dependencies where a
  small, auditable, well-known algorithm suffices. Investigation during brainstorming found
  `constraint.rs` has never had an evaluator (only a parser) — this task is the first real
  consumer that needs one. Also found the one real example family's domain-validity
  requirement is already fully captured by its existing declarative bounds, meaning the
  sampling engine can be fully generic (data-driven from the schema) rather than needing
  bespoke per-family Rust code — what's genuinely bespoke per family is the property test
  itself, not the sampling mechanism. Spec and plan written by Claude (architect role per
  `AGENTS.md`), handed to Codex for implementation.

## What was built / tested / left out

(filled in when moving to review)

## Review

(filled in when a reviewer picks this up)

## Follow-ups

- Knowledge-authoring-time validation (task 054's module) should require every
  `ParameterSpec` have either a fixed `value` or both `min` and `max` — today's schema
  allows an unsampleable parameter to pass authoring-time validation, only failing at
  generation time instead (spec §9).
- `CanonicalSolution::Numeric` has no expression/template mechanism, so a `Numeric`-response
  family's canonical answer cannot depend on its own sampled parameters — a real limitation
  of the already-locked schema, not a problem for the one real example family (spec §9).
- Real reference-package content authoring (`knowledge-package/problems/`) — deferred per
  the engine-only scope decision.
- Bespoke non-generic generator functions for a future family whose domain-validity can't be
  expressed as declarative bounds/constraints — the `match`-based dispatch in `mod.rs` is
  built to accept new arms without redesign.
- Sub-project 4 (Practice Core Utility, the actual caller of this engine), sub-projects 5–6
  (Study Session UI, offline acceptance test).
