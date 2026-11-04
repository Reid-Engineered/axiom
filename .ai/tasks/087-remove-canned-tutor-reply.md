---
id: 087
title: Remove canned tutor reply and surface unavailable state
status: review
owner: antigravity
stage: 8
depends_on: []
---

## Scope

Split out from item 1 of `.ai/tasks/077-application-wide-affordance-purge.md` (§5.1 / §7.2 of
`docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`).

`add_tutor_exchange_handler` (`src-tauri/src/commands/session.rs`) previously inserted a row into
`tutor_exchanges` with the hardcoded answer
`"Start with what the current representation makes visible, then test one step against the goal."`,
indistinguishable in SQLite from a real tutor reply. Because no real tutor capability exists yet
(`ROADMAP.md` "Beyond the migration"), Axiom must never persist a fabricated tutor reply and must
instead return a structured unavailable status and render an honest unavailable state in the tutor
pane's reply area.

Also includes a database migration (`0005_remove_canned_tutor_exchanges.sql`) to delete any
existing `tutor_exchanges` rows carrying that canned reply from pre-existing databases.

Does **not** build: a real tutor capability, changes to `CORE.md` or `ARCHITECTURE.md`, design
token changes, Practice generation changes, or items 2–6 of task `077`.

## Plan

- `src-tauri/src/db/migrations/0005_remove_canned_tutor_exchanges.sql` — delete existing rows from
  `tutor_exchanges` whose `answer` matches the fabricated canned sentence.
  - **Migration rationale:** §2 of `2026-09-09-mock-and-dead-end-containment-design.md` establishes
    the invariant that the rendered app must never present as the learner's own history anything
    the learner did not do, and §5.1 identifies the canned tutor answer as the only defect in the
    inventory that writes fiction into the learner's own record. Stopping future writes alone leaves
    previously persisted canned rows in `tutor_exchanges`, which `load_session` would continue to
    load and display as real exchanges on session resume. Deleting rows matching the exact canned
    answer string removes the fabricated history cleanly while preserving the table schema and any
    non-canned rows.
- `src-tauri/src/db/schema.rs` & `src-tauri/src/db/tests.rs` — register migration 5, bump
  `LATEST_SCHEMA_VERSION` to 5, and test that migrating from v4 removes canned replies while
  preserving sessions and non-canned rows.
- `src-tauri/src/commands/models.rs` & `src-tauri/src/commands/session.rs` — add optional
  `tutor_status: Option<TutorStatus>` (`Unavailable`) to `Session`; update
  `add_tutor_exchange_handler` to validate the session is mutable, write zero rows to
  `tutor_exchanges`, and return the session with `tutor_status: Some(TutorStatus::Unavailable)`.
- `src-tauri/src/commands/tests.rs` — assert `add_tutor_exchange_handler` returns
  `TutorStatus::Unavailable`, persists zero rows in `tutor_exchanges`, and still rejects missing or
  completed sessions.
- `src/types/session.ts` — add optional `tutorStatus?: 'unavailable'` to `Session` (preserving the
  `addTutorExchange(sessionId, question): Promise<Session>` service signature in
  `src/services/sessionService.ts`).
- `src/hooks/useSessions.ts` & `src/hooks/useSessions.test.tsx` — treat `tutorStatus` as a
  response-only status; lifecycle updates replace it rather than copying it forward.
- `src/test/mockBackend.ts` — mirror the real handler in `addTutorExchange` (validate mutable
  session, store no exchange, return `tutorStatus: 'unavailable'`).
- `src/pages/StudySessionPage.tsx` & `src/pages/StudySessionPage.test.tsx` — render the honest
  unavailable message beside existing exchanges in a persistent live region. Keep the notice
  in ephemeral page state scoped to the session, preserve the typed question, and test that
  asking displays the notice without hiding history or incrementing the exchange count.

## Worklog

- 2026-11-04 ? Codex fixed the independent review findings and integrated current master.
  Typecheck, lint, build and all 210 frontend tests (62 files) passed, as did GNU rustfmt
  and diff checks. GNU cargo test --locked --lib compiled, but Windows could not start the
  executable (0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND); no Rust tests ran locally.
  Fresh CI Windows, Linux and macOS backend checks passed; native E2E remains pending.

- 2026-11-03 — Split item 1 out of task `077` into `087`, claimed by `antigravity` (`in-progress`).
- 2026-11-03 — Added migration `0005_remove_canned_tutor_exchanges.sql` (`LATEST_SCHEMA_VERSION = 5`),
  updated `add_tutor_exchange_handler` and `mockBackend.ts` to store no reply and return
  `tutorStatus: 'unavailable'` on `Session`, preserved `tutorStatus` across `useSession` updates,
  and updated `TutorPane` in `StudySessionPage.tsx` to render the unavailable state where the reply
  was shown.
- 2026-11-03 — Added Rust unit tests (`remove_canned_tutor_exchanges_migration_purges_fabricated_replies`,
  `add_tutor_exchange_reports_unavailable_without_persisting_a_reply`) and frontend tests in
  `StudySessionPage.test.tsx` and `useSessions.test.tsx`. All local frontend and Rust gates green;
  moving to `review`.

## What was built / tested / left out

- **Built:**
  - `src-tauri/src/db/migrations/0005_remove_canned_tutor_exchanges.sql` and
    `src-tauri/src/db/schema.rs`: migration v5 (`remove_canned_tutor_exchanges`) deleting any rows
    from `tutor_exchanges` matching the fabricated canned reply.
  - `src-tauri/src/commands/models.rs` and `src-tauri/src/commands/session.rs`: added
    `TutorStatus::Unavailable` and optional `tutor_status` on `Session`; `add_tutor_exchange_handler`
    now validates `ensure_mutable_session`, writes zero rows to `tutor_exchanges`, and returns the
    session with `tutor_status: Some(TutorStatus::Unavailable)`.
  - `src/types/session.ts`: added optional `tutorStatus?: 'unavailable'` to `Session` (all existing
    consumers and `sessionService.ts` signatures typecheck unchanged).
  - `src/hooks/useSessions.ts`: lifecycle mutations replace the session without carrying a
    previous tutor response status forward.
  - `src/test/mockBackend.ts`: updated `addTutorExchange` to return `tutorStatus: 'unavailable'`
    without pushing to `session.exchanges`.
  - `src/pages/StudySessionPage.tsx`: renders `"Tutor replies are not available in this build yet."`
    in a persistent live region alongside existing exchanges; the notice is ephemeral page state,
    scoped to the session, and unavailable submissions retain the typed question.
- **Tested:**
  - `npm run typecheck` — passed (0 errors).
  - `npm run lint` — passed (0 errors).
  - `npm run build` — passed.
  - `npx vitest run` — 61 test files, 166 tests passed (including new/updated tests in
    `src/pages/StudySessionPage.test.tsx` and `src/hooks/useSessions.test.tsx`).
  - Design-token grep over `src/` diff — 0 matches.
  - `cargo fmt --check` (WSL) — passed.
  - `cargo check` (WSL) — passed.
  - `cargo clippy --all-targets -- -D warnings` (WSL) — passed.
  - `cargo test` (WSL) — 320 passed, 0 failed (including
    `db::tests::remove_canned_tutor_exchanges_migration_purges_fabricated_replies` and
    `commands::tests::add_tutor_exchange_reports_unavailable_without_persisting_a_reply`).
  - Manual gates: `ARCHITECTURE.md` unchanged (no folder structure, top-level directory, or data-flow
    rule changed); shared type addition in `src/types/session.ts` is optional (`tutorStatus?: 'unavailable'`)
    and verified across all consumers via `npm run typecheck`; no component in `src/components/`
    imports from `src/services/`.
- **Left out:**
  - Real tutor integration (`ROADMAP.md` "Beyond the migration").
  - Items 2–6 of `.ai/tasks/077-application-wide-affordance-purge.md`.

## Review

Reviewer: Claude (independent review, 2026-11-04)
Reviewed head: `3f707c3e7504ac22e3deb380918bb5bce7a7e742`

Verdict: changes-requested

- Correctness: the unavailable branch hid the latest real exchange; lifecycle updates
  copied the response-only status into later session payloads.
- Coverage: migration test omitted assertions for session preservation and migration v5.
- Process: required CI predates merged 082/083; integrate current master and rerun gates.
- Architecture, tokens, scope, and remaining implementation checks passed.

Codex remediation (2026-11-04): integrated `origin/master`; removed status carry-forward;
kept availability as ephemeral page state and displayed it in an always-present live
region alongside current history; retained typed questions; added assertions for latest
exchange visibility, response status replacement, session preservation and migration v5.
Fresh validation and final independent review remain pending.

Remediation validation: `npm run typecheck`, `npm run lint`, `npm run build` and
`npm run test` passed (62 files, 210 tests), as did `git diff --check` and
`cargo +stable-x86_64-pc-windows-gnu fmt --all --check`. The GNU `cargo test --locked --lib`
build was still running when this update was recorded; fresh CI remains required,
including native Linux E2E.

Final independent review: Claude, 2026-11-04, head
`fee035275409aab657d17c5d8d3a601035853c7d`. All prior blocking code findings resolved.
Correctness, coverage, architecture and UI rules pass; current master is integrated.
Verdict: approve once all seven required CI checks pass. No merge pass is recorded
while gates remain pending. This subsequent commit records review and validation only.

## Follow-ups

- Task 077: disclose unavailable tutoring before learners type a question. This task keeps
  the question intact after submitting but does not redesign that affordance.
