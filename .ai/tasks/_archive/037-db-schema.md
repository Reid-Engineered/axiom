---
id: 037
title: src-tauri/src/db/ SQLite schema + migrations
status: done
owner: claude
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
- 2026-08-29 (Codex): Claimed the first Stage 7 task after confirming no other in-progress
  task touches `src-tauri/`. Re-read the current `src/types/index.ts` barrel, Architecture
  runtime boundary, Stage 7 scope, and backend gates. Selected internal `rusqlite` with its
  bundled SQLite build; no SQL execution will be exposed to the frontend.
- 2026-08-29 (Codex): Added the version-1 schema and a transactional migration runner for
  Axiom's single local database. Pinned every direct Cargo dependency exactly, including
  `rusqlite = 0.40.2` with bundled SQLite, and retained the architecture boundary: the
  database module is Rust-only and registers no frontend SQL plugin or raw query command.
- 2026-08-29 (Codex): Used normalized, ordered child tables for array-shaped contract fields
  (goal tools, concept edges and evidence, module relationships, session exchanges and
  conclusions, material segments, and marked sections). Flattened fixed optional objects
  whose fields are queried together (`Goal.inferred`, `Session.intent`, and offline partial
  availability) rather than storing opaque JSON. Module catalog rows retain the contract's
  baseline `enabled` and `visibility` fields, while workspace-specific state and
  `Workspace.enabledModuleIds` normalize through `workspace_modules`.
  `MaterialResult.material_id` is an internal ownership key needed to scope results and is
  not a frontend contract addition. `Concept.notesCount` remains a stored contract field and
  is maintained by note triggers.
- 2026-08-29 (Codex): Updated `ARCHITECTURE.md` to reflect the new persistence layer. Verified
  the migration directly against an in-memory SQLite database (24 domain tables), then ran
  `cargo check --locked`, `cargo fmt --check`, and `git diff --check` successfully. This
  initial verification was ad hoc rather than a permanent Rust test; no frontend files or
  TypeScript contracts changed.
- 2026-08-29 (Codex): Addressed review by adding three permanent Rust regression tests. They
  verify that migrations apply once at the current version and create all 24 domain tables,
  a workspace rejects a second Guiding goal, and note-count triggers handle insert, concept
  reassignment, and delete. The shared fixture inserts the circular workspace/guiding-goal
  pair inside an explicit transaction, giving 038 a working example of the required write
  shape. `cargo test --locked` passes all 3 tests; `cargo check --locked`,
  `cargo fmt --check`, and `git diff --check` also pass.

## What was built / tested / left out

- Built `src-tauri/src/db/` with connection configuration, ordered transactional migrations,
  schema-version tracking, and the initial schema covering Workspace, Goal, Concept, Module,
  WorkspaceTemplate, Session, Material/MaterialResult, Note, and WorkspaceActivityEvent.
- Added foreign keys, enum/boolean/range checks, ordering constraints, lookup indexes, the
  one-guiding-goal invariant, and note-count maintenance triggers.
- Pinned all direct Cargo dependencies and generated a locked `rusqlite`/SQLite dependency
  graph. No `tauri-plugin-sql` dependency or frontend-visible database surface was added.
- Tested with `cargo test --locked` (3 schema tests) and `cargo check --locked`; also checked
  Rust formatting and patch whitespace. There are intentionally no command handlers or seed
  data here; those are tasks 038 and 039.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Task 039 should seed `Note` rows consistently with each concept's `notesCount`; after initial
  import, the schema triggers keep that denormalized contract count synchronized.

## Review

Reviewer: claude-code
Date: 2026-08-29

Read the full 318-line migration and `db/mod.rs` directly, not just the worklog summary.
The schema itself is genuinely strong: correct handling of the circular
`workspaces.guiding_goal_id` ↔ `goals.workspace_id` dependency via `DEFERRABLE INITIALLY
DEFERRED`, several product invariants correctly encoded as CHECK constraints rather than left
to application code (`one_guiding_goal_per_workspace` unique index, `session_settled_conclusions`
capped at 2 rows matching the type's "up to two conclusions" doc comment, `material_chapter_segments`
capped at exactly 4 matching "always four segments"), sound normalized-vs-flattened judgment
calls (documented inline in the Worklog), and no frontend-visible DB surface.

- [ ] Correctness — FAIL, but not in the schema: **zero automated tests exist** for 318 lines
      of schema, 3 triggers, and numerous CHECK constraints. The worklog's "tested" claim
      ("execution of the full migration in SQLite, including constraint validation") is a
      manual/ad hoc check, not a `#[test]` — `grep -rn "#\[test\]" src-tauri/src/db/` returns
      nothing. I verified this concern hands-on: wrote three throwaway tests (migration
      applies and creates ≥24 tables; the one-guiding-goal constraint rejects a second
      Guiding goal; the notes-count trigger tracks insert/delete correctly), ran them, then
      removed them since adding tests isn't my role here. All three passed once I got the
      test right — **the schema is correct** — but getting there surfaced something 038
      needs to know:
      - `workspaces.guiding_goal_id`'s deferred FK only actually defers within an **explicit
        transaction** (`connection.transaction()` in rusqlite). A plain autocommit insert of
        a workspace referencing a not-yet-existing goal fails immediately with `FOREIGN KEY
        constraint failed` — I hit this myself on the first attempt. Any command that creates
        a workspace and its guiding goal together (`create_workspace`, most obviously) must
        wrap both inserts in one transaction, or workspace creation will be broken from the
        very first call. Added this to 038's task file directly since it's actionable
        information that task needs before writing `create_workspace`, not a finding against
        this task's schema design (which is correct as-is).
      This isn't a hard bug in 037's deliverable, but per this session's own standard (029
      was sent back purely for missing regression tests on an otherwise-correct fix), a
      foundational schema this subtle (deferred FKs, triggers, capped-position constraints)
      shouldn't rely on a one-time manual check as its only verification. Recommend at least
      three tests before this is fully done: migration applies cleanly, the guiding-goal
      constraint is enforced, and the notes-count trigger is correct — I've already proven
      all three are cheap to write and pass.
- [x] Correctness — pass otherwise, confirmed by direct read: every field from
      `Workspace`/`Goal`/`Concept`/`Module`/`Session`/`Material`/`MaterialResult`/`Note`/
      `WorkspaceActivityEvent`/`WorkspaceTemplate` is represented; array-shaped fields are
      normalized into ordered child tables with `position` columns preserving order;
      `VisualizationScene` correctly excluded per the updated scope note.
- [x] Architecture conformance — pass. No `tauri-plugin-sql` or other frontend-visible query
      surface; `lib.rs` only adds `pub mod db;` internally. `ARCHITECTURE.md` §1/§2 updated
      accurately to describe the current phase.
- [x] Process — pass on what's checkable pre-038. Independently reran `cargo check --locked`
      and `cargo fmt --check` (both clean) and `git diff --check`; all Cargo dependencies
      pinned to exact versions in `Cargo.toml`, matching the Stage 0 risk note.

Verdict: changes-requested. The schema design itself is approved as-is — this isn't asking
for a redesign. Blocking on: add automated test coverage (the three cases above are a
reasonable minimum) before this task is `done`, since nothing currently protects this schema
from a silent regression, and 038/039 will build directly on top of it.

## Follow-ups (from review)

- (claude-code, 2026-08-29) Added a note directly to 038's task file: `create_workspace` (and
  any other command creating a workspace + its guiding goal together) must wrap both inserts
  in an explicit `connection.transaction()` for the deferred FK to succeed — a plain
  autocommit insert will fail immediately. Discovered hands-on while verifying this review,
  not theoretical.

## Re-review (test coverage)

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. `src-tauri/src/db/tests.rs` covers all three recommended cases,
      plus more: migration idempotency (`migration_count == 1`, not just version), exact
      `domain_table_count == 24`, the guiding-goal constraint asserted by specific
      `ErrorCode::ConstraintViolation` rather than just `is_err()`, and the notes-count
      trigger test goes beyond insert/delete to also cover **concept reassignment**
      (`UPDATE notes SET concept_id = ...`), which I hadn't explicitly asked for. The shared
      `insert_workspace_and_guiding_goal` fixture wraps both inserts in an explicit
      `connection.transaction()` — exactly the pattern 038 needs and now has a working
      example of, not just a written warning.
- [x] Process — pass. Independently reran `cargo test --locked` (3 tests, all pass),
      `cargo check --locked`, `cargo fmt --check`, and `git diff --check`; all clean.

Verdict: approved. No blocking findings remain. 038 is unblocked.

## Merge

2026-08-29 — Code committed to `master` at `b451790`. Status moved to `done`; file archived.
