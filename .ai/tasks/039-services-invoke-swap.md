---
id: 039
title: src/services/* swap to invoke() calls
status: proposed
owner: codex
stage: 7
depends_on: [038]
---

## Scope

Swap `src/services/*` from `mockData/` reads to `invoke()` calls — signatures unchanged from 006/020. No page, hook, or component outside `services/*` should need to change; if one does, that's a finding against Stage 2's contract-locking worth a retro before continuing. Mock fixtures repurposed as first-launch/sample-workspace seed data, not deleted.

## Plan

- src/services/workspaceService.ts
- src/services/goalService.ts
- src/services/conceptService.ts
- src/services/moduleService.ts
- src/services/sessionService.ts

## Worklog

- 2026-08-29 (Codex, from 038): `get_active_session_by_workspace` returns Rust
  `Option<Session>`, whose IPC representation is `Session | null`. Preserve the locked
  service signature `Promise<Session | undefined>` by normalizing the invoke result with
  `?? undefined`; callers and hook tests should not change.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
