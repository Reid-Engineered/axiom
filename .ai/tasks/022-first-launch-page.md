---
id: 022
title: FirstLaunchPage implementation
status: changes-requested
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

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
