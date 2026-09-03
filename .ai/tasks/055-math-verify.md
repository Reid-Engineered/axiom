---
id: 055
title: math.verify capability (deterministic numeric + mathcore symbolic-expression)
status: review
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

Implementation files created or touched (all new — no existing production file outside
`Cargo.toml`/`lib.rs` is modified; see the plan's 7 tasks for exact per-file, per-step detail):
- New dependency: `mathcore = "=0.3.1"` (MIT, `default-features = false`; the published
  crate has no `std` feature)
  in `src-tauri/Cargo.toml`.
- Modify: `src-tauri/src/lib.rs` (add `pub mod capabilities;`).
- Create: `src-tauri/src/capabilities/mod.rs`,
  `src-tauri/src/capabilities/math_verify/{mod.rs,types.rs,error.rs,provider.rs,module.toml}`,
  `src-tauri/src/capabilities/math_verify/tests/mod.rs`.
- Documentation bookkeeping: this task file, the implementation plan correction, and the
  `ARCHITECTURE.md` folder tree required for the new top-level backend directory.

## Worklog

- 2026-09-02 — Brainstormed with Marcus. Symbolica (roadmap's original CAS choice) was ruled
  out over licensing/offline-activation risk for a no-accounts desktop app. Two open-source
  Rust CAS alternatives were checked and rejected (`Symmetrica`: archived/dead; `cas-rs`:
  actively maintained but self-described "very early stage"). `mathcore` (MIT, 127 stars) was
  chosen deliberately despite no commits since its initial release — mitigated by using only
  its narrow `MathCore::calculate` parse+evaluate surface, never its differentiation/
  integration/solve/matrix code. Spec and plan written by Claude (architect role per
  `AGENTS.md`), handed to Codex for implementation.
- 2026-09-02 — Codex started implementation directly on `master` per the current workflow.
  Confirmed the checked-in registry interfaces and preserved the unrelated untracked
  `.claude/` directory.
- 2026-09-02 — The plan's dependency declaration did not resolve: published `mathcore`
  0.3.1 has no `std` feature. Confirmed from `cargo info` and the downloaded crate manifest
  that its optional features are `parallel`, `fft`, `full`, and `wasm`; retained the intended
  minimal build as `default-features = false` with no feature list.
- 2026-09-02 — Implemented the complete capability and embedded manifest. The focused suite
  passes with 26 tests. This corrects the plan's stated total of 30 (its listed groups total
  23) by adding the spec-required `tan` case, both tolerance-boundary directions, and an
  unknown-version case. Non-finite numeric input is exercised against the typed verifier
  because JSON cannot represent `NaN` or infinity.
- 2026-09-02 — Final validation passed: `cargo check --locked`, all 204 Rust tests,
  Clippy with warnings denied, rustfmt, and `git diff --check`. The first native E2E attempt
  could not launch because the host lacked both drivers. Installed `tauri-driver` and the
  Ubuntu WebKit driver into a disposable `/tmp` tree, reran `npm run test:e2e:linux`, and
  both native flows passed; the temporary tree was then removed.

## What was built / tested / left out

Built the `math.verify` v1 provider with typed numeric/symbolic requests, correctness
results, canonical-expression failure handling, the exact absolute/relative tolerance
comparison, narrow `MathCore::calculate` symbolic evaluation, and an embedded first-party
manifest. Added the `capabilities/` module to the backend architecture tree. Corrected the
implementation plan and task dependency text after confirming published `mathcore` 0.3.1
has no `std` feature; its default Rayon/FFT features remain disabled and absent from the
dependency graph.

Validation on 2026-09-02:

- `cargo check --locked --quiet` — pass
- `cargo test --locked --quiet` — pass, 204 tests total (26 new capability tests)
- `cargo clippy --all-targets --locked --quiet -- -D warnings` — pass
- `cargo fmt --all --check` — pass
- `git diff --check` — pass
- `npm run test:e2e:linux` — pass, 2 native flows (using disposable local driver installs)

Partial credit, diagnostics/hint selection, free-variable equivalence, generator functions,
Practice Core/app-startup registration, Tauri commands, and UI remain out of scope as
specified.

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
