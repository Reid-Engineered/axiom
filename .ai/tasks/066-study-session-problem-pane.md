---
id: 066
title: Study session problem pane — types, useAttempt hook, and ProblemPane wiring
status: review
owner: antigravity
stage: 8
depends_on: [063]
---

## Scope

Tasks 3, 4 and 5 of `docs/superpowers/plans/2026-09-09-study-session-ui-integration.md`: the
frontend half of the Study Session integration. Adds the `AttemptDescription` type and
`Session.currentAttemptId`, the `describeAttempt` / `nextProblem` service functions, mock-IPC
support for both, the `useAttempt` hook, and the `ProblemPane` rewire that replaces the
screen's hardcoded problem with the session's real bound attempt across four states: bound,
unbound, wrong answer, solved.

Does **not** build: any Rust — the two commands are task `065`, owned by codex and running in
parallel. This task's tests run against `src/test/mockBackend.ts`, so it is not blocked on
`065` landing. Does not touch the visualization pane, the tutor pane, working-area
persistence, or `ROADMAP.md` (task `064`'s Task 6).

## Plan

Follow the plan document's Task 3, Task 4 and Task 5 exactly; it carries the full test code
and implementation for each. Files:

- `src/types/practice.ts`, `src/types/session.ts`, `src/types/index.ts`
- `src/services/practiceService.ts`, `src/services/sessionService.ts`
- `src/test/mockBackend.ts` — bind an attempt on `startSession` for a mapped concept; handle
  `describeAttempt` and `nextProblem`
- `src/hooks/useAttempt.ts` + `src/hooks/useAttempt.test.tsx`
- `src/pages/StudySessionPage.tsx`, `.module.css`, `.test.tsx`

Design source: `docs/superpowers/specs/2026-09-09-study-session-ui-integration-design.md`
§§4–7. Two known risks the plan calls out: Task 5's CSS token names are written from
neighbouring conventions and need checking against `src/styles/tokens.css` (substitute the
nearest real token, never a literal), and Task 3's `startSession` change may require updating
existing session tests that assert exact session objects.

## Worklog

- 2026-09-09 — Filed by claude from the approved plan, split from task `065` so the Rust and
  frontend halves have unambiguous single owners per `.ai/lifecycle.md`. Assigned to
  antigravity, which owns `StudySessionPage` from task 026 and every other page-layer task in
  the archive.
- 2026-09-09 — Task 3 implemented: added `AttemptDescription` to `src/types/practice.ts`,
  `currentAttemptId?: string` to `Session` in `src/types/session.ts` (re-exported in `src/types/index.ts`),
  added `describeAttempt` to `practiceService.ts` and `nextProblem` to `sessionService.ts`, updated
  `src/test/mockBackend.ts` with mapped concept attempt creation on `startSession` and handlers
  for `describeAttempt` and `nextProblem`. Added unit test in `practiceService.test.ts`.
- 2026-09-09 — Task 4 implemented: added `src/hooks/useAttempt.ts` with full attempt lifecycle
  (load/describe, check with typed response, hint revelation on wrong answer, advance with next)
  and 7 unit tests in `src/hooks/useAttempt.test.tsx`.
- 2026-09-09 — Task 5 implemented: rewired `StudySessionPage.tsx` and `ProblemPane` to `useAttempt(session, setData)`,
  removed hardcoded `shellExpression`, rendered four states (bound open, unbound, wrong answer with hint, solved with next problem),
  added token-compliant styles in `StudySessionPage.module.css`, and verified with 4 new tests in `StudySessionPage.test.tsx`.
- 2026-09-09 — Quality gates verified: `npm test` (60 test files, 161 tests passed), `npm run typecheck` (0 errors),
  `npm run lint` (0 errors), `npm run build` (succeeded), and token regex check (0 hardcoded values). Marked for review.
- 2026-09-09 — Review findings addressed: rebased cleanly on `origin/master`; wrapped check, hint, and next actions in `StudySessionPage.tsx` with `setMutationError` to surface failures to the user; switched `hintList` keys to array indices; added test asserting error alert on check failure; documented `vite.config.ts` test exclusion for `.claude/**` to isolate external Claude worktrees.

## What was built / tested / left out

- **Built:**
  - `AttemptDescription` type interface and `Session.currentAttemptId?: string`
  - `describeAttempt` and `nextProblem` service functions
  - Mock IPC support for `describeAttempt` and `nextProblem`, plus attempt binding on `startSession` for mapped concepts
  - `useAttempt` hook managing attempt state, answer, hints, evaluation, checking, and problem advancement
  - `ProblemPane` rewiring supporting bound attempt prompt, numeric/symbolic answer input, feedback messages, hint list, solved state, and unbound empty state
  - Error state handling in `StudySessionPage` wrapping check, hint, and next actions with `mutationError` alerts
  - CSS module styling using tokens from `tokens.css`
  - Excluded `.claude/**` in `vite.config.ts` `test.exclude` to prevent test runner from picking up files in external Claude worktrees

- **Tested:**
  - `src/services/practiceService.test.ts` (describing generated attempts)
  - `src/hooks/useAttempt.test.tsx` (describing on mount, unbound case, hint reveals on wrong answer, exhausted hints boundary, solved status, next problem advancement, error handling)
  - `src/pages/StudySessionPage.test.tsx` (rendering bound attempt prompt, unbound state message, hint revelation on wrong answer, solved confirmation and next problem action, and error alert on check failure)
  - Full frontend suite: 60 test files / 162 tests passing

- **Left out:**
  - Rust Tauri commands (handled in parallel by task 065)
  - Visualization and tutor pane backend integration (mock by design for Stage 8)
  - Working area scratch persistence (deferred per spec §9)

## Review

## Follow-ups
