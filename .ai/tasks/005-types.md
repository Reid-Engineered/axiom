---
id: 005
title: src/types/* domain model
status: proposed
owner: claude-code
stage: 2
depends_on: []
---

## Scope

Populate `src/types/` per `ARCHITECTURE.md` §4, matching `AXIOM-HANDOFF.md` §1 exactly: all five mastery states, all goal states, offline statuses, trust levels. This is the first thing locked in Stage 2 — everything else (service signatures, component props, mock fixtures) is written against it. No dependency on Stage 1 primitives.

## Plan

- src/types/common.ts
- src/types/workspace.ts
- src/types/goal.ts
- src/types/concept.ts
- src/types/module.ts
- src/types/session.ts
- src/types/index.ts (barrel re-export)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
