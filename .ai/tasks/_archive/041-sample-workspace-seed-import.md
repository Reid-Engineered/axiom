---
id: 041
title: Runtime sample-workspace SQLite seed import
status: done
owner: codex
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
- 2026-08-29 (codex): Claimed after confirming no other in-progress task touches the seed
  importer, command registry, first-launch flow, or related services. `AXIOM-HANDOFF.md`
  Screen 18 and `18-material-textbook.png` label these aggregates “YOUR MARKS IN THIS BOOK”
  and describe answers pinned to passages, while the locked `Note` contract is concept-linked
  and the schema has no note-to-material relation. The importer will preserve
  `materials.highlights_count` and `materials.notes_count` as book-level aggregates,
  independently of concept-note rows.
- 2026-08-29 (codex): Implemented the typed fixture payload, transactional Rust normalizer,
  idempotent `importSampleWorkspace` command, hook integration, and first-launch action. The
  first backend test pass covered rollback, re-import without resetting learner changes,
  reconstruction through domain query handlers, trigger-derived concept note counts, and
  preserved material mark aggregates (14 Rust tests passed). A full-fixture constraint audit
  found four tutor exchanges whose IDs were reused across two sessions even though the schema
  makes exchange IDs globally unique; corrected the retained session fixture and added a
  frontend payload regression test for nested identity uniqueness.
- 2026-08-29 (codex): Updated the existing app navigation tests to await the now-asynchronous
  sample import, then completed the full exit gates: `npm run typecheck`, `npm run lint`,
  `npm run build`, and all 57 frontend test files / 135 tests passed; `cargo fmt --check`,
  `cargo check`, all 14 Rust tests, and `cargo clippy --all-targets -- -D warnings` passed.
  Scoped Prettier, hardcoded-value scans, and `git diff --check` were also clean. Moved to
  review.

## What was built / tested / left out

Built:

- Added a typed `SampleWorkspaceSeed` IPC payload and the domain-specific,
  transaction-wrapped `importSampleWorkspace` command. It normalizes the retained fixtures
  across all existing SQLite tables and returns the requested sample workspace.
- Kept `concepts.notes_count` out of concept INSERTs and inserted only real `Note` fixtures,
  allowing the existing triggers to derive the count. Preserved material highlight/note
  aggregates as the handoff's distinct book-level marks.
- Made re-import idempotent without overwriting changes already made inside the sample
  workspace.
- Wired the command through a dedicated frontend service, `useWorkspaces`, and the
  first-launch “Explore a sample workspace” action, including loading and failure behavior.
- Corrected reused tutor-exchange fixture IDs so the retained sample graph satisfies the
  schema's global primary-key constraint.

Tested:

- Rust command tests cover normalized reconstruction through domain handlers, circular-FK
  transaction handling, trigger-derived concept note counts, material aggregates, complete
  rollback on an invalid fixture, and idempotent re-import that preserves learner changes.
- Frontend tests cover the full 92-concept fixture payload and its references, unique nested
  identities, hook de-duplication, first-launch success/failure behavior, and asynchronous app
  navigation.
- All frontend and backend quality gates named in the final Worklog entry passed.

Left out:

- No automatic startup seed was added. The handoff presents sample exploration as an explicit
  first-launch choice, so data is imported only when the learner selects it.
- No synthetic notes were created to reproduce fixture-only concept counters, and no raw SQL
  surface was exposed to the frontend.

## Review

Reviewer: claude-code
Date: 2026-08-29

Independently reran every gate rather than trusting the Worklog's claim: `npm run
typecheck`, `npm run lint`, `npm run build`, `npm test -- --run` (57 files / 135 tests,
matches), `cargo test --locked` (14 tests, matches — including all three new
`seed::` tests), `cargo check --locked`, `cargo fmt --check`, `cargo clippy --all-targets
--locked -- -D warnings`, scoped `npx prettier --check`, `git diff --check`, and a
hardcoded-value grep — all clean.

- [x] Correctness — pass, verified hands-on:
      1. `import_sample_workspace_handler` (`seed.rs:564`) checks for the sample workspace's
         existence before opening a transaction and returns early if found — combined with
         the whole import running inside one `connection.transaction()`, this makes
         "imported or not" atomic, so the idempotency check can't observe a half-imported
         state. Confirmed with the existing rollback test
         (`sample_import_rolls_back_every_table_when_one_fixture_is_invalid`, a real FK
         violation — checked `PRAGMA foreign_keys = ON` is actually set in
         `db/mod.rs:30`, not a no-op) and the idempotency test (re-import after a direct
         `UPDATE` preserves the mutated row and doesn't duplicate it).
      2. `insert_concepts` (`seed.rs:177`) omits `notes_count` from its INSERT, letting
         037's triggers derive it — `sample_import_normalizes_the_seed_and_preserves_owned_counts`
         directly asserts the imported concept's `notes_count` is `1` (from the one real
         seeded Note) even though the seed's own JSON fixture says `notesCount: 4`, proving
         the synthetic mock value is actually discarded, not just theoretically ignored.
      3. The `materials.notesCount` question this task was scoped to resolve: I opened
         `reference/UI/screenshots/18-material-textbook.png` directly rather than taking the
         Worklog's citation on faith. The screenshot does show a "YOUR MARKS IN THIS BOOK"
         panel reading "41 highlights · 18 notes" and "Most marked: §7.3, §8.2, §11.4" —
         numbers that match `mockMaterials` exactly. This is a real, verified basis for
         treating `materials.highlights_count`/`notes_count` as a distinct book-level
         reading-annotation aggregate, correctly preserved as-is rather than reconciled
         against the concept-linked `notes` table.
      4. The duplicate tutor-exchange fixture IDs (`mockData/sessions.ts`) are fully fixed —
         grepped the file for every exchange id source; all three `longTutorHistory` slices
         now remap to distinct prefixes (`exchange-shell-`, `exchange-parts-`,
         `exchange-eigen-`), and `sampleWorkspaceService.test.ts` directly asserts global
         exchange-id uniqueness rather than relying on this being noticed incidentally.
      5. `App.test.tsx`'s `enterSampleWorkspace()` helper and every navigation test using it
         were correctly made `async` to await the now-asynchronous import — an easy thing to
         miss that would have left tests passing for the wrong reason (racing the assertion
         against a still-pending promise).
- [x] Architecture conformance — pass. `sampleWorkspaceService.ts` is the only new
      `invoke()` call site, consumed by `useWorkspaces`'s `importSample` and called only from
      `FirstLaunchPage` (a page) — hooks-call-services / pages-call-hooks (§5 rule 1) holds.
      No new global state (result flows through the existing `useAsyncResource`-backed hook
      state). No new shared type was added for `SampleWorkspaceSeed` on the frontend side —
      correct, since the payload is built once at a single call site from already-typed mock
      arrays; a dedicated TS interface mirroring the Rust struct would be pure duplication.
      `ARCHITECTURE.md` correctly wasn't touched — this task adds a service+command pair
      following the existing pattern, not a new data-flow rule.
- [x] UI rules — pass. `<p role="alert">{sampleImportError}</p>` on `FirstLaunchPage.tsx:86`
      is not a novel notification pattern despite `ARCHITECTURE.md` §7 listing "notifications"
      as not-yet-designed — grepped the rest of `src/pages/` and found the identical bare
      `role="alert"` convention already used for load failures in `MaterialPage`,
      `ConceptsListPage`, `ModuleDetailPage`, `MarketplacePage`, `StudySessionPage`,
      `ConceptViewPage`, `CreateWorkspacePage`, and `GoalEditingSheet` — this reuses an
      established pattern rather than inventing one. No hardcoded design values introduced.
- [x] Process — pass. Worklog is detailed and dated per entry, not batched. Scope matches
      the task file (the seed importer plus the two contract questions it was created to
      settle); no unrelated refactor beyond incidental Prettier re-wrapping of
      already-touched JSX lines in `FirstLaunchPage.tsx` (cosmetic, gate-verified, not a
      logic change).

Verdict: approved. No blocking findings.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
