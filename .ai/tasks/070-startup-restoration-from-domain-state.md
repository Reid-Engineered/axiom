---
id: 070
title: Startup restoration from domain state
status: review
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
- 2026-09-10 — Claimed by codex on `agent/codex/070-startup-restoration-from-domain-state`; beginning TDD implementation of the locked domain-derived boot rule. The `useCommandPalette` mock workspace default and missing-workspace navigation guards move here from `077`, because removing only `App.tsx`'s fallback would silently preserve the fabricated id one hop downstream.
- 2026-09-10 — Added the five-branch pure comparator test and App boot/create/import tests first. The first executable targeted run was red: the comparator module was absent and three new App assertions still observed unconditional First Launch. Implemented domain-derived boot and then fixed a test-exposed import race by making `useRestoredContext`'s initially loaded workspace list the persistent sidebar's source rather than starting a second competing request.
- 2026-09-10 — Expanded the planned file list narrowly to `useCommandPalette`, `useConcepts`, and `useSessions`: hooks cannot be called conditionally, so the latter two accept an absent workspace and resolve locally to empty/null while the palette suppresses workspace actions, notes, and concepts. Added a mock invocation assertion proving `getConceptsByWorkspace`, `getActiveSessionByWorkspace`, and `getRecentNotes` are not called without an active workspace. `WorkspaceProvider` already accepted `initialWorkspaceId`, so no implementation change was needed there.
- 2026-09-10 — All applicable local gates passed and the task moved to `review`. CI/PR verification remains for the publishing step.
- 2026-09-10 — Parent review found the restored-Continue test proved only the destination
  route while its selected fixture had no bound attempt. Strengthened it to bind a real Shell
  method attempt before boot, capture its prompt through `describeAttempt`, select that
  second-listed workspace by activity, and assert Continue renders the identical prompt.
  `npm run test -- --run src/test/App.test.tsx` then passed 17/17; typecheck, lint, build,
  and the full 62-file / 175-test suite were re-run and remained green.

## What was built / tested / left out

- Added `selectStartupWorkspace`, a pure total-order selector implementing activity descending,
  the all-null creation-time fallback, and descending id tiebreak without changing sidebar order.
- Added `useRestoredContext`; production startup renders nothing until `getWorkspaces` resolves,
  then initializes First Launch for zero workspaces or Home plus the selected workspace otherwise.
- Removed `FALLBACK_WORKSPACE_ID` and the command palette's sample-workspace default. App navigation
  and workspace-scoped palette data/actions now require a real active workspace; global Marketplace
  remains available without one.
- Preserved create/import identity by continuing to set the returned workspace directly and using
  the restoration resource only as the refreshed sidebar list. Tests cover both paths against a
  competing more-recent workspace.
- Verified the selected workspace's Continue card reopens its exact bound attempt by comparing
  the rendered prompt with the pre-boot `describeAttempt` response.
- Test-only mock support can replace persisted workspaces, records invoked command names, and inserts
  the sample workspace on explicit import when the boot baseline is empty.
- TDD targeted command: `npm run test -- --run src/hooks/selectStartupWorkspace.test.ts src/hooks/useCommandPalette.test.tsx src/test/App.test.tsx` — 3 files, 25 tests passed after the recorded red run.
- `npm run test` — 62 files, 175 tests passed.
- `npm run typecheck` — passed with zero errors.
- `npm run lint` — passed with zero errors or warnings.
- `npm run build` — passed; 165 modules transformed.
- `rg -n "#[0-9a-fA-F]{3,6}|rgba\\("` over every changed `src/` file — no matches (rg exit 1).
- No Rust, route persistence, `localStorage`, new IPC command, attempt-lifecycle, task 074, or task
  068 changes were made. No design/CSS values were introduced.

## Review

## Follow-ups
