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

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
