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

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
