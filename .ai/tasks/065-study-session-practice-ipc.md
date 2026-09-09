---
id: 065
title: Study session Practice IPC — describeAttempt and nextProblem commands
status: review
owner: codex
stage: 8
depends_on: [063]
---

## Scope

Tasks 1 and 2 of `docs/superpowers/plans/2026-09-09-study-session-ui-integration.md`: the two
Tauri commands the Study Session UI needs. `describeAttempt` exposes task 063's existing
`practice.describe` capability, which today has no command wrapper and is therefore
unreachable from the frontend. `nextProblem` rebinds a session to a freshly generated attempt
and advances its problem counter, reusing the `start_practice_attempt` helper task 063 added
to `session.rs`.

Does **not** build: anything under `src/` — the frontend types, services, hook, and page
wiring are task `066`, owned by antigravity and running in parallel against
`src/test/mockBackend.ts`. Does not change any capability request/response shape, anything
under `src-tauri/src/generation/` or `src-tauri/src/knowledge/`, or `ROADMAP.md` (task `064`'s
Task 6).

## Plan

Follow the plan document's Task 1 and Task 2 exactly; it carries the full test code and
implementation for both. Files:

- `src-tauri/src/commands/practice.rs` — `DescribeAttemptInput`, `AttemptDescription`,
  `From<DescribeResponse>`, `describe_attempt_handler`, `describe_attempt`, plus two tests in
  the existing `mod tests`.
- `src-tauri/src/commands/session.rs` — `next_problem_handler` and `next_problem`.
- `src-tauri/src/commands/tests.rs` — two `nextProblem` orchestration tests.
- `src-tauri/src/lib.rs` — register both commands in `invoke_handler`.

Design source: `docs/superpowers/specs/2026-09-09-study-session-ui-integration-design.md` §3.

## Worklog

- 2026-09-09 — Filed by claude from the approved plan, split from task `066` so the Rust and
  frontend halves have unambiguous single owners per `.ai/lifecycle.md`. Assigned to codex as
  a direct continuation of its own task 063.
- 2026-09-09 — Claimed by codex on `agent/codex/065-study-session-practice-ipc`, branched
  directly from current `master`.
- 2026-09-09 — Task 1 TDD: added the two `describe_attempt` tests, observed the expected
  missing input/handler compile failure, implemented the IPC wrapper, and passed all 299
  Rust tests plus formatting and Clippy with warnings denied.
- 2026-09-09 — Task 2 TDD: added the two `next_problem` tests, observed the expected missing
  handler compile failure, implemented the generate-first session update, and passed all 301
  Rust tests plus formatting and Clippy with warnings denied. Tests ran in the existing WSL
  toolchain because the Windows GNU linker cannot produce this repository's `cdylib` test
  target (`export ordinal too large`).
- 2026-09-09 — After tasks 066 and 062 merged, rebased the branch onto current `master`
  (`38e15cb`) and checked the now-real frontend contract in `practiceService.ts`,
  `sessionService.ts`, the shared types, and `mockBackend.ts`. The command names, argument
  nesting, camelCase response fields, and returned `Session` match without frontend changes.
- 2026-09-09 — Re-ran the final backend gates on the rebased source: 301 Rust tests,
  `cargo check --locked`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets
  --locked -- -D warnings` all pass. Implementation complete; moved to `review`.

## What was built / tested / left out

Built the two additive Tauri commands required by the merged Study Session frontend:

- `describeAttempt` accepts `{ input: { workspaceId, attemptId } }`, invokes the existing
  `practice.describe` capability, and returns exactly `prompt`, `responseType`, `hintsTotal`,
  `hintsRevealed`, `status`, and `submissionCount`. Two command-layer tests cover the current
  state of a generated attempt and an unknown attempt id.
- `nextProblem` accepts `{ sessionId }`, loads the session's Knowledge Package concept
  crosswalk, reuses task 063's unchanged `start_practice_attempt` helper, and only writes the
  new attempt id plus incremented counter after generation succeeds. Two orchestration tests
  cover successful rebinding and the inherited degrade-to-NULL/no-counter-advance path.
- Both commands are registered in `src-tauri/src/lib.rs`; nothing under `src/`,
  `src-tauri/src/generation/`, `src-tauri/src/knowledge/`, or `ROADMAP.md` changed.

Plan deviation recorded explicitly: Task 1's sample conversion uses
`response.response_type.into()`, but the current command DTO and `DescribeResponse` both use
the same `crate::knowledge::ResponseType`. Clippy rejects that identity conversion as
`useless_conversion`, so the implementation assigns `response.response_type` directly. Its
serialized `responseType` remains the exact union the merged frontend expects.

Tested on rebased commit base `38e15cb` with the WSL Rust toolchain:

- `cargo test --manifest-path src-tauri/Cargo.toml --locked` — 301 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml --locked` — passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` — passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`
  — passed.

Deliberately left out: the merged frontend, roadmap amendments, capability contracts,
generation engine, and Knowledge Package. The native full-loop manual check is deferred
until this PR lands, as requested. The automated gap to check manually is the boundary the
unit suites do not cross: a real WebView invoking these registered commands against native
SQLite, then submitting an answer and advancing to a newly bound attempt. The existing
frontend tests use `mockBackend.ts`, while the Rust tests call handlers directly.

## Review

## Follow-ups

- After merge, manually exercise launch → mapped Shell-method session → prompt → answer →
  solved state → next problem, and record any native IPC or persistence behavior that the
  separate mock-frontend and Rust-handler suites did not expose.
