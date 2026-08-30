---
id: 049
title: Fix cylindrical-shells reference Knowledge Package findings
status: done
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

- 2026-08-30 (codex): Claimed the task and confirmed `knowledge-package/` has no nested
  agent instructions. I will use one mechanically enforced dependent-parameter convention
  across the three interval families, independently derive the integral-count family before
  editing it, and validate each changed JSON document plus representative boundary math.
  Knowledge Package schema design and the package/concept/objective ID-grammar ambiguity
  remain explicitly out of scope and will not be changed.
- 2026-08-30 (codex): Chose one dependent-bound representation for all three affected
  families: a `min` or `max` object shaped as `{ "parameter": "<id>", "offset": <integer> }`,
  evaluated after the referenced parameter. Thus polynomial `b.max` references `coeff`,
  while reciprocal and shifted-axis `b.min` reference `a` with offset 1. This removes the
  independently sampled invalid combinations rather than merely restating their constraints.
- 2026-08-30 (codex): Independently derived the integral-count geometry for general
  intercept `C`: horizontal slices have `0 <= y <= C/2`, inner radius `y`, and outer radius
  `C-y`, so disks/washers about the y-axis require the single integral
  `integral_0^(C/2) pi((C-y)^2-y^2) dy`. Vertical shells have heights `x` on `[0,C/2]` and
  `C-x` on `[C/2,C]`, so shells require two x-integrals. I can verify the stated family's
  mathematics without the book, but cannot verify why the absent source and its provenance
  note claim the reverse; the corrected family will therefore be `needs-review` rather than
  presented as source-verified.
- 2026-08-30 (codex): Applied the remaining scoped content fixes: removed unused `a`/`b`
  parameters from the between-curves family; standardized exponent templates on single-pass
  `{key}` replacement inside separate LaTeX grouping braces; added both shell concepts as
  method-selection prerequisites; and added explicit objective/problem-family inventories
  to `package.json`. I chose manifest arrays over a directory-scan convention because they
  make the current package self-describing while remaining simple content metadata, not a
  claim that this ad hoc shape is the future schema. The method-selection concept still has
  a real external disk/washer prerequisite that this package cannot model; it is recorded in
  `synthesis-report.md` without inventing a stub or cross-package schema.
- 2026-08-30 (codex): Boundary checks before retaining the three families' `verified`
  status: polynomial `(coeff,b)=(2,2)` has `f(x)=x(2-x)>=0` and `V=8pi/3`, while `(6,1)`
  gives `V=7pi/2`; reciprocal `(k,a,b)=(1,4,5)` gives `V=2pi` and `(8,1,10)` gives
  `V=144pi`; shifted-axis `(k,a,b)=(1,1,2)` reproduces `23pi/3`, while the formerly
  degenerate `a=2,b=2` is unreachable and its new lower boundary `(1,2,3)` gives
  `53pi/3`. A read-only Node check parsed all 18 JSON files, enumerated all 20 polynomial,
  240 reciprocal, and 15 shifted-axis parameter combinations, and matched every canonical
  formula against its defining antiderivative. It also checked every between-curves `m`,
  all intercepts 2 through 6 for equal washer/shell volumes, all exponent renders for
  `n=2..5`, prerequisites, and exact manifest-to-directory ID coverage.
- 2026-08-30 (codex): Added the required `ARCHITECTURE.md` folder-inventory line because
  this task brings the previously untracked top-level `knowledge-package/` directory into
  version control. The description explicitly calls the content ad hoc until Knowledge
  Package v1, preserving the task's schema-design deferral.
- 2026-08-30 (codex): Final checks passed: all package JSON parses, the scoped math and
  inventory assertions pass, and `git diff --cached --check` reports no whitespace errors.
  No repository test harness targets this ad hoc content yet, so the exact read-only checks
  are recorded below rather than represented as an automated-suite pass. Moving 049 to
  review.
- 2026-08-30 (claude-code): Task created from a full-file review (not reproduced here in
  full — see the review conversation if you need the complete rationale for a finding).
  Scope above carries forward only what's fixable without a locked schema; schema design and
  the ID-grammar question are explicitly deferred, not silently dropped — see "Explicitly
  out of scope."

## What was built / tested / left out

**Built**

- Brought the complete cylindrical-shells reference package into version control and added
  its required top-level folder entry to `ARCHITECTURE.md`.
- Replaced prose-only interval conditions in the polynomial, reciprocal, and shifted-axis
  families with a consistent machine-readable dependent-bound representation.
- Corrected the integral-count family's internal geometry to one y-axis washer integral
  versus two x-axis shell integrals, and downgraded it to `needs-review` because the absent
  source text cannot be reconciled with the opposite claim in provenance.
- Removed dead between-curves parameters, standardized exponent templates and documented
  their substitution convention, added shell-method prerequisites, and made the package
  manifest enumerate all objectives and problem families.
- Updated `synthesis-report.md` for the dependent ranges, integral-count correction and
  source uncertainty, template convention, manifest inventory, and external disk/washer
  prerequisite gap.

**Tested**

- Read-only Node validation parsed all 18 JSON files successfully.
- Enumerated every allowed dependent-range instance: 20 polynomial `(coeff,b)` pairs, 240
  reciprocal `(k,a,b)` triples, and 15 shifted-axis `(k,a,b)` triples. All enforced the
  intended inequalities, and every stated volume expression matched its defining
  antiderivative.
- Hand-checked boundary instances for each retained `verified` family, including the
  original shifted-axis example (`23pi/3`) and the new nondegenerate `a=2` lower boundary.
- Verified the between-curves formula for every `m=1..4`; compared the one-integral washer
  volume with the two-integral shell volume for every `intercept=2..6`; rendered exponent
  templates for every `n=2..5`; and checked prerequisite and manifest-directory ID equality.
- `git diff --cached --check` — passed.
- No npm, Cargo, or package-specific automated test applies: this directory contains JSON
  content and Markdown only, and no Knowledge Package schema, loader, or test harness exists.

**Left out**

- Knowledge Package v1 schema design and the package/concept/objective ID-grammar question
  remain unchanged and explicitly deferred.
- The `src-openstax-calc2-ex2-17` provenance note remains as originally generated pending
  source-text review; `pf-method-select-integral-count` is `needs-review` so the conflict is
  visible rather than guessed away.
- No disk/washer stub concept or cross-package prerequisite shape was invented. The external
  dependency is documented for the future schema work.
- Other known minor findings remain outside this task exactly as listed under Follow-ups.

## Review

Reviewer: claude-code
Date: 2026-08-30

- [x] Correctness — pass. I independently re-derived every corrected formula from scratch
      rather than trusting the worklog's claims, and all of it checks out:
      - `pf-shell-y-poly`: `b.max = {parameter: coeff, offset: 0}` makes `b>coeff`
        unreachable. Hand-checked `(coeff,b)=(2,2)→8π/3` and `(6,1)→7π/2` against
        `V=2π(coeff·b³/3−b⁴/4)` — both match.
      - `pf-shell-y-reciprocal` / `pf-shell-shifted-vertical-axis`: `b.min =
        {parameter: a, offset: 1}` makes `b≤a` unreachable, closing the reversed-interval
        and zero-width-interval cases from the original review. Hand-checked
        `(k,a,b)=(1,4,5)→2π`, `(8,1,10)→144π`, and shifted-axis `(1,1,2)→23π/3` (matches the
        original textbook example) and the new minimum `(1,2,3)→53π/3` — all match.
      - `pf-method-select-integral-count`: re-derived the triangle (vertices `(0,0),(1,1)
        scaled to (C/2,C/2),(C,0)`) from first principles — washer method integrated w.r.t.
        y is a single contiguous horizontal segment `[y, C−y]` for `0≤y≤C/2`, one integral;
        shell method integrated w.r.t. x has a piecewise height (`x` then `C−x`, splitting
        at the vertex), two integrals. This is the *opposite* of the original family and
        matches what I found in the original review. Spot-checked total volume equality
        between methods at `C=2` (2π) and `C=6` (54π) — both reconcile. Correctly downgraded
        to `needs-review` rather than asserted `verified`, since the actual OpenStax source
        text isn't available in this repo to settle why `provenance.json`'s note claims the
        reverse — the right call given the task's explicit instruction on this point.
      - `pf-shell-y-between-curves`: dead `a`/`b` parameters removed; only `m` remains,
        matching every other reference to this family.
      - `pf-shell-setup-integrand`: unified on `x^{{n}+1}` throughout (was split between
        `x^{n+1}` and `x^{{n}+1}` in the same file) and — importantly — the resulting
        single-pass, no-arithmetic substitution behavior (`n=2` renders `x^{2+1}`, not
        `x^{3}`) is explicitly documented in both the family file and
        `synthesis-report.md` §"Structural inferences" #5, rather than left as an
        implicit assumption. That was the actual ask — not that the exponent render
        prettily, but that the convention be stated so it isn't reinvented per-family.
- [x] Architecture conformance — pass, with one correctly-handled edge: bringing
      `knowledge-package/` into git for the first time required a folder-inventory line in
      `ARCHITECTURE.md` (added, correctly marked "ad hoc until Knowledge Package v1" —
      doesn't overclaim a schema that doesn't exist).
- [x] UI rules — N/A, no UI surface touched.
- [x] Process — pass. Every fix traces to a specific Plan item; nothing outside the 9 items
      was touched (the `pf-shell-x-axis-dy` "perfect square" description and the orphaned
      `provenance.json` entry — both MINOR, both explicitly out of scope — are confirmed
      byte-for-byte unchanged). The two "Explicitly out of scope" items (schema design,
      ID-grammar) are genuinely untouched, not quietly resolved. No test harness exists for
      this content yet, so "tested" is a described read-only validation pass (18 JSON files
      parsed, all reachable parameter combinations enumerated for the three fixed interval
      families, formulas checked against their antiderivatives) rather than a checked-in
      automated suite — correctly stated as such rather than overclaimed as a real gate.
      `git diff --check` clean, worktree clean, no stray tooling left behind.

One nit, not blocking: `method-selection-volume-of-revolution.json` now has the exact same
two IDs in both `prerequisiteIds` and `relatedConceptIds`. Not wrong, just redundant —
`relatedConceptIds` could drop what's already implied by `prerequisiteIds`. Leaving as a
Follow-up rather than failing Process over it.

Verdict: done

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope — expected
candidates already known: Knowledge Package v1 schema design, the ID-grammar
reconciliation, and any MINOR findings not listed above (orphaned `provenance.json` entry
`src-openstax-calc2-book`, `pf-shell-x-axis-dy`'s misleading "perfect square" parameter
description, undocumented difficulty rubric).

- 2026-08-30 (claude-code, review): `method-selection-volume-of-revolution.json` has the
  same two concept IDs in both `prerequisiteIds` and `relatedConceptIds` — harmless
  redundancy, not worth a changes-requested round, but `relatedConceptIds` could drop what's
  already implied by `prerequisiteIds` whenever this file is next touched.
