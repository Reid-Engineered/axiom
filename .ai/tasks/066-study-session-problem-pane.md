---
id: 066
title: Study session problem pane — types, useAttempt hook, and ProblemPane wiring
status: proposed
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

## What was built / tested / left out

## Review

## Follow-ups
