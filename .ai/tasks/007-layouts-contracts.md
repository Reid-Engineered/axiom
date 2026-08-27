---
id: 007
title: layouts/* stub contracts
status: proposed
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full TypeScript prop interface + TSDoc for `SessionShell`, `FullVisualizationShell`, `TwoPaneLayout`, `CenteredColumnLayout` — stub bodies only. Also reconfirms `AppShell`'s contract (already has an empty implementation from Stage 0) against the rest of the layout inventory in `ARCHITECTURE.md` §2, without changing Stage 0's behavior.

## Plan

- src/layouts/SessionShell.tsx (stub)
- src/layouts/FullVisualizationShell.tsx (stub)
- src/layouts/TwoPaneLayout.tsx (stub)
- src/layouts/CenteredColumnLayout.tsx (stub)
- src/layouts/AppShell.tsx (contract review only, no behavior change)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
