---
id: 014
title: pages/*Page.tsx stub contracts (all 14 screens)
status: proposed
owner: claude-code
stage: 2
depends_on: [005, 006, 007, 008, 009, 010, 011, 012, 013]
---

## Scope

Full prop interface + TSDoc for every page in `ARCHITECTURE.md` §3's table — stub bodies rendering their assigned layout with placeholder content, no data fetching. Depends on every other Stage 2 contract task since each page composes components whose shapes must already be locked.

## Plan

- src/pages/FirstLaunchPage.tsx
- src/pages/CreateWorkspacePage.tsx
- src/pages/HomePage.tsx
- src/pages/WorkspaceOverviewPage.tsx
- src/pages/StudySessionPage.tsx
- src/pages/FullVisualizationPage.tsx
- src/pages/ConceptViewPage.tsx
- src/pages/ConceptsListPage.tsx
- src/pages/MaterialPage.tsx
- src/pages/WorkspaceToolsPage.tsx
- src/pages/MarketplacePage.tsx
- src/pages/ModuleDetailPage.tsx
- src/pages/GoalEditingSheet.tsx
- (CommandPalette overlay contract covered in 011)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
