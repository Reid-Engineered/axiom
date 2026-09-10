---
id: 076
title: Real elapsed-time tracking for sessions
status: proposed
owner: unassigned
stage: 9
depends_on: [073]
---

## Scope

Deferred follow-up from §7.1 of
`docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.

`sessions.elapsed_minutes` is written once as `0` on insert and never updated, so the session
toolbar read `0′ of 20′` permanently. `073` removes the readout; nothing replaces it.

If session timing is still wanted, it needs `elapsed_minutes` genuinely maintained across
pause, resume and `nextProblem`, plus a decision about whether an elapsed-against-target
figure reads as context or as a score under `AGENTS.md:22` ("no percentages … never a number
presented as a score").

**Not required for `064` criteria 1, 2 or 6.** Filed so the removal in `073` is a recorded
deferral rather than a silent loss.

## Plan

Not planned. Needs its own brainstorming pass, starting with the `AGENTS.md:22` question —
the answer determines whether there is a task here at all.

## Worklog

- 2026-09-09 — Filed by claude alongside `073`, which removes the readout this would restore.

## What was built / tested / left out

Not started.

## Review

## Follow-ups
