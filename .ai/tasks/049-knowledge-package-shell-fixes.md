---
id: 049
title: Fix cylindrical-shells reference Knowledge Package findings
status: proposed
owner: codex
stage: 8
depends_on: []
---

## Scope

`knowledge-package/` (cylindrical shells reference content) was generated ahead of any
locked Knowledge Package schema and ahead of a task file — this task is the first time it
enters the `.ai/tasks/` process. It was reviewed by Claude against the raw files (not
against a schema, since none exists yet — see `ROADMAP.md`'s "Remaining Stage 8 scope,"
which lists "Knowledge Package v1 schema" as not yet designed). The review's full text is
not reproduced here in full; this task carries forward only the findings that are fixable
today, within the package's current ad hoc structure.

**In scope**: the concrete BLOCKING and MAJOR findings below — parameter/constraint bugs,
one internally-contradictory problem family, a dead-parameter cleanup, a template
inconsistency, a missing prerequisite link, and package manifest completeness.

**Explicitly out of scope** — do not attempt to fix these, only note them in the Worklog:
- Designing a Knowledge Package v1 schema. Per `ROADMAP.md`, that's its own future
  brainstorm → spec → plan cycle, same rigor as `045`. Inventing one unilaterally inside a
  bugfix task would repeat the exact mistake this task exists to correct.
- The ID-grammar ambiguity between `package.json`'s `id` (`org.axiom.reference.calculus.
  cylindrical-shells`, cosmetically `ModuleId`-shaped but containing a hyphen `ModuleId`'s
  actual grammar in `src-tauri/src/modules/identifier.rs` would reject) versus concept IDs
  (pure hyphen-slugs) versus objective IDs (mixed dot+hyphen). This needs a schema decision,
  not a find-and-replace.
- Anything requiring the actual OpenStax *Calculus Volume 2* source text — it is not present
  in this repo. Where a finding below needs source verification you can't do, downgrade the
  family's `status` to `"needs-review"` and say why in the Worklog rather than guessing.

## Plan

Files to touch, each tied to a specific finding:

1. **`problem-families/pf-shell-y-poly.json`** — `coeff`∈[2,6] and `b`∈[1,6] are sampled
   independently; the real requirement (`b ≤ coeff`, needed so `f(x)=coeff·x−x²≥0` on
   `[0,b]`) exists only as prose in `constraints`. `coeff=2, b=6` is in-range and generates a
   region where `f(x)<0` past `x=2` — the prompt's described region and the canonical
   solution stop matching reality. Fix: make `b`'s effective range actually depend on
   `coeff` (e.g. cap `b`'s max at `coeff` in whatever way this format allows — a parameter
   reference if you add one, or restructure so `b` is derived rather than independently
   ranged) so no valid parameter combination can violate the constraint. Re-verify a few
   boundary instances by hand before leaving `status: "verified"`.

2. **`problem-families/pf-shell-y-reciprocal.json`** — same class of bug: `a`∈[1,4], `b`∈
   [2,10], required `b > a` is prose-only. `a=4, b∈{2,3}` is in-range and produces a
   reversed/invalid interval. Same fix approach as #1.

3. **`problem-families/pf-shell-shifted-vertical-axis.json`** — `a`∈[1,2], `b`∈[2,4],
   required `b > a` is prose-only; `a=2, b=2` is reachable (1 of 6 combinations) and
   produces a zero-width, degenerate interval. Same fix approach as #1.

4. **`problem-families/pf-method-select-integral-count.json`** — its own `constraints` text
   ("shell method integrating with respect to y" for y-axis revolution) directly
   contradicts `concepts/shell-method-vertical-axis.json`'s own rule (shells for y-axis
   revolution integrate with respect to **x** — `V = ∫ 2πx f(x) dx`, per Rule 2.6). The
   canonical solution's formula (`π f(x)² dx`, split at the vertex `x = intercept/2`) is the
   x-axis disk formula, not a technique that computes volume about the y-axis. Independently
   re-deriving the stated region (triangle bounded by `y=x, y=2−x, y=0`, revolved about the
   y-axis) gives washer(y)=1 integral, shell(x)=2 integrals — opposite of what this family
   and `provenance.json`'s note on `src-openstax-calc2-ex2-17` both claim. Resolve the
   self-contradiction against `shell-method-vertical-axis` at minimum; if you can't get
   independent confirmation of which count is actually correct for the source example
   without the book itself, set `status: "needs-review"` and document the contradiction
   plainly rather than picking one arbitrarily.

5. **`problem-families/pf-shell-y-between-curves.json`** — declares `a` (fixed 0) and `b`
   (ranged, constrained `b == m`) that never appear in `promptTemplate`,
   `canonicalSolution`, or any hint — only `m` is used anywhere. Remove the dead `a`/`b`
   parameters (or, if there's a reason to keep the interval generalized beyond `[0, m]`,
   actually wire them into the prompt/solution/hints — don't leave declared-but-unused
   parameters with an unenforced identity constraint).

6. **`problem-families/pf-shell-setup-integrand.json`** — `canonicalSolution.structure` uses
   `x^{n+1}` (arithmetic inside a single brace pair); hint level 3 in the *same file* uses
   `x^{{n}+1}` (nested-brace form) for the identical substitution. No template grammar is
   documented anywhere in this package or repo, and a plain key→value renderer can't resolve
   arithmetic-in-braces at all. Pick one consistent form, use it throughout this file, and
   add a one-line note (in this file or `synthesis-report.md`) stating what substitution
   convention the package assumes, so the next family author doesn't reinvent it.

7. **`concepts/method-selection-volume-of-revolution.json`** — `prerequisiteIds: []` despite
   genuinely depending on both sibling concepts. Add `shell-method-vertical-axis` and
   `shell-method-horizontal-axis` to `prerequisiteIds` (they're currently only in the weaker,
   symmetric `relatedConceptIds`). Leave a Worklog note that this concept also assumes
   disk/washer-method knowledge this package doesn't model — don't invent a stub concept for
   it, just flag it as a real external dependency for whoever eventually designs
   cross-package prerequisites.

8. **`package.json`** — currently lists only `conceptIds`; there's no way to tell from the
   manifest alone what objectives or problem families the package contains without
   directory-scanning. Add `objectiveIds` and `problemFamilyIds` arrays (or, if you'd rather
   not commit to that shape pre-schema, add an explicit one-line note in `package.json` or
   `synthesis-report.md` stating that consumers are expected to directory-scan
   `objectives/` and `problem-families/` — pick whichever is less likely to need reversing
   once the real schema lands, and say which you picked and why in the Worklog).

9. **`synthesis-report.md`** — update any section whose claims no longer match the files
   after the fixes above (e.g. `pf-method-select-integral-count`'s rationale if its status
   changes, the parameterization description for any family whose ranges change).

## Worklog

- 2026-08-30 (claude-code): Task created from a full-file review (not reproduced here in
  full — see the review conversation if you need the complete rationale for a finding).
  Scope above carries forward only what's fixable without a locked schema; schema design and
  the ID-grammar question are explicitly deferred, not silently dropped — see "Explicitly
  out of scope."

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope — expected
candidates already known: Knowledge Package v1 schema design, the ID-grammar
reconciliation, and any MINOR findings not listed above (orphaned `provenance.json` entry
`src-openstax-calc2-book`, `pf-shell-x-axis-dy`'s misleading "perfect square" parameter
description, undocumented difficulty rubric).
