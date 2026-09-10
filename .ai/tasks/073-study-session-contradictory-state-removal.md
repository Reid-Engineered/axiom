---
id: 073
title: Study Session contradictory-state removal
status: proposed
owner: codex
stage: 8
depends_on: [071]
---

## Scope

Implement §6.5 of `docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.

The session screen renders authored content beside a real generated attempt: a prefilled
working area reading `r = x, h = x² − 1`, a visualization pane naming `y = x² − 1 on [1, 3]`
while the real problem is something else entirely, a "Problem N of 1" counter and five-dash
indicator with no persisted total behind them, and a `0′ of 20′` readout frozen because
`elapsed_minutes` is written once as `0` and never updated.

Functional removal and a prop-shape change. **Not a visual task** — see Follow-ups.

Does **not** build: elapsed-time tracking (the readout goes, the mechanism is `076`), any Rust
change, the tutor pane (its canned persisted answer is a fabrication rather than a
contradiction — `077`), or `FullVisualizationPage` (`077`).

**Dependency.** Gated on `071`'s session contract being locked in review, not on `071`
merging. Touched files are disjoint from `071`'s (`src-tauri`) and from `070`'s, so
implementation may overlap both.

## Plan

- `src/pages/StudySessionPage.tsx` — empty the prefilled `working` default; replace the
  visualization pane's hardcoded region copy with an inert placeholder naming no specific
  region; stop passing a synthesised `problemCount`.
- `src/components/session/SessionToolbar.tsx` — `problemCount` becomes optional; the five-dash
  indicator renders only when a real persisted total exists; remove the elapsed readout and
  its `elapsedMinutes` / `targetMinutes` props.
- `src/pages/StudySessionPage.test.tsx`, `src/components/session/SessionComponents.test.tsx`.

**Tests — absence must be asserted.** "Existing tests pass untouched" is explicitly *not*
acceptance evidence for removed content. This task adds:

- the working area renders with an empty value on mount;
- no element on the session screen contains `y = x² − 1` or `[1, 3]`;
- with `problemCount` absent, no five-dash indicator renders and no "of N" appears in the
  counter or its `aria-label`;
- no elapsed-time claim renders — no `′` readout, no "of {targetMinutes}" text;
- the positive path is re-asserted in the same file: a bound attempt renders its prompt, a
  wrong answer produces "That does not match yet." plus a hint, a correct answer produces
  "Correct.", and Next problem resets the answer and hint list.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, `proposed` for codex.

## What was built / tested / left out

Not started.

## Review

## Follow-ups

- `078` — bounded Antigravity CSS pass, to be filed **only if** removing these elements leaves
  the toolbar or visualization pane visually broken. Layout and tokens only; the data and prop
  behaviour stays here and is not reassigned.
