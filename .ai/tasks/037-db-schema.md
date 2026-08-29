---
id: 037
title: src-tauri/src/db/ SQLite schema + migrations
status: proposed
owner: codex
stage: 7
depends_on: [005]
---

## Scope

Schema and migrations for Workspace, Goal, Concept, Module, Session, matching `src/types/*` (005) exactly.

## Plan

- src-tauri/src/db/schema.rs (or .sql migrations)
- src-tauri/src/db/mod.rs

## Worklog

- 2026-08-29 (claude-code): Scope note — `src/types/*` has grown since this task was
  written (Stage 6 added real domain types beyond the original five). Before starting,
  check the current barrel (`src/types/index.ts`) rather than trusting this file's
  original list. Specifically:
  - **Needs a table**: `Material` and `MaterialResult` (`material.ts`) — per-workspace
    mutable state (`highlightsCount`, `notesCount`, segment progress) that must survive
    restart, same as the original five.
  - **Needs a table**: `Note` (`note.ts`) — real user-created content, not catalog data.
  - **Needs a table**: `WorkspaceActivityEvent` (`workspace.ts`) — simple append-only log,
    bounded to 3 shown per workspace at the app layer (already enforced there), no need to
    enforce the bound in schema.
  - **Static/seeded, not a mutable table**: `WorkspaceTemplate` (`module.ts`) — marketplace
    catalog data, same treatment as `Module`'s own catalog rows (seeded, not user-mutated).
  - **Out of scope for this task**: `VisualizationScene` and its primitives
    (`visualization.ts`) — these are Stage-8-adjacent verified-primitive scene definitions,
    not user-owned mutable state; its own doc comment already frames it as forward-looking
    for a later real engine. Don't build schema for it now.
  This isn't a contract change — no `src/types/*` file needs editing for this task, just
  make sure the schema actually covers what's there today, not what was there when 037 was
  first written.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
