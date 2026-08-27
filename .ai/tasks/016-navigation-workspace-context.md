---
id: 016
title: NavigationContext + WorkspaceContext
status: proposed
owner: codex
stage: 3
depends_on: [005]
---

## Scope

Implement `NavigationContext` (discriminated-union `Route` + optional `Overlay`) and `WorkspaceContext` (active workspace id) per `ARCHITECTURE.md` §5 rule 5. No `react-router` — routing is this state machine. Logic-only task, no Antigravity polish pass needed.

## Plan

- src/hooks/useNavigation.ts (or context provider file)
- NavigationContext provider
- WorkspaceContext provider

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
