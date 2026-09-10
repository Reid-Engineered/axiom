---
id: 070
title: Startup restoration from domain state
status: proposed
owner: codex
stage: 8
depends_on: [071, 072]
---

## Scope

Implement §6.4 of `docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.
Supersedes the Windows-walk observation filed verbally as "070 — resume practice attempt after
restart".

`src/App.tsx:187` mounts `NavigationProvider` with `initialRoute={{ type: 'firstLaunch' }}`
unconditionally, and `WorkspaceProvider` holds `activeWorkspaceId` in a plain `useState`. So
relaunching shows First Launch beside a sidebar full of real persisted workspaces, and nothing
reads `063`'s `current_attempt_id` binding.

Frontend only. **No route persistence** — no `app_state` table, no `localStorage`, no stored
UI position, and no new Tauri command. Restoration is derived from domain state that `071`
starts writing.

Does **not** build: any Rust change, booting directly into the session (rejected — a product
decision, not a defect fix), or any change to `useAttempt` and the attempt lifecycle.

## Plan

- `src/hooks/selectStartupWorkspace.ts` — new pure comparator, unit-testable in isolation.
- `src/hooks/useRestoredContext.ts` — new; blocks first paint on `getWorkspaces()` and applies
  the boot rule.
- `src/App.tsx` — use it; remove `FALLBACK_WORKSPACE_ID`.
- `src/hooks/WorkspaceProvider.tsx` — accept the derived initial workspace.
- `src/test/App.test.tsx`, new `src/hooks/selectStartupWorkspace.test.ts` — tests below.

**The locked boot rule.**

1. Block first paint on `getWorkspaces()`, so the app never renders First Launch and then
   corrects itself.
2. Zero workspaces → `firstLaunch`.
3. Otherwise → `home`, with `activeWorkspaceId` chosen by this total ordering:
   1. workspaces with non-null `lastActivityAt` sort before those with null;
   2. among those, greatest `lastActivityAt` first;
   3. if **every** workspace has null `lastActivityAt`: non-null `createdAt` before null, then
      greatest `createdAt` first;
   4. final tiebreak: `id` descending.

   Step 4 exists to make the order **total and reproducible**, not to approximate recency —
   workspace ids are random, so the tiebreak is arbitrary but deterministic. `getWorkspaces`
   keeps its existing `ORDER BY rowid`, so the sidebar's listing order is unchanged; the
   comparator runs in the frontend over that list, which is why `071` exposes `createdAt`.
4. **Immediately after a create or import action, use the workspace that action returned** —
   do not re-run the fallback. A freshly created workspace has null activity, and a freshly
   imported sample would otherwise compete on timestamps it does not have.
5. Home's Continue resolves the selected workspace's most recently active non-completed
   session via `getActiveSessionByWorkspace`, and reopens that session's bound attempt
   unchanged.

`workspace-calculus-ii` is not reintroduced as a fallback anywhere.

**Criterion 2 wording.** This restores the correct in-progress attempt and puts it one click
from launch; it does not land the learner inside the session. `064` criterion 2 is recorded in
those terms.

**Tests.**

- `selectStartupWorkspace`, per branch: mixed null/non-null activity; all-null activity falling
  through to `createdAt`; all-null activity *and* all-null `createdAt` falling through to the
  id tiebreak; a single workspace; an empty list.
- Boot against `mockBackend`: zero workspaces renders First Launch; one or more renders Home;
  the restored workspace is the one last worked in, not `workspaces[0]`; the post-create and
  post-import paths use the returned workspace rather than the comparator.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, `proposed` for codex.

## What was built / tested / left out

Not started.

## Review

## Follow-ups
