---
id: 073
title: Study Session contradictory-state removal
status: review
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
- 2026-09-10 — Claimed by codex. Beginning the TDD removal of contradictory Study Session
  content and unbacked toolbar claims after confirming task 071's session contract is
  approved and merged.
- 2026-09-10 — Added the required absence assertions first. The targeted suite failed on
  the prefilled working and on an always-rendered five-dash indicator labelled `Problem 3
  of undefined`, then passed after the minimal implementation (2 files, 9 tests).
- 2026-09-10 — Removed the authored working and region/readout copy, made persisted problem
  totals optional (including a defensive `null` from real IPC), removed elapsed-time props
  and output, and deleted only the CSS selectors made dead by those removals.
- 2026-09-10 — All applicable local gates passed: `npm run typecheck` (zero errors),
  `npm run lint` (zero errors), `npm run build` (163 modules transformed), and
  `npm run test` (61 files, 165 tests). Separate design-value greps for
  `#[0-9a-fA-F]{3,6}` and `rgba(` outside `tokens.css` both returned rg exit 1 (no
  matches).
- 2026-09-10 — Reviewed task 071's degraded solved-attempt observation. The fallback stays
  on the same explicit unbound message and controlled working area; this task makes it less
  misleading by leaving that working empty and otherwise does not change the degraded path.
- 2026-09-10 — Visual assessment: the visualization pane retains its full-size centred
  layout and is not empty, but omitting progress also removes the toolbar element that
  supplied `margin-left: auto`, pulling Pause beside the intent controls. That is the
  collapsed-spacing condition for the already-filed conditional task 078, which should be
  activated after 073; no styling was added here.

## What was built / tested / left out

- The learner's working starts empty; the visualization names no specific region and no
  longer presents a fabricated mathematical readout.
- The five-dash indicator and `of N` labels render only with a real problem total. The
  frozen elapsed-time claim and its component props are removed.
- Tests explicitly cover every required absence plus prompt → wrong answer and authored
  hint → correct answer → next problem with cleared answer, hint, and solved feedback.
- Left out by design: Rust and persistence changes, tutor content, Full Visualization,
  elapsed-time tracking, and CSS polish. Task 078 owns the observed toolbar alignment gap.
- Local verification is complete as recorded above. PR publication and required CI are
  intentionally left to the parent workflow.

## Review

## Follow-ups

- `078` — bounded Antigravity CSS pass, to be filed **only if** removing these elements leaves
  the toolbar or visualization pane visually broken. Layout and tokens only; the data and prop
  behaviour stays here and is not reassigned.
