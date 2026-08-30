---
id: 041
title: Runtime sample-workspace SQLite seed import
status: proposed
owner: unassigned
stage: 7
depends_on: [037, 038, 039]
---

## Scope

Build the first-launch/sample-workspace import path that inserts `src/services/mockData/*`
fixture content into SQLite as real rows, per 039's scope note ("Mock fixtures repurposed as
first-launch/sample-workspace seed data, not deleted"). This does not touch
`src/services/*`'s `invoke()` bodies (039's scope) or add a new IPC command surface beyond
whatever minimal import command this needs — it is data, not new business logic.

## Plan

Files likely touched (confirm against current tree before starting, per `.ai/lifecycle.md`):

- A new import path, either a `#[tauri::command]` in `src-tauri/src/commands/` or an
  app-startup hook — decide which based on whether it needs to be user-triggered
  (re-importable "explore a sample workspace" per `AXIOM-HANDOFF.md`'s first-launch screen)
  or is truly first-run-only.
- Whatever converts `src/services/mockData/*.ts` fixture shapes into the insert statements.

## Worklog

- 2026-08-29 (claude-code, from 039): Two data-consistency questions to resolve while
  scoping this, both found by reading 037's actual schema rather than the mock fixtures
  alone:
  1. `concepts.notes_count` (`src-tauri/src/db/migrations/0001_initial.sql:91`) is
     trigger-maintained (`notes_count_after_insert`/`_after_delete`/`_after_concept_change`,
     same file lines 292-316) — it is **never** set directly by any command. Do not copy
     `mockConcepts[i].notesCount` into an INSERT; insert concept rows without that column
     (defaults to 0) and insert the real `mockNotes` fixture row(s) — the trigger derives the
     correct count. Fabricating extra Note text to match the mock's synthetic
     `index % 5`/`index % 3` counts is unnecessary and was explicitly ruled out in 039's
     Worklog as fabricating learner data.
  2. `materials.notes_count` and `materials.highlights_count`
     (`src-tauri/src/db/migrations/0001_initial.sql:242`) are **not** trigger-backed — plain
     columns with only a `CHECK (>= 0)`. `mockData/material.ts` hardcodes `notesCount: 18`
     against the same single real `mockNotes` fixture used everywhere else. Unlike concepts,
     the DB won't catch a mismatch here, but showing "18 notes" on the Material page when
     only one real note exists anywhere in the seeded workspace is the same
     fabricated-data problem in spirit. Needs an explicit decision before this task starts:
     either seed `materials.notes_count`/`highlights_count` as 0 (or some other value
     genuinely backed by seeded rows), or determine from `AXIOM-HANDOFF.md` whether this
     field is meant to represent book-level annotations distinct from concept-linked
     `notes` rows (in which case it may legitimately not need to reconcile at all — confirm
     rather than assume).

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
