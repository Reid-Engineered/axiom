---
id: 055
title: math.verify capability (deterministic numeric + mathcore symbolic-expression)
status: proposed
owner: codex
stage: 8
depends_on: [054]
---

## Scope

Add a first-party `math.verify` v1 capability under a new `src-tauri/src/capabilities/`
module: given a `ProblemInstance`'s (`src-tauri/src/knowledge/types.rs`) `response_type` and
`canonical_solution` plus a student's response, decide correctness. Sub-project 2 of 6 in
the "Practice engine" initiative — `ROADMAP.md` Stage 8, following directly on the canonical
Problem schema (`.ai/tasks/_archive/054-canonical-problem-schema.md`, merged) and the
module-capability runtime (tasks 045–048, done). This is the first concrete
`CapabilityProvider` built against that runtime.

Full design: `docs/superpowers/specs/2026-09-02-math-verify-design.md`.
Full task breakdown: `docs/superpowers/plans/2026-09-02-math-verify.md`.

Does not build: partial credit, error classification, or any diagnostic reasoning beyond a
correctness boolean (Practice's concern, sub-project 4); the generator functions that
produce `ProblemInstance` values (sub-project 3); any Tauri command or app-startup wiring
that registers this capability into a real running `ModuleRegistry` (deliberately deferred —
see the plan's Global Constraints); any UI (sub-projects 5–6).

## Plan

Files to be created or touched (all new — no existing file outside `Cargo.toml`/`lib.rs` is
modified; see the plan's 7 tasks for exact per-file, per-step detail):
- New dependency: `mathcore = "=0.3.1"` (MIT, `default-features = false, features = ["std"]`)
  in `src-tauri/Cargo.toml`.
- Modify: `src-tauri/src/lib.rs` (add `pub mod capabilities;`).
- Create: `src-tauri/src/capabilities/mod.rs`,
  `src-tauri/src/capabilities/math_verify/{mod.rs,types.rs,error.rs,provider.rs,module.toml}`,
  `src-tauri/src/capabilities/math_verify/tests/mod.rs`.

## Worklog

- 2026-09-02 — Brainstormed with Marcus. Symbolica (roadmap's original CAS choice) was ruled
  out over licensing/offline-activation risk for a no-accounts desktop app. Two open-source
  Rust CAS alternatives were checked and rejected (`Symmetrica`: archived/dead; `cas-rs`:
  actively maintained but self-described "very early stage"). `mathcore` (MIT, 127 stars) was
  chosen deliberately despite no commits since its initial release — mitigated by using only
  its narrow `MathCore::calculate` parse+evaluate surface, never its differentiation/
  integration/solve/matrix code. Spec and plan written by Claude (architect role per
  `AGENTS.md`), handed to Codex for implementation.

## What was built / tested / left out

(filled in when moving to review)

## Review

(filled in when a reviewer picks this up)

## Follow-ups

- Formula-shaped symbolic answers with genuine free variables (domain-sampling equivalence
  checking) — not needed until a problem family actually requires one (spec §8).
- Symbolic-exactness enforcement (rejecting a numerically-correct decimal approximation for
  a `SymbolicExpression` problem) if it turns out to matter pedagogically (spec §8).
- Validate at Knowledge-authoring time that `CanonicalSolution::Symbolic`'s `expression`
  string is actually `mathcore`-parseable, instead of only discovering a broken authored
  expression at verification time (spec §8).
- Wire `math.verify` into a real running `ModuleRegistry` at app startup — command/app-layer
  work, deliberately out of scope for this task.
