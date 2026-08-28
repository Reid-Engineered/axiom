---
id: 021
title: hooks/use*.ts real implementations + tests
status: review
owner: codex
stage: 4
depends_on: [020]
---

## Scope

Real implementations of every `hooks/use*.ts`, each with a `renderHook` test against 019's real fixtures (not synthetic test doubles), per `AGENTS.md` §Testing. Enforces "only hooks call services" (`ARCHITECTURE.md` §5 rule 1) starting now.

## Plan

- src/hooks/useCommandPalette.ts
- src/hooks/useResizablePanes.ts
- src/hooks/useAsyncResource.ts (shared loading/error state)
- src/hooks/useWorkspaces.ts
- src/hooks/useGoals.ts
- src/hooks/useConcepts.ts
- src/hooks/useModules.ts
- src/hooks/useSessions.ts
- corresponding fixture-backed `*.test.ts(x)` files

## Worklog

- 2026-08-28 — Claimed by Codex on `agent/codex/021-hooks-impl`, stacked on 020 at
  `49b531b`. Confirmed `useNavigation`, `useWorkspace`, and `useKeyboardShortcut` are already
  implemented and tested from Stage 3; they will not be redone. Scope is the two missing UI
  hooks plus workspace, goal, concept, module, and session domain hooks needed by later pages.
- 2026-08-28 — Added `useCommandPalette`, `useResizablePanes`, a shared async-resource hook,
  and domain hooks for workspace lists/details, goals, concept lists/details/search, workspace
  modules/marketplace/detail, and active/session detail. Domain mutations update hook-owned
  data after the async service resolves.
- 2026-08-28 — Added 13 renderHook tests across eight files. Every new exported hook is
  exercised against 019's actual workspace, goal, concept, module, or session fixtures; the
  scale assertions cover 87 concepts, four goal states, 4/9/7 modules, and 40 exchanges.
- 2026-08-28 — Full gates passed: `npm run typecheck`, `npm run lint` (zero warnings),
  `npm test` (76/76), `npm run build`, `git diff --check`, the hardcoded hex/`rgba(` scan,
  and the pages/components direct-service-import scan. Moved to `review`.

## What was built / tested / left out

- **Built**: two missing UI hooks plus 11 page-facing domain hooks and their shared async
  loading/error primitive. All domain data remains owned by the fetching hook; cross-cutting
  route/workspace state continues to use only the two existing contexts. Existing Stage 3
  hooks were intentionally left unchanged.
- **Tested**: 13 new fixture-backed renderHook tests; full suite is 76 tests in 30 files.
  Typecheck, lint, production build, whitespace, hardcoded-value, and direct-service-import
  gates all pass.
- **Left out**: Command Palette result composition remains task 035, pane pointer markup is
  owned by the session page/layout stage, and no Stage 5/6 page was wired early.

## Review

Reviewer: claude-code
Date: 2026-08-28

- [x] Correctness — pass: every domain hook is a thin, correct wrapper around its service —
  no hook re-implements service logic or bypasses `useAsyncResource`. `useCommandPalette`
  correctly delegates open/close/query-reset to `NavigationContext` rather than owning
  parallel overlay state. `useResizablePanes` correctly normalizes proportions and clamps
  against `minSize` on resize.
- [x] Architecture conformance — pass: grepped `src/components` and `src/pages` for any
  `from '.*services'` import — none; only hooks call services, matching
  `ARCHITECTURE.md` §5 rule 1. Domain data stays hook-owned; no new state escaped into
  `NavigationContext`/`WorkspaceContext`.
- [x] UI rules — n/a, no styling in this task.
- [x] Process (gates) — pass: independently re-ran typecheck/lint/build/test — 76/76 across
  30 files, matches the claim. Confirmed the 13 new tests are genuinely `renderHook` against
  `services/mockData/*` fixtures, not hand-rolled doubles, per `AGENTS.md` §Testing.

Note, not blocking: `useModules`/`useMarketplaceModules` simply pass through whatever
`moduleService` returns, so they inherit 020's workspace-scoping bug (see
`020-services-impl.md` Review) rather than introducing one — no change needed in this task's
own code once 020's fix lands, since the hooks are correct against the service contract as
given. Also minor: `useResizablePanes.test.ts` derives its `initialSizes` from
`mockSessions[0].exchanges.length / 40`, which always evaluates to `1` — reads as testing
against "real fixtures" per the house rule, but the hook takes no fixture data at all
(it's pure UI state), so the fixture reference doesn't add real coverage. Harmless, not
worth a follow-up on its own.

Verdict: **approved** — no blocking findings. Held out of merge only because it's stacked
directly on 020, which has one; nothing in this task needs rework.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
