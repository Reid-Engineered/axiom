---
id: 021
title: hooks/use*.ts real implementations + tests
status: proposed
owner: codex
stage: 4
depends_on: [020]
---

## Scope

Real implementations of every `hooks/use*.ts`, each with a `renderHook` test against 019's real fixtures (not synthetic test doubles), per `AGENTS.md` §Testing. Enforces "only hooks call services" (`ARCHITECTURE.md` §5 rule 1) starting now.

## Plan

- src/hooks/useNavigation.ts
- src/hooks/useWorkspace.ts
- src/hooks/useCommandPalette.ts
- src/hooks/useResizablePanes.ts
- src/hooks/useKeyboardShortcut.ts
- + any domain hooks the pages in Stage 5/6 need
- corresponding *.test.ts(x) files

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
