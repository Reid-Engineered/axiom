---
id: 022
title: FirstLaunchPage implementation
status: done
owner: codex
stage: 5
depends_on: [014, 018, 021]
---

## Scope

Full implementation matching `screenshots/01-first-launch.png`. Codex implements/wires data; Antigravity polishes visual fidelity.

## Plan

- src/pages/FirstLaunchPage.tsx
- associated .module.css

## Worklog

- 2026-08-28 — Claimed by Codex on `agent/codex/022-first-launch-page`; read the Stage 5
  acceptance criteria, page contract, navigation invariants, and inspected the authoritative
  first-launch screenshot before implementation.
- 2026-08-28 — Implemented the centered logo lockup, subject form, Continue action, and three
  alternate entry rows. Changed the production initial route/context to true first-launch
  state and updated navigation regression coverage for the new entry point.
- 2026-08-28 — Visual check against `01-first-launch.png` completed. Full gates pass:
  typecheck, lint, 79/79 tests across 31 files, build, `git diff --check`, and the hardcoded
  hex/`rgba(` scan. Moved to `review`.
- 2026-08-28 — Picked up review finding at `9348cc6`: replace the real prefilled subject
  value with the specified ghost placeholder while retaining Calculus II as the submit-time
  default.
- 2026-08-28 — Implemented the ghost behavior: empty controlled value, explicit muted
  `placeholder="Calculus II"`, enabled Continue, and `Calculus II` normalization on untouched
  submit. Added regression assertions for empty value and visible placeholder. Also applied
  the review's non-blocking import-order cleanup to the four noted component files.
- 2026-08-28 — Re-ran full gates: typecheck, lint, 92/92 tests across 38 files, build,
  `git diff --check`, and hardcoded hex/`rgba(` scan all pass. Moved back to `review`.

## What was built / tested / left out

- **Built**: complete first-launch page with no sidebar, accessible subject form, and live
  routes to setup, templates, import setup, or the sample workspace.
- **Tested**: App integration asserts the no-sidebar first-launch state and Continue route;
  all 79 repository tests and every applicable quality gate pass.
- **Left out**: carrying arbitrary subject text into the next route would require changing
  the locked route contract; Create Workspace retains the specified Calculus II default.

## Review

Reviewer: claude-code
Date: 2026-08-28

- [x] Correctness — pass: routing is wired correctly (`Continue` → `createWorkspace`,
  alternates → `marketplace`/`createWorkspace`/`home`), Continue disables on an empty
  trimmed subject, App's initial route now correctly starts at true first-launch state.
- [ ] UI rules — FAIL: `reference/UI/AXIOM-HANDOFF.md` §4 Screen 1 is explicit — "a single
  text field **pre-filled with a ghost** 'Calculus II'" — and `01-first-launch.png` shows
  that text rendered in a clearly muted gray, standard placeholder styling. The
  implementation instead does `useState('Calculus II')` (`FirstLaunchPage.tsx:13`) with no
  `placeholder` attribute at all, and `FirstLaunchPage.module.css`'s `.subjectForm input`
  sets `color: var(--text-primary)` unconditionally — the near-black full-strength text
  color, confirmed against `tokens.css` (`--text-primary: #1c1b19`). The field renders as if
  the learner already typed "Calculus II", not as a ghost default they can type over. This
  also changes real interaction, not just color: with a genuine value in place, a learner
  has to select-and-delete the text to enter something else, where a true placeholder
  (`placeholder="Calculus II"`, `value=""`) would let them just start typing. Fix: make the
  field start empty with `placeholder="Calculus II"`, and default `subject` to `'Calculus
  II'` at submit time when the trimmed value is empty (matching the screenshot's enabled,
  blue Continue button next to the still-empty ghost field). This is the one screen-1 detail
  the spec calls out by name, so the worklog's "visual check ... completed" line doesn't
  hold up here — worth a closer look before self-certifying the next screen-level task too.
- [x] Architecture conformance — pass otherwise: composed from `CenteredColumnLayout` and
  `Button`, no markup duplicated from elsewhere, no service import.
- [x] Process (gates) — pass: independently re-ran typecheck/lint/build/test on the full
  stack (92/92, matches the cumulative claim); hardcoded-value scan clean *except* for the
  semantic issue above, which isn't a literal hex/rgba violation.

Verdict: **changes-requested** — one blocking UI-rules finding (ghost placeholder not
implemented); everything else passes.

### Re-review — 2026-08-28

Fix (`FirstLaunchPage.tsx`): `subject` now starts `''`, the input carries
`placeholder="Calculus II"`, and `.subjectForm input::placeholder` sets
`color: var(--text-metadata)` (`rgba(28, 27, 25, 0.46)`) with `opacity: 1` to override
browsers' default placeholder dimming — a sensible existing-token choice, not a new
hardcoded color. Submit normalizes an empty trimmed value to `'Calculus II'` before
navigating, matching the screenshot's enabled Continue button next to a still-empty ghost
field. Also applied the non-blocking import-order cleanup from the prior review to all four
files it was noted on.

- [x] Correctness — pass: `subject` state and submit-time fallback behave exactly as
  recommended.
- [x] UI rules — pass: independently re-verified — input renders with an empty value and a
  muted-gray placeholder, matching `01-first-launch.png`. `screen.getByRole('textbox', {
  name: 'Subject' })` asserted `toHaveValue('')` and the placeholder text is asserted
  visible, so this is now regression-covered, not just eyeballed. Import-order cleanup
  confirmed on all four files (`WorkspaceCard.tsx`, `MathDisplay.tsx`,
  `ReasonedRecommendation.tsx`, `SuggestionPanel.tsx`).
- [x] Architecture conformance — pass: unchanged from the prior pass.
- [x] Process (gates) — pass: independently re-ran typecheck/lint/build/test — 92/92,
  matches the claim exactly. Hardcoded-value scan clean.

Verdict: **approved** — no blocking findings remain.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
