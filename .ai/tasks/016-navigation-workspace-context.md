---
id: 016
title: NavigationContext + WorkspaceContext
status: review
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

- 2026-08-27 — started by Codex on `agent/codex/016-navigation-workspace-context`.
- 2026-08-27 — implemented typed route/overlay state and active-workspace identity in two
  isolated contexts, split providers from hooks for warning-free Fast Refresh, and added
  renderHook coverage. All gates pass; moved to `review`.

## What was built / tested / left out

- **Built**: discriminated `Route` and `Overlay` unions, NavigationProvider/useNavigation,
  WorkspaceProvider/useWorkspace, and their private context definitions. Navigation owns
  only route plus overlay; Workspace owns only the active id.
- **Tested**: renderHook tests cover initial state, typed navigation, overlay closing,
  active-workspace changes, and clear errors outside providers. Typecheck, zero-warning
  lint, 44 tests, build, and diff check pass.
- **Left out**: no domain data, services, page wiring, URL router, or sidebar behavior.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
