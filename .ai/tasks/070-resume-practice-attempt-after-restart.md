---
id: 070
title: Resume the active Practice attempt after app restart
status: proposed
owner: unassigned
stage: 8
depends_on: [063, 065, 066]
---

## Scope

Make the rendered Study Session recover the same open Practice attempt after Axiom is
closed and relaunched. The prompt, attempt id, revealed hints, submissions, and open status
must come from the persisted binding rather than starting a fresh session or attempt.

Does not change Practice generation/evaluation semantics, add a new problem family, or
redesign the Study Session UI. If restoring the startup route and restoring the bound
attempt prove to be independent fixes, split the startup-route work rather than expanding
this task silently.

## Reproduction and acceptance

Observed in the Windows release build from `0503f3e` while walking task `064`:

1. Import the sample workspace, open Calculus II > Concepts > Shell method, and choose
   "Practice this."
2. Observe `f(x) = 4x - x^2` on `[0, 3]`.
3. Close Axiom completely and relaunch it.
4. The app displays the first-launch page even though Calculus II, Linear Algebra, and
   Mechanics remain in the sidebar.
5. Navigate back to Shell method and choose "Practice this" again.
6. A fresh `f(x) = 3x - x^2` on `[0, 2]` problem appears instead of the prior attempt.

Acceptance: after step 3, the learner can reopen the session and sees the exact same
prompt and open attempt state from step 2. Add coverage at the rendered/service composition
boundary that would have failed for the behavior above; the existing database persistence
test alone is not sufficient.

## Plan

Determine the smallest frontend navigation/session-selection change needed after reading
the task 063 binding design and the current `useSession` / Study Session route. Name exact
files here before implementation. Keep the existing backend degrade-to-NULL policy intact.

## Worklog

- 2026-09-09 — Filed from the task 064 Windows release-build walk. The storage test proves
  the binding survives reopening SQLite, but the rendered app did not recover it.

## What was built / tested / left out

## Review

## Follow-ups
