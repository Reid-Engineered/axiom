---
id: 017
title: Sidebar / WorkspaceTree implementation
status: proposed
owner: codex
stage: 3
depends_on: [009, 016]
---

## Scope

Implement `Sidebar` / `WorkspaceTree` against mock workspace names only (no real data wiring — that's Stage 4). Two-level expand rule enforced in code; only the "open" workspace expands. Codex implements; Antigravity polishes visual fidelity.

## Plan

- src/components/workspace/WorkspaceTree.tsx (real implementation)
- sidebar wiring inside src/layouts/AppShell.tsx

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
