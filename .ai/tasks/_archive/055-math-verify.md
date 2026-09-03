---
id: 055
title: math.verify capability (deterministic numeric + mathcore symbolic-expression)
status: done
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

Reviewed by Claude (`/code-review`, high effort, 4 finder angles + manual verification +
actual `cargo test`/`cargo clippy` run) against commit `215c0fe` on `master`. No blocking
findings.

- [x] Correctness — pass. Tolerance/comparison logic (`provider.rs:81-88`) matches spec §5
      exactly, including the `is_finite` guard on both operands (fixes the failure mode a
      naive port would hit if `canonical` were ever non-finite). `mathcore` usage is
      strictly limited to `MathCore::new()`/`.calculate()` — verified no
      `differentiate`/`integrate`/`solve`/`simplify`/matrix calls anywhere in the diff.
      `CapabilityProvider`/`InvocationError` mapping follows `registry.rs`'s established
      convention exactly. `VerifyRequest`/`VerifyResult` match the spec's contract
      field-for-field. 204/204 tests pass (26 new), `cargo clippy -D warnings` clean.
- [x] Architecture conformance — pass, with one process note (not a code defect): this
      commit (owned by `codex`) edits `ARCHITECTURE.md:72` directly, which `CLAUDE.md`
      reserves to Claude ("anything touching `ARCHITECTURE.md`, `AGENTS.md`, or `.ai/`
      itself"). The edit itself is correct and mechanical — one line adding `capabilities/`
      to the folder tree, reflecting a placement already locked in the design spec §3 — so
      no fix needed here, but future structural-doc updates should be surfaced for Claude's
      sign-off rather than merged directly, per the coordination rule.
- [ ] UI rules — N/A, no frontend/UI touched by this task.
- [x] Process — pass. Worklog reflects what happened in enough detail to follow without the
      diff (including the `mathcore` `std`-feature plan correction). Scope matches the task
      as created — no unrequested expansion. `ARCHITECTURE.md` updated for the new
      top-level directory.

Two non-blocking findings, both confirmed by direct reading (`provider.rs`), moved to
Follow-ups below rather than fixed here per this repo's review convention (reviewer records,
original author or a follow-up task applies the fix).

Verdict: no blocking findings — moving to `done`.

## Follow-ups

- `provider.rs:67` — `student_response` is passed into `mathcore`'s recursive-descent
  parser/evaluator with no length or nesting bound. Not a logic bug today (this capability
  isn't wired to any live input path yet), but once Practice actually feeds real learner
  input through it, a pathological string (e.g. deeply nested parentheses) could drive
  excessive stack depth/CPU before `verify()` ever returns `is_correct: false`. Revisit when
  wiring this capability to a real caller (see the app-startup-registration follow-up below).
- `provider.rs:60-65` — `canonical_solution.clone()` inside the `map_err` closure is
  unnecessary: `canonical_solution` is never read again after this point (the `?` returns
  immediately on error, and the success path only uses `student_response`), so a `move`
  closure could take ownership directly instead of cloning. Cosmetic/efficiency only.
- Formula-shaped symbolic answers with genuine free variables (domain-sampling equivalence
  checking) — not needed until a problem family actually requires one (spec §8).
- Symbolic-exactness enforcement (rejecting a numerically-correct decimal approximation for
  a `SymbolicExpression` problem) if it turns out to matter pedagogically (spec §8).
- Validate at Knowledge-authoring time that `CanonicalSolution::Symbolic`'s `expression`
  string is actually `mathcore`-parseable, instead of only discovering a broken authored
  expression at verification time (spec §8).
- Wire `math.verify` into a real running `ModuleRegistry` at app startup — command/app-layer
  work, deliberately out of scope for this task.
