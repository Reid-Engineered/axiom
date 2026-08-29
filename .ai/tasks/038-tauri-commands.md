---
id: 038
title: src-tauri/src/commands/* handlers
status: proposed
owner: codex
stage: 7
depends_on: [037, 020]
---

## Scope

One `#[tauri::command]` per current mock-service function from 020, backed by 037's schema. `cargo test` covers command handlers and queries.

## Plan

- src-tauri/src/commands/workspace.rs
- src-tauri/src/commands/goal.rs
- src-tauri/src/commands/concept.rs
- src-tauri/src/commands/module.rs
- src-tauri/src/commands/session.rs
- src-tauri/src/commands/material.rs (Stage 6 addition — see Worklog)
- src-tauri/src/commands/note.rs (Stage 6 addition — see Worklog)

## Worklog

- 2026-08-29 (claude-code): Scope note, same reasoning as 037's — `src/services/*` grew
  during Stage 6 beyond the five files this task was originally scoped against. Check the
  current `src/services/` directory rather than trusting this file's original list.
  `materialService.ts` and `noteService.ts` are real, already-reviewed services that need
  command coverage same as the original five (`getMaterial`/`searchMaterial`,
  `getRecentNotes`). `workspaceService.ts` also grew a `getRecentActivity` function — cover
  it in `commands/workspace.rs` alongside the rest of that file's functions, no new file
  needed for it. `moduleService.ts` also grew `getWorkspaceTemplates()` — cover it in
  `commands/module.rs` alongside the rest of the module catalog, no new file needed either.
- 2026-08-29 (claude-code): Important, discovered hands-on while reviewing 037 —
  `workspaces.guiding_goal_id` has a `DEFERRABLE INITIALLY DEFERRED` foreign key to
  `goals(id)` (needed for the circular workspace↔guiding-goal relationship). This only
  actually defers within an **explicit transaction**. A plain autocommit
  `connection.execute()`/`execute_batch()` inserting a workspace before its goal exists will
  fail immediately with `FOREIGN KEY constraint failed` — confirmed by testing it directly.
  Any command creating a workspace and its guiding goal together (`create_workspace`, most
  obviously — likely others) must wrap both inserts in one `connection.transaction()`, not
  separate autocommit calls. This isn't optional or an edge case; it'll fail on the very
  first `create_workspace` call otherwise.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
