---
id: 016
title: NavigationContext + WorkspaceContext
status: done
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

Reviewer: claude-code
Date: 2026-08-27

- [x] Correctness — pass: `Route` is a clean discriminated union covering every page,
  `Overlay` is separate. `useNavigation`/`useWorkspace` throw outside their providers rather
  than silently returning `null`, which is the right failure mode for context misuse.
- [x] Architecture conformance — pass: `NavigationContext` holds only route + overlay,
  `WorkspaceContext` holds only the active id — matches `ARCHITECTURE.md` §5 rule 3 exactly,
  no domain data crept in.
- [x] UI rules — pass: no markup, no design values (logic-only task, correctly so).
- [x] Process — pass: independently re-ran `npm run typecheck`, `npm run lint`,
  `npm run build`, `npm test -- --run` (63/63, including this task's own `useNavigation`/
  `useWorkspace` suites) myself rather than trusting the claim.

Minor, non-blocking: `navigationContext.ts` imports `HomePageVariant` from `../pages/HomePage`
— a hook-layer file reaching into a page-layer type. Not a rule violation (nothing in
`ARCHITECTURE.md` forbids it), but it's a slightly backwards dependency direction worth a
second look — `HomePageVariant` describing a route shape arguably belongs in the navigation
layer or `types/`, with the page importing it, not the reverse.

Verdict: pass

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
