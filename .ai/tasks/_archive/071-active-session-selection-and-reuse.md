---
id: 071
title: Active-session selection and reuse
status: done
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
- 2026-09-10 — CI caught what local Linux runs could not: `backend-checks (windows-latest)`
  failed `session_activity_updates_both_timestamps_without_changing_elapsed_minutes` on a
  one-millisecond mismatch. **A test defect, not a production one.** The array literal
  collecting `pause` and `resume` evaluated both handlers before the loop body ran, so the
  first iteration compared pause's timestamp against a workspace row resume had already
  overwritten. Linux and macOS passed only because both writes landed in the same
  millisecond. claude rewrote the assertion to check each handler immediately after it runs
  and re-ran the suite: 312 passed / 0 failed, clippy and fmt clean.
- 2026-09-10 — **Review independence:** claude has now made two edits to this branch — the
  `cargo fmt` wrapping and this test-assertion fix. The second is a material change to test
  code, so claude must not be the sole approver. This PR needs a second reviewer (codex or
  the human) for at least those two commits.
- 2026-09-10 — Rebased onto `master` at `8096d09`, after `072` (#20) and `075` (#21) merged.
  Three conflicts, all resolved deliberately rather than by taking a side wholesale:
  - `src/services/mockData/workspaces.ts` — kept `createdAt`, **dropped** the `lastActivityAt`
    this branch had added. `072` removed seeded activity on purpose and `070`'s boot ordering
    reads that column, so restoring it would have re-fabricated an active workspace. A
    creation timestamp is identity, not activity, so `createdAt` stays.
  - `src-tauri/src/commands/seed.rs` — kept `now` in the import list (this branch needs it for
    `created_at` on sample import) and dropped `Session`, which became unused once `072`
    removed `insert_sessions`.
  - `src-tauri/src/commands/workspace.rs` — kept `075`'s async signature and two-transaction
    provisioning, and added this branch's `created_at` to its workspaces `INSERT`.
  Gates re-run on the rebased result, which is the first time all three tasks were compiled
  together: `cargo test` 318 passed / 0 failed (master's 311 plus this task's 7), clippy and
  fmt clean, `npm run typecheck` / `lint` / `build` pass, `npx vitest run` 61 files / 165
  tests, design-token grep empty.

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

Reviewer: claude
Date: 2026-09-10

Self-review, explicitly permitted by the human after claude made two edits to this branch
(the `cargo fmt` wrapping and the Windows test-assertion fix). Both are named below rather
than folded into a general pass, so the record shows exactly what was reviewed by its own
author.

Verified independently rather than from the PR body: the branch was checked out, every gate
re-run on a Linux toolchain against the rebased tree, and the session and workspace diffs read
against design §6.1 line by line.

- [x] **Correctness — pass.** The code does what the task claims, and the tests cover the
      stated behaviour rather than the happy path only.
      - The ordering clause is exactly the one the design locked, and
        `active_session_selection_prefers_recent_activity_then_newest_row` proves it against an
        older open session built in the test rather than imported from `mockData` — so it does
        not depend on the fixtures `072` removed, which was the specific risk here.
      - Reuse is proven in both directions: same session and same prompt on a second
        `startSession`, and same session with a *new* attempt once the current one is solved.
      - `attempt_is_solved` goes through `practice.describe` rather than reading Practice's
        tables, so Core still does not reach into the module's storage.
      - `elapsed_minutes` staying untouched is asserted directly, not assumed — which matters,
        because the whole point of deferring `076` is that this task must not start tracking
        duration.
      - Edge cases from the schema are handled: `problem_index` advances only when an attempt
        actually bound (`WHEN ?3 AND ?2 IS NOT NULL`), completed sessions are excluded from
        both reuse and selection, and all four pre-existing degradation branches still return
        a usable session.
- [x] **Architecture conformance — pass.** No frontend hook or service changed; `lastActivityAt`
      and `createdAt` are optional fields on the existing `Session` and `Workspace` interfaces
      in `src/types/`, already re-exported by `index.ts`'s `export *`. No new global state.
      Both new columns are nullable, so the migration cannot fail on existing rows —
      `session_activity_columns_are_nullable_and_preserve_existing_rows` proves it starting
      from the prior schema rather than a fresh one. No structural change, so no
      `ARCHITECTURE.md` update is owed.
- [x] **UI rules — N/A.** No file under `src/components/`, `src/pages/`, or any `.module.css`
      changed. The design-token grep returns nothing outside `tokens.css`.
- [x] **Process — pass.** All seven required checks green on `0669c87`. The worklog is
      detailed enough to follow the change without the diff, including the Windows failure and
      the three rebase resolutions. One scope deviation — `commands/practice.rs` — is disclosed
      in the task file rather than made silently, and is justified: attempt evaluation lives
      there and is one of the five activity paths the task must wire.

### The two edits claude made, stated plainly

1. **`cargo fmt` on `commands/seed.rs` and `commands/session.rs`.** Import-line wrapping only.
   Codex's record said fmt passed; it did not. No semantic change.
2. **The Windows test-assertion fix.** `backend-checks (windows-latest)` failed on a
   one-millisecond mismatch. The array literal collecting `pause` and `resume` evaluated both
   handlers before the loop body ran, so the first iteration compared pause's timestamp against
   a workspace row resume had already overwritten. **The handlers were always correct** — this
   was a test that passed by timing luck on Linux and macOS. Each handler is now asserted
   immediately after it runs.

### Observations (none blocking)

1. **A solved attempt's id can be dropped in a degraded path.** In the reuse branch, if
   `needs_attempt` is true because the current attempt is solved *and* `practice.start` then
   fails or the concept has no crosswalk, `new_attempt_id` is `None` and the `UPDATE` writes
   `current_attempt_id = NULL` — so the session loses its pointer to the solved attempt and the
   screen falls back to the unbound state instead of the solved one. Only reachable when
   Practice is already broken or a crosswalk was removed after an attempt bound, and the
   session itself survives, which is what the degradation contract actually promises. Worth
   knowing before `073` renders these states.
2. **`072`'s long-absence test will shortly be able to drop its `mockIPC` override.** That test
   injects `lastActivityAt` onto a workspace because `072` removed the seeded value. Once this
   task lands, `startSession` writes `workspaces.last_activity_at` itself, so the override
   becomes redundant. Not this task's to change.
3. **The rebase dropped `lastActivityAt` from `mockData/workspaces.ts`,** which this branch had
   added. Recorded here because it is the one resolution a future reader might mistake for a
   lost change: it was deliberate, since `072` removed seeded activity on purpose and `070`'s
   boot ordering reads that column.

Verdict: approve


## Follow-ups
