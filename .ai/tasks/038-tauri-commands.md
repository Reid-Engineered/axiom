---
id: 038
title: src-tauri/src/commands/* handlers
status: review
owner: claude
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
- 2026-08-29 (Codex): Claimed the task after confirming 037 is approved and archived and
  no other in-progress task touches `src-tauri/`. Inventoried all current service exports:
  the command surface is 29 functions across workspace, goal, concept, module, session,
  material, and note domains. The Rust handlers will preserve each service's parameters and
  serialized return shape, with command/query regression tests added alongside each domain.
- 2026-08-29 (Codex): Implemented a shared Rust DTO layer with `camelCase` serialization,
  an internal mutex-protected `rusqlite` connection, and all 29 domain handlers plus thin
  registered `#[tauri::command]` wrappers. The app opens one `axiom.sqlite3` file in its
  app-data directory; no raw connection or SQL command crosses IPC. Added exact pins for the
  existing-lockfile versions of `chrono` and `uuid` for service-compatible timestamps and ids.
- 2026-08-29 (Codex): Matched the existing service behavior, including bounded oldest-first
  workspace activity, goal update/revert history, concept evidence search, workspace-scoped
  module state, full session lifecycle, in-syllabus material search, and newest-first notes.
  `create_workspace_handler` inserts its Workspace, Guiding Goal, and four offline rows in
  one explicit transaction, satisfying the deferred circular foreign key discovered in 037.
- 2026-08-29 (Codex): Added eight command tests covering every one of the 29 handlers and
  their query/mutation paths, including ordered nested reconstruction and frontend field-name
  serialization. Together with 037's three schema tests, `cargo test --locked` runs 11 tests.
  `cargo check --locked`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  and `git diff --check` also pass.

## What was built / tested / left out

- Built one command module per current service domain: workspace, goal, concept, module,
  session, material, and note, backed directly by 037's schema.
- Registered all 29 commands and initialized the single local database as managed Tauri state.
  IPC command names exactly match the existing camelCase TypeScript service names; multiword
  parameters and serialized DTO fields are camelCase as well, so 039's swap is direct.
- Tested all handlers through an in-memory production schema, including every mutation and
  error-sensitive lifecycle. No mock seed import or frontend `invoke()` swap is included;
  both belong to 039.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Tauri serializes Rust `Option<Session>::None` as JSON `null`. In 039,
  `getActiveSessionByWorkspace` should normalize the invoke result with `?? undefined` to
  preserve its existing `Promise<Session | undefined>` TypeScript contract.

## Review

Reviewer: claude-code
Date: 2026-08-29

Verified completeness directly rather than trusting the "29" count: grepped every
`src/services/*.ts` file for `export async function` (14 across concept/session/goal + 15
across material/module/note/workspace = 29) and cross-checked against every command
registered in `lib.rs`'s `invoke_handler!` — exact 1:1 match, nothing missing, nothing extra.

- [x] Correctness — pass, verified hands-on in two places, not just by reading:
      1. `create_workspace_handler` (`workspace.rs:162`) wraps the workspace, guiding goal,
         and four offline-availability rows in one explicit `connection.transaction()` —
         exactly satisfies 037's deferred-FK requirement. This is the first real usage of
         that pattern outside a test, and it's correct.
      2. `revert_goal_handler`'s single-statement swap (`SET text = previous_text,
         previous_text = text`) relies on SQL's simultaneous-assignment semantics (all
         right-hand sides evaluate against the pre-update row, not sequentially) — easy to
         get backwards if someone "simplifies" it later. Wrote a throwaway test proving it's
         correct today, then removed it since `goal_handlers_preserve_previous_text_and_revert`
         (`tests.rs:93`) already asserts the same round-trip through the real handler, which I
         confirmed passes.
      `get_recent_activity_handler`'s `ORDER BY occurred_at ASC LIMIT 3` matches
      `workspaceService.ts`'s existing `.sort().slice(0, 3)` exactly — not a new or different
      bound. `material_handlers_reconstruct_book_and_exclude_out_of_syllabus_results`
      (`tests.rs:313`) directly proves out-of-syllabus exclusion at the search layer, the
      same invariant 030's review traced through on the mock side.
- [x] Architecture conformance — pass. No `tauri-plugin-sql` or raw query surface; every
      command is a specific, named `#[tauri::command]` matching one service function.
      `models.rs`'s DTOs use `#[serde(rename_all = "camelCase")]` with
      `skip_serializing_if = "Option::is_none"` on optional *fields*, which elegantly avoids
      most null/undefined friction (an absent field is omitted, not nulled, matching
      TypeScript's `?:` semantics for free). The one documented Follow-up (bare top-level
      `Option<Session>` serializes as JSON `null`, since `skip_serializing_if` only applies
      to struct fields, not a whole return value) is a real, correctly-scoped exception, not
      an oversight — accurately flagged for 039 rather than hacked around on the Rust side.
- [x] Process — pass. Independently reran `cargo test --locked` (11 tests: 037's 3 schema
      tests + 038's 8 command tests, all pass), `cargo check --locked`, `cargo fmt --check`,
      `cargo clippy --all-targets --locked -- -D warnings`, and `git diff --check`; all clean.

Verdict: approved. No blocking findings. 039 is unblocked.
