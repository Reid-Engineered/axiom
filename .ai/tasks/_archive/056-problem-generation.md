---
id: 056
title: Problem generation engine (deterministic seeded sampling + domain-validity property test)
status: done
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
- 2026-09-02 — Codex started implementation directly on `master` per the current workflow.
  Confirmed local `master` matches `origin/master`, dependencies 054/055 are done, and the
  unrelated untracked `.claude/` directory remains out of scope.
- 2026-09-02 — Corrected one additional flaky assertion from the plan: the canonical hint
  fixture intentionally contains TeX braces, so the full-pipeline test checks specifically
  for unresolved declared-parameter placeholders instead of rejecting every `{` character.
  Number formatting also uses `f64::to_string()` directly so large whole values are not
  truncated through an `i64` cast.
- 2026-09-02 — Focused tests exposed an unused-import warning for the test-only
  `parse_constraint` re-export; gated that re-export with `cfg(test)`. Self-review also
  hardened identifier substitution so negative values remain grouped under exponentiation
  and parameter names cannot be substituted inside scientific-notation literals.
- 2026-09-02 — Strengthened the narrow-constraint test with a seed whose first draw is known
  to fail and second draw succeeds, so it exercises actual reject-and-resample behavior
  rather than merely accepting a lucky first draw.
- 2026-09-02 — Fixed a second Rust module-wiring issue in the plan before handoff: its inline
  entry-point tests and external property-test directory were both named `tests`, which
  cannot coexist in one module. The implementation and corrected plan use `unit_tests` for
  the inline suite and `tests` for the property suite.
- 2026-09-02 — Final validation passed: `cargo check --locked`, all 235 Rust tests,
  Clippy with warnings denied, rustfmt, and `git diff --check`. Installed `tauri-driver` and
  the Ubuntu WebKit driver into a disposable `/tmp` tree for the native gate;
  `npm run test:e2e:linux` passed both flows, and the temporary tree was removed.
- 2026-09-02 — Codex began the requested review-fix pass after confirming the blocking
  finding remained live in `dc05aa1`. Added explicit reversed-range rejection to both RNG
  sampling helpers, widened integer range arithmetic through `i128`/`u128`, and mapped
  invalid resolved bounds to contextual `GenerationError::InvalidParameterBounds`.
- 2026-09-02 — Self-review moved the authoritative reversed-bound check ahead of the
  `f64`-to-`i64` conversion, preventing very large reversed integer bounds from collapsing
  to the same saturated integer and bypassing the RNG-level guard.
- 2026-09-02 — Review-fix validation passed: `cargo check --locked`, all 240 Rust tests,
  Clippy with warnings denied, rustfmt, and `git diff --check`. Five new regressions cover
  reversed integer/float RNG ranges without state consumption, full-width `i64` sampling,
  contextual error display, reference-offset reversal, and pre-conversion reversal. The
  native E2E suite passed three consecutive runs after separate test-infrastructure fixes;
  ESLint passed and the final run left no isolated app-data directory behind.

## What was built / tested / left out

Built deterministic SplitMix64 sampling, recursive dependency-aware parameter resolution,
constraint evaluation and capped reject-and-resample behavior, prompt/hint and symbolic
expression substitution, generator dispatch, and the public `generate_problem_instance`
entry point. Added the 10,000-seed `problem.shell_y_poly` domain-validity test and updated
the backend architecture tree for the new top-level `generation/` module. No production or
development dependency was added.

Validation on 2026-09-02:

- `cargo check --locked --quiet` — pass
- `cargo test --locked --quiet` — pass, 240 tests total (36 new tests)
- `cargo clippy --all-targets --locked --quiet -- -D warnings` — pass
- `cargo fmt --all --check` — pass
- `git diff --check` — pass
- `npm run test:e2e:linux` — pass, 2 native flows (using disposable local driver installs)

Real reference-package authoring, Knowledge-authoring-time completeness validation,
parameter-dependent numeric canonical solutions, bespoke generators, Practice Core/Tauri
command wiring, frontend integration, and UI remain out of scope as specified.

## Review

Reviewed by Claude (`/code-review`, high effort, cross-file + manual verification) against
commit `dc05aa1` on `master`. One blocking finding.

- [x] Correctness — FAIL, then fixed and independently re-verified (see below):
      `src-tauri/src/generation/rng.rs:18` — `sample_integer(min, max)`
      has no guard against `min > max`. `range = (max - min + 1) as u64` becomes `0` (a
      `next_u64() % 0` panic) when `min == max + 1`, or wraps into an enormous `u64` via the
      negative-to-unsigned cast when `min` is further above `max`, in which case the
      returned value can land far outside the declared `[min, max]` — silent corruption
      instead of a `GenerationError`. Reachable whenever a resolved `Bound::Reference` (with
      an offset) ends up pushing `min` above `max` — a plausible authoring mistake neither
      task 054's schema validation nor this task's own resolution code currently catches.
      Same class of issue as task 054's blocking constraint-parser finding: malformed
      content must produce a graceful error, never a crash. Confirmed by direct reading, not
      agent speculation.
- [x] Architecture conformance — pass on the code itself. Process note (non-blocking, but
      now a **repeat**): this commit (owned by `codex`) again edited `ARCHITECTURE.md`
      directly (adding the `generation/` line), which `CLAUDE.md` reserves to Claude — same
      finding as task 055's review. The edit is again mechanically correct, so not reverted,
      but a second occurrence means this should stop happening going forward, not just be
      noted again.
- [ ] UI rules — N/A, no frontend/UI touched by this task.
- [x] Process — pass. Worklog is detailed and honest about several plan corrections made
      during implementation (module-naming collision, unused-import gating, a flaky-test
      fix). 235/235 tests pass, clippy/fmt/E2E all green.

Two additional non-blocking findings, confirmed by direct reading, moved to Follow-ups:

Verdict: **changes-requested** — fix the `sample_integer` bounds guard, then resubmit for
re-review.

### 2026-09-02 — Codex response to blocking finding

Implemented the requested bounds guard. `sample_integer` and `sample_float` now return
`None` for reversed ranges before consuming RNG state; integer span arithmetic uses
`i128`/`u128`, so the full inclusive `i64` range cannot overflow. `resolve_parameter`
checks resolved floating-point bounds before integer conversion and maps an invalid range to
`GenerationError::InvalidParameterBounds { family_id, parameter, min, max }`. This covers
the review's reachable reference-offset case and the saturation edge where very large
reversed floats would otherwise collapse to the same `i64`.

Regression coverage is named in the latest worklog entry above. Task returned to `review`
for an independent verdict; the original review checkbox remains unchanged until that
review occurs.

### 2026-09-02 — Independent re-verification (Claude)

Read `rng.rs`/`sampling.rs`/`error.rs` directly against commit `3a96234` rather than taking
the fix report at face value (the prior report for this task didn't match what was actually
on `master`). Confirmed correct:

- `sample_integer`/`sample_float` return `Option`, `None` on `min > max`, and a dedicated
  test (`reversed_ranges_are_rejected_without_consuming_rng_state`) proves rejection doesn't
  consume RNG state.
- Integer span arithmetic goes through `i128`/`u128`
  (`full_width_integer_range_does_not_overflow` exercises the entire `i64::MIN..=i64::MAX`
  span).
- The subtle case I hadn't even asked for is handled and tested:
  `reversed_integer_bounds_are_rejected_before_saturating_conversion` uses `min = 1e20`,
  `max = 9e19` — both beyond `i64::MAX`, which would saturate to the *same* `i64::MAX` under
  a naive cast and silently bypass an RNG-only guard. `resolve_parameter` checks `min > max`
  on the resolved `f64` values before any lossy cast, catching exactly this.
- Independently re-ran (not just read the reported numbers): `cargo test --locked` (240
  passed), `cargo clippy --all-targets --locked -- -D warnings`, `cargo fmt --all --check`,
  `git diff --check` — all clean.

No new `ARCHITECTURE.md` edit in this commit (the prior process note about `codex` editing
Claude-reserved docs doesn't recur here).

Verdict: **done**.

## Follow-ups

- `src-tauri/src/generation/template.rs:19-22` — the scientific-notation fix (preventing
  `"2e5"`'s `e` from being misread as a standalone identifier) also suppresses identifier
  detection for any parameter name immediately preceded by a digit with no separating
  operator (e.g. `"3b"` would leave `b` unsubstituted instead of replacing it). Low risk
  today — the real fixture always uses explicit `*`, and an author who wrote `"3b"` would
  already hit a `mathcore` parse failure regardless (a confusing error, not a silent wrong
  answer) — but worth tightening if a future family's expression relies on bare adjacency.
- `src-tauri/src/generation/template.rs:31` — `value.is_sign_negative()` is `true` for
  `-0.0`, so a resolved parameter that lands on negative zero renders as `"(-0)"` instead of
  `"0"` in a substituted expression. Cosmetic only (mathematically harmless), not correctness
  — noted for whoever next touches `format_number`.

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
