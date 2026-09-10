---
id: 071
title: Active-session selection and reuse
status: review
owner: codex
stage: 8
depends_on: []
---

## Scope

Implement §6.1 of `docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.
Supersedes the Windows-walk observation filed verbally as "071 — sample resume practice dead
end"; that dead end is one of two symptoms of this task's root cause.

Gives the application a representation of "the session the learner is actually in": activity
timestamps to order by, and reuse semantics so "Practice this" reopens a concept's open
session instead of creating a new one every press.

Does **not** build: any frontend change (`070` and `073` own those), elapsed-*duration*
tracking (`sessions.elapsed_minutes` is untouched — that is `076`), the sample seed change
(`072`), or workspace concept provisioning (`075`).

## Plan

- `src-tauri/src/db/migrations/0004_session_activity.sql` — `ALTER TABLE sessions ADD COLUMN
  last_activity_at TEXT`; `ALTER TABLE workspaces ADD COLUMN created_at TEXT`. Both nullable
  so existing rows migrate untouched.
- `src-tauri/src/commands/session.rs` — write `sessions.last_activity_at` and the owning
  workspace's `last_activity_at` on start, resume, pause, `nextProblem`, and attempt
  evaluation. Change `get_active_session_by_workspace_handler`'s ordering to
  `ORDER BY last_activity_at IS NULL, last_activity_at DESC, rowid DESC`. Add reuse to
  `start_session_handler`.
- `src-tauri/src/commands/workspace.rs` — set `created_at` on workspace creation.
- `src-tauri/src/commands/seed.rs` — set `created_at` on imported sample workspaces.
- `src-tauri/src/commands/models.rs` — `Session.last_activity_at`, `Workspace.created_at`.
- `src/types/session.ts`, `src/types/workspace.ts` — `lastActivityAt`, `createdAt` on the wire
  models. `070`'s startup comparator runs in the frontend and needs `createdAt`.
- `src/test/mockBackend.ts` — mirror the new fields and the reuse behaviour, so the double
  does not stay richer than the backend (see design §4.3).
- `src-tauri/src/commands/tests.rs` — the test matrix below.

**Reuse rule.** Given `(workspace_id, concept_id)` with an existing non-completed session,
return that session rather than inserting. If its `current_attempt_id` is null or its attempt
is solved, bind a fresh attempt onto it — the session row persists, the problem advances.
This is an approved change to `startSession`'s Stage 7 semantics, not an incidental one.

**Tests.**

- Migration: existing `sessions` and `workspaces` rows survive with both new columns unset.
- Selection ordering: recent real activity wins over **an older open session created by test
  setup**. Construct the older session in the test — do not import `mockData` fixtures, which
  `072` removes.
- Selection ordering: all-null `last_activity_at` falls back to `rowid DESC`.
- Reuse: two and three `startSession` calls for one `(workspace, concept)` yield one row and
  the identical prompt.
- Reuse after solving: same session row, new attempt bound.
- `workspaces.last_activity_at` advances on session activity and on nothing else.
- All four existing degradation branches still succeed: unmapped concept, module disabled,
  forced invoke error, missing workspace.
- `sessions.elapsed_minutes` is unchanged by every path this task touches.

If the file list grows materially once work starts, split rather than expand — see
`.ai/lifecycle.md`.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, `proposed` for codex.
- 2026-09-09 — Claimed by codex on `agent/codex/071-active-session-selection-and-reuse`; beginning TDD implementation of the locked activity-ordering and session-reuse contract.
- 2026-09-09 — Implementation complete. Gates run by claude on a Linux (WSL) toolchain,
  because the Windows MSVC linker is unavailable here and the GNU toolchain fails to link
  the Tauri cdylib with `export ordinal too large`. Those are toolchain failures, not test
  results, and were not counted as passes.
- 2026-09-09 — `cargo fmt --check` failed on two files (`commands/seed.rs`,
  `commands/session.rs`, both import-line wrapping). claude ran `cargo fmt`; no semantic
  change. This is the only edit claude made to this branch.
- 2026-09-09 — All applicable gates green; moved to `review`. CI on the PR is the source of
  truth for the full required set, including e2e.

## What was built / tested / left out

- Added `sessions.last_activity_at` and `workspaces.created_at` as nullable columns in
  migration `0004_session_activity.sql`, bumped `LATEST_SCHEMA_VERSION` to 4, and exposed
  `lastActivityAt` / `createdAt` on the `Session` and `Workspace` wire models.
- Changed `get_active_session_by_workspace_handler` to
  `ORDER BY last_activity_at IS NULL, last_activity_at DESC, rowid DESC`, so the most
  recently touched session wins instead of the lowest `rowid`.
- Added reuse to `start_session_handler`: an existing non-completed session for the same
  `(workspace, concept)` is returned rather than a new row inserted. A fresh attempt binds
  only when the session has no attempt or `practice.describe` reports the current one
  solved; `problem_index` advances only in that case.
- Both `sessions.last_activity_at` and the owning `workspaces.last_activity_at` are written
  on start, resume, pause, `nextProblem`, and attempt evaluation, each inside a transaction.
- `commands/practice.rs` gained a call to `touch_session_for_attempt`. That file was not in
  the Plan's list: attempt evaluation lives there, and the task requires evaluation to
  advance session and workspace activity, so the alternative was leaving one of the five
  named activity paths unwired. Disclosed here rather than made silently.
- `mockBackend.ts` mirrors all of it — selection ordering, reuse, and the activity touches —
  so the double does not stay richer or poorer than the backend it stands in for.
- `elapsed_minutes` is untouched on every path, asserted directly by
  `session_activity_updates_both_timestamps_without_changing_elapsed_minutes`.

**Tested** (all run by claude on Linux; exact commands and results in the Review section):
seven new Rust tests covering reuse, solved-attempt rebinding, selection ordering against an
older open session created by test setup, both-timestamp updates without elapsed change,
non-session mutations not advancing workspace activity, missing-workspace rejection, and
migration nullability. `cargo test` 312 passed / 0 failed; `cargo clippy --all-targets -- -D
warnings` clean; `cargo fmt --check` clean after the fmt fix; `npm run typecheck`, `npm run
lint`, `npm run build`, `npx vitest run` (61 files / 165 tests) all pass; design-token grep
returns nothing outside `tokens.css`.

**Left out as scoped:** `elapsed_minutes` duration tracking (`076`), the boot rule (`070`),
the Study Session screen (`073`), the sample seed (`072`), and workspace provisioning (`075`).

## Review

## Follow-ups
