---
id: 072
title: Sample seed carries curriculum, not activity
status: proposed
owner: codex
stage: 8
depends_on: []
---

## Scope

Implement §6.2 of `docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.

The sample import currently writes four sessions, 40 tutor exchanges, settled conclusions, a
`progress` of `0.58` and a `lastActivityAt` into SQLite — activity records for things no
learner did. The seeded session is what Home's "Resume session" resolves to, and it has no
bound attempt, which is the dead end the Windows walk hit.

Removes seeded *activity* from the sample. Curriculum stays: workspaces, concepts, goals,
modules, workspace templates, material, and the crosswalk that makes Shell method practiceable.

Does **not** build: the selection fix (`071`), the boot fix (`070`), or the wider seed hygiene
— mastery states, diagnostics, notes, heuristics and activity events are also fabricated
history but cause neither the dead end nor a boot misselection, and belong to `077`.

## Plan

- `src-tauri/src/commands/models.rs` — drop `sessions` from `SampleWorkspaceSeed`.
- `src-tauri/src/commands/seed.rs` — drop `insert_sessions` and its `tutor_exchanges` /
  `session_settled_conclusions` writes from `import_seed`.
- `src/services/sampleWorkspaceService.ts` — stop sending `sessions` in the seed payload.
- `src/services/mockData/workspaces.ts` — `progress: 0` and no `lastActivityAt` on all three
  sample workspaces. `070`'s boot ordering reads `last_activity_at`, so a seeded value would
  fabricate an active workspace.
- `src/test/mockBackend.ts` — **in this task, not a follow-up.** Line 83 seeds the double from
  `mockSessions`, which is exactly the divergence that let the frontend suite stay green
  through this defect (design §4.3). A double richer than the backend preserves the blind spot.
- `src-tauri/src/commands/tests.rs`, `src/pages/HomePage.test.tsx` — tests below.

`src/services/mockData/sessions.ts` is **not** deleted. `mockBackend` and individual tests may
still construct sessions; the file simply stops being shipped as seed.

**Tests.**

- After sample import, `getActiveSessionByWorkspace` returns `None` for all three sample
  workspaces.
- After sample import, zero rows exist in `sessions`, `tutor_exchanges` and
  `session_settled_conclusions`.
- After sample import, the sample workspace has `progress = 0` and `last_activity_at IS NULL`.
- The Shell method concept still resolves `shell.method_vertical_axis`; the existing crosswalk
  guard still passes.
- Frontend: Home renders no Continue card immediately after import.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, `proposed` for codex.

## What was built / tested / left out

Not started.

## Review

## Follow-ups
