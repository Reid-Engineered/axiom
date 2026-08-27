---
id: 006
title: src/services/* function signatures
status: proposed
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Write `services/*Service.ts` function signatures against the types locked in 005 — real return types (`Promise<T>`, per `ARCHITECTURE.md` §5 rule 2), bodies `throw new Error('not implemented')`. Locks the contract Stage 4's hooks build against. No real logic or mock data reads yet.

## Plan

- src/services/workspaceService.ts
- src/services/goalService.ts
- src/services/conceptService.ts
- src/services/moduleService.ts
- src/services/sessionService.ts

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
