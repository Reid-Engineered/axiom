---
id: 039
title: src/services/* swap to invoke() calls
status: done
owner: claude
stage: 7
depends_on: [038]
---

## Scope

Swap `src/services/*` from `mockData/` reads to `invoke()` calls — signatures unchanged from 006/020. No page, hook, or component outside `services/*` should need to change; if one does, that's a finding against Stage 2's contract-locking worth a retro before continuing. Mock fixtures repurposed as first-launch/sample-workspace seed data, not deleted.

## Plan

- src/services/workspaceService.ts
- src/services/goalService.ts
- src/services/conceptService.ts
- src/services/moduleService.ts
- src/services/sessionService.ts

## Worklog

- 2026-08-29 (Codex, from 038): `get_active_session_by_workspace` returns Rust
  `Option<Session>`, whose IPC representation is `Session | null`. Preserve the locked
  service signature `Promise<Session | undefined>` by normalizing the invoke result with
  `?? undefined`; callers and hook tests should not change.
- 2026-08-29 (Codex): Claimed the task after confirming 038 is approved and archived and
  no other in-progress task touches the service boundary. Confirmed all seven current
  service domains map 1:1 to the 29 registered camelCase commands. No separate seed task
  exists; the retained fixtures will back the mocked IPC boundary in unit tests while the
  runtime sample-data import is assessed separately below.
- 2026-08-29 (Codex): Replaced all mock-data service bodies with typed `invoke()` calls to
  the 29 approved command names. Every exported parameter and return signature is unchanged;
  no hook, page, component, or shared type changed. `getActiveSessionByWorkspace` explicitly
  converts Rust's top-level `null` to the existing JavaScript `undefined` contract.
- 2026-08-29 (Codex): Repurposed the retained fixtures behind Tauri's official `mockIPC`
  adapter for frontend tests. The adapter implements the same 29 commands and resets its
  cloned state before every test, so hooks/pages now exercise the real invoke-backed service
  layer rather than importing an alternate service implementation. Added a direct regression
  test for the `null` normalization. All 55 files / 132 tests pass, as do typecheck, lint,
  build, scoped Prettier, and `git diff --check`.
- 2026-08-29 (Codex): Ran `npx tauri build --debug --no-bundle --ci`; the production frontend
  and Rust application compiled together successfully. The CLI emitted the pre-existing
  warning that `com.axiom.app` ends in `.app`; no identifier or packaging configuration was
  changed in this service-boundary task.
- 2026-08-29 (claude-code): Resolved the seed-data contract question below by reading 037's
  actual schema rather than reasoning from the mock fixtures alone.
  `concepts.notes_count` (`src-tauri/src/db/migrations/0001_initial.sql:91`) is a
  DB-trigger-maintained denormalized column — `notes_count_after_insert` /
  `_after_delete` / `_after_concept_change` (same file, lines 292-316) derive it solely from
  actual rows in the `notes` table; no command sets it directly (`load_concept` in
  `concept.rs` only reads it), and grepping the whole tree for `INSERT INTO concepts` finds
  only Rust test helpers — no production seed/import command exists yet, in `src-tauri/` or
  `src/services/`. So `mockConcepts[i].notesCount` (a synthetic `index % 5` / `index % 3`
  pattern generated for pre-Stage-7 `ConceptRow` scale testing, see `mockData/concepts.ts`)
  was never wired as a seed source of truth and needs no reconciliation: whoever builds the
  runtime importer inserts concept rows without specifying `notes_count` (defaults to 0 per
  the schema) and inserts only the real `mockNotes` fixture row(s) — the trigger derives the
  correct count automatically. No note text needs to be fabricated, and 037's
  count-consistency invariant holds by construction. Confirming the split: 039's own scope
  (the `invoke()` swap) is functionally complete per the Worklog above; the runtime
  sample-workspace import is split out to `041-sample-workspace-seed-import.md` (`proposed`),
  which also inherits a related, not-yet-resolved question about `materials.notesCount`
  (`mockData/material.ts:18`, value 18 against the same single real Note fixture) — that
  column has no trigger backing it, so it needs its own look rather than blocking here.
  039 is unblocked to move to `review` once Codex fills in "What was built / tested / left
  out" below.

## What was built / tested / left out

- Replaced all seven service-domain implementations (29 exported functions) with typed
  `invoke()` calls using the command names and camelCase payloads approved in 038. Public
  function parameters and return types are unchanged.
- Preserved `getActiveSessionByWorkspace(): Promise<Session | undefined>` by converting the
  Rust command's top-level `null` response to `undefined`, with a focused regression test.
- Installed Tauri's official IPC mock in the shared Vitest setup and moved retained fixture
  behavior behind a resettable test backend. Existing hook and page tests now cross the same
  service/IPC boundary used in production; no alternate mock service implementation exists.
- Verified with `npm run typecheck`, `npm run lint`, `npm run build`, `npm test` (55 files,
  132 tests), scoped `npx prettier --check`, `git diff --check`, and
  `npx tauri build --debug --no-bundle --ci`.
- No hook, page, component, shared type, or command contract changed. Runtime sample-workspace
  import is intentionally left out and tracked by 041.

## Review

Reviewer: claude-code
Date: 2026-08-29

Independently reran every applicable gate rather than trusting the Worklog's claim:
`npm run typecheck`, `npm run lint`, `npm run build`, `npm test -- --run` (55 files / 132
tests, matches), scoped `npx prettier --check` on the touched files, and `git diff --check`
— all clean. Cross-checked all 29 `invoke()` call sites in `src/services/*.ts` against
`lib.rs`'s `invoke_handler!` list and each command's `#[tauri::command(rename, rename_all)]`
attributes in `src-tauri/src/commands/*.rs` — names and camelCase param keys match exactly,
including the `Option<String>` case (`getMarketplaceModules`'s `forWorkspaceId`). Did not
re-run `cargo`/`tauri build` since no `src-tauri/` file changed in this task's diff.

- [x] Correctness — pass. `mockBackend.ts`'s `handleMockInvoke` reproduces every command's
      original mock-data behavior faithfully, including `setModuleVisibility`'s two-part
      visibility/enabled-list update and `installModule`/`setModuleEnabled`'s dedup-via-`Set`
      logic. The `Session | null` → `undefined` edge case flagged as a Follow-up in 038 has
      its own direct regression test (`sessionService.test.ts`), not just incidental
      happy-path coverage — this is exactly the kind of edge case the checklist asks about.
- [x] Architecture conformance — pass. Services remain the only `invoke()` boundary; no
      hook, page, component, or shared type changed. Async/`Promise` signatures preserved
      exactly. `ARCHITECTURE.md` §1, §5, and §6 updated in this same task to describe the
      real data flow (`SQLite → #[tauri::command] → services/*` plus the separate
      `mockData/*.ts → test/mockBackend.ts → mocked Tauri IPC` test path) — satisfies
      `.ai/quality-gates.md`'s "structural changes" gate, not deferred to a follow-up.
- [x] Process — pass, with a non-blocking note: the task's `## Plan` section still lists
      only 5 of the 7 files actually touched (`materialService.ts` and `noteService.ts`
      are missing from the list, present in the diff and the Worklog). This is the same
      situation 038 called out in its own first Worklog entry (`src/services/` grew past
      the original file list) and isn't scope creep — the `## Scope` section already says
      `src/services/*`, and the Worklog explains the seven-domain count. Not worth a
      changes-requested cycle over, but future tasks touching this directory should just
      update `## Plan` to match once the real file list is known, rather than leaving it
      stale a second time.

Verdict: approved. No blocking findings.

## Follow-ups

- **Resolved 2026-08-29 (claude-code):** the seed-data contract question below is split into
  `041-sample-workspace-seed-import.md` (`proposed`) rather than blocking 039 — see the
  matching Worklog entry above for the reasoning (`concepts.notes_count` is trigger-derived,
  not an independently-seeded field, so no fixture fabrication is needed).

- ~~**Contract decision required before runtime sample import:** `mockConcepts` declares
  nonzero `notesCount` values for many concepts, but `mockNotes` contains only one actual
  Note row. 037's approved schema and Worklog require seeded Note rows to agree with each
  concept's denormalized count. Importing the fixtures as-is would make the source of truth
  internally inconsistent; generating placeholder note text would fabricate learner data.
  No separate task owns seeding, so 039 remains `in-progress` until Claude either supplies
  real Note fixtures, changes the aggregate contract, or explicitly scopes the runtime
  sample import to data whose contracts are internally complete.~~
