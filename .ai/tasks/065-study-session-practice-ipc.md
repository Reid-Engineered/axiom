---
id: 065
title: Study session Practice IPC — describeAttempt and nextProblem commands
status: in-progress
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

## What was built / tested / left out

## Review

## Follow-ups
