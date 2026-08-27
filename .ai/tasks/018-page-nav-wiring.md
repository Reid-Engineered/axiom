---
id: 018
title: Wire page stubs into sidebar nav + stub CommandPalette
status: proposed
owner: codex
stage: 3
depends_on: [014, 016, 017]
---

## Scope

Every `pages/*` stub from 014 reachable via sidebar navigation using `NavigationContext` from 016, rendering as an empty page with its assigned layout. `⌘K` opens a stub `CommandPalette` overlay (empty results acceptable — real results are 035).

## Plan

- route/overlay wiring in App.tsx or AppShell.tsx
- src/hooks/useKeyboardShortcut.ts (⌘K binding)
- src/components/overlays/CommandPalette.tsx (stub-level open/close only)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
