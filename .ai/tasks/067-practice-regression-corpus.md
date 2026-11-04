---
id: 067
title: Permanent Practice regression corpus
status: review
owner: claude
stage: 8
depends_on: [059, 063]
---

## Scope

Build the "permanent Practice regression corpus" that `ROADMAP.md:297` names as remaining
Stage 8 scope, and that task `064`'s beta gate leans on for criterion 1's "not a stub" bar.

The corpus pins Practice's *generated output*, not just its invariants. Task 059's property
tests already prove the generator is deterministic and that every instance satisfies its
declared bounds across 10,000 seeds — but they assert *properties*, so a change that alters
what `problem.shell_y_poly` actually produces for a given seed passes them silently, as long
as the new output is also in-bounds and also deterministic. That is exactly the regression
this corpus exists to catch: today the bundled family's content, prompt text, hint ladder, and
canonical solution could all shift without a single test failing.

Does **not** build: new problem families or generators (each is its own design + content
task), any change to `src-tauri/src/generation/`'s algorithm, any UI, the offline acceptance
test (task `068`), or `math.verify` tolerance changes.

## Prior art to follow

This repo already has a named convention for this, established twice, and the corpus should
adopt it rather than invent a third shape:

- **Module conformance fixtures** — `src-tauri/src/modules/tests/fixtures/*.toml`, one
  committed file per case, named for the case.
- **Knowledge Package conformance corpus** — 29 named mutation functions in
  `src-tauri/src/knowledge/tests/conformance.rs` and `conformance_problem_family.rs`, over a
  shared valid base package in `tests/mod.rs`'s `support` module.

The convention is stated at `docs/superpowers/plans/2026-08-30-knowledge-package-v1.md:3205`:
a *"permanent, case-named, never-deleted regression corpus"* — committed artifacts, named by
the case, **never deleted once the bug they caught is fixed**. That never-deleted discipline
is the point of the task; a corpus that gets pruned when it goes green is just a test suite.

Task 059 already anticipated this: its determinism test exists partly to give
"the regression corpus stable seed-to-instance mappings"
(`.ai/tasks/_archive/059-first-production-problem-family.md:180`).

## Plan

Files expected to be created or touched:

- `src-tauri/src/generation/tests/corpus/` (new) — the committed corpus itself: for a fixed,
  explicitly-listed set of seeds against `problem.shell_y_poly`, the full expected
  `ProblemInstance` — resolved parameters, prompt text, every hint in order, response type,
  and canonical solution. Serialized (JSON per case, named by seed) so a diff shows exactly
  what changed about a problem, not just that a hash moved.
- `src-tauri/src/generation/tests/corpus.rs` (new) — the runner: for each committed case,
  regenerate from the seed and assert the whole instance matches, field by field, with a
  failure message that names the drifting field.
- `src-tauri/src/practice/tests/` — one end-to-end corpus case that carries a known-correct
  answer through `practice.evaluate` and asserts it is accepted, plus a known-wrong one
  asserting rejection. This pins the generator *and* `math.verify` agreeing, which no current
  test does for committed content.
- A short `README.md` or module doc in the corpus directory stating the never-delete rule and
  how to add a case, so the discipline survives the next agent who sees a red corpus test.

**Seed selection matters and should be deliberate, not arbitrary.** The set should cover every
`(c, b)` parameter pair the family can produce (059 established there are 20), not just a
handful of round numbers — otherwise a regression confined to one corner of the parameter
space slips through. State the chosen seeds and why in the handoff doc.

**A regeneration escape hatch is required.** When a *deliberate* content change makes the
corpus stale, there must be a documented, single command to regenerate it (an ignored test, a
small bin target, or an env-gated mode), plus a rule that regenerating is a reviewable diff —
never a silent overwrite folded into an unrelated commit.

## Open question for whoever claims this

Committed-JSON-per-seed is the recommended shape above because it diffs legibly, but the
Knowledge Package precedent used Rust mutation *functions* rather than committed files, for a
stated reason (a package fixture is a whole directory; a function was the better fit). A
`ProblemInstance` is a single serializable value, so committed files should fit better here —
but if the implementer finds a reason the function-based shape is better, that is a legitimate
deviation to argue for in the task file rather than a rule to follow blindly.

## Worklog

- 2026-09-09 — Filed by claude. Named as "not yet filed" in task `064`'s follow-ups; this
  closes that gap. Scope drawn from `ROADMAP.md:297` and the existing corpus convention rather
  than newly invented.
- 2026-11-03 — Claimed by claude. Built the corpus (22 committed cases), runner, Practice
  end-to-end case and README. Mutation-checked: hand-editing one committed hint fails both the
  generator runner and the Practice case, naming the seed and the drifting `hints[n]`. Moved to
  `review`.

## What was built / tested / left out

**Built**

- `src-tauri/src/generation/tests/corpus/shell_y_poly/seed-<n>.json` — 22 committed cases for
  `problem.shell_y_poly` from the bundled `knowledge-package/`. Each has the full instance:
  resolved parameters, prompt, all 4 hints in order, response type and canonical solution.
- `src-tauri/src/generation/tests/corpus.rs` — runner and loader:
  - `every_committed_shell_y_poly_case_regenerates_identically` regenerates each seed and
    reports every drifting field by name (`prompt`, `hints[2]`, `canonical_solution`, ...)
    with committed vs. generated values.
  - `shell_y_poly_corpus_files_match_the_seed_list` fails if a file or a list entry
    disappears (enforces never-delete) or the list has a duplicate.
  - `shell_y_poly_corpus_covers_every_reachable_parameter_pair` keeps all 20 `(coeff, b)`
    pairs covered.
  - `regenerate_shell_y_poly_corpus` is the one regeneration command: `#[ignore]`d *and*
    gated on `AXIOM_REGENERATE_CORPUS=1`, so `cargo test -- --ignored` can't silently
    overwrite the corpus.
- `src-tauri/src/practice/tests/mod.rs` —
  `regression_corpus_cases_round_trip_through_practice_and_math_verify`. For every committed
  case, through the real registry with the real `math.verify` provider: the served prompt,
  response type and hint ladder match the corpus, `(<canonical>) + 1` is rejected, and the
  committed canonical solution is accepted (status `solved`, 2 submissions). Every case, not
  just one, because it's cheap (~0.05s for all 22).
- `src-tauri/src/generation/tests/corpus/README.md` — the never-delete rule, when and how to
  regenerate (as its own reviewable commit), how to add a case, why these seeds.
- `src-tauri/src/generation/mod.rs` — `mod tests` is now `pub(crate)` (still `#[cfg(test)]`)
  so the Practice test can share the corpus loader instead of duplicating it.

**Seeds and why:** the lowest seed reaching each of the 20 reachable `(coeff, b)` pairs
(`0, 1, 3, 4, 6, 7, 10, 11, 12, 15, 17, 20, 22, 26, 27, 37, 39, 46, 49, 100`), plus `42` (the
seed existing command/registry tests use) and `u64::MAX` (far edge of the RNG's seed space).

**Open question resolved:** committed JSON per seed, not Rust mutation functions. A
`ProblemInstance` is one serializable value, and a JSON diff shows exactly which field of a
problem changed, which is what a reviewer needs to judge a regeneration.

**Left out (per scope):** other problem families, any change to the generation algorithm,
UI, task `068`, `math.verify` tolerances. No new production code: only tests, test data and
the `pub(crate)` test-module visibility change.

**Gates run locally** (Windows, GNU toolchain — this machine has no MSVC build tools):
`cargo fmt --all --check` clean; `cargo clippy --all-targets --locked -- -D warnings` clean;
`cargo test --locked --lib` 322 passed, 1 ignored (the regeneration test). `cargo test`
without `--lib` couldn't link the `cdylib` under GNU ld ("export ordinal too large"), a local
toolchain limit, not a code issue; the crate has no integration-test targets, so `--lib` is
the whole Rust suite. No `src/` changes, so the npm gates don't apply. CI on the PR is the
source of truth.

**Not checkable by CI:** no structural change, so `ARCHITECTURE.md` is untouched. The corpus
lives inside the existing `generation/tests/` module.

## Review

Reviewer: Codex independent review, 2026-11-04
Implementation PR: https://github.com/Reid-Engineered/axiom/pull/30

- Correctness: pass after returned-seed comparison fix and focused regression test.
- Architecture: pass; production boundaries unchanged, implementation is test-only.
- UI rules: not applicable.
- Scope and worklog: pass; 22 cases cover all 20 reachable parameter pairs; actual
  bundled package and real Practice/math.verify providers are exercised.
- Regeneration safety: pass; both explicit environment variable and ignored-test opt-in
  are required. File/list agreement catches accidental one-sided removal; append-only
  history remains a review policy.
- Process: all seven required CI checks passed at head `73eb333` on 2026-11-04.
  Human sign-off for this task-record change remains required before merge.
  Targeted GNU corpus test cold build was interrupted before execution, not a test failure.

Verdict: pass

The independent code review and all required CI gates pass at `73eb333`. This review-only
commit changes no code or corpus content; its fresh required checks must also pass before
merge. Human sign-off under merge-strategy remains a separate merge requirement.

2026-11-04 — Independent Codex review identified that the field-level drift reporter
omitted the returned seed. Added seed comparison and `corpus_reports_returned_seed_drift`,
which mutates only the returned seed and checks the precise diagnostic. Integrated current
`origin/master` (082/083); CI on the resulting PR must validate the combined tree.
The finding is resolved. Final review passes; all seven checks passed at `73eb333`.

## Follow-ups
