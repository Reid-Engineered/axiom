---
id: 067
title: Permanent Practice regression corpus
status: proposed
owner: unassigned
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

## What was built / tested / left out

## Review

## Follow-ups
