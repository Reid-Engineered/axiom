---
id: 069
title: Guard against unregistered IPC commands
status: review
owner: claude
stage: 8
depends_on: [065]
---

## Scope

Add a test asserting that every command the frontend `invoke`s is actually declared and
registered in `lib.rs`, so a service can never call into a command that does not exist in a
real build.

Does **not** build: any change to a command, service, or hook; any runtime check; validation
of argument shapes or return types across the boundary (a genuinely useful but much larger
piece of work — see Follow-ups).

## Why

This closes a class of bug that already shipped. Task `066` merged the frontend calling
`describeAttempt` and `nextProblem`; task `065`, which registers them, is still in review. So
`master` carries a UI invoking two commands the backend does not expose, and the problem pane
cannot hydrate in a real build.

All seven required checks stayed green throughout, because the two suites sit on either side
of the boundary and neither crosses it: frontend tests run against `src/test/mockBackend.ts`,
which implements both commands, and the Rust tests never go through IPC. Nothing in CI looks
at the seam itself.

That is not specific to these two commands — any future service function can be merged ahead
of its command the same way, and splitting Rust and frontend halves into parallel tasks (as
`065`/`066` deliberately did, for good reason) makes it *more* likely, not less.

## Plan

- `src/test/commandRegistration.test.ts` (new) — parses `invoke('name')` calls under
  `src/services/`, `#[tauri::command(rename = "...")]` declarations under
  `src-tauri/src/commands/`, and the `generate_handler![...]` list in `lib.rs`, then asserts
  every invoked name is both declared and wired.

Deliberately a static text check rather than anything cleverer: it needs no running app, no
Tauri context, and costs milliseconds, so it can live in the ordinary frontend suite that
already runs on every PR.

Three assertions rather than one, so a failure says which of the two ways a command can be
unreachable actually happened:

1. A sanity assertion that all three sets are non-empty — without it, a refactor to how
   services call `invoke` would make the whole test pass vacuously on an empty set. This is
   the assertion that keeps the other two honest.
2. Every invoked name has a `#[tauri::command]` declaration.
3. Every invoked name's fn appears in `generate_handler!`. This is the case that actually
   bit us — a command can exist and still be unreachable.

## Worklog

- 2026-09-09 — Filed and implemented by claude, at the human's request, after finding the
  `master` breakage while verifying task `064`'s criterion 2.

## What was built / tested / left out

`src/test/commandRegistration.test.ts`, three assertions as above. Failure messages name the
offending command *and* the service file that calls it, so the fix location is immediate.

**Verified in both directions, rather than assumed:**

- Against `master`, it **fails**, reporting exactly `describeAttempt` and `nextProblem` and
  nothing else — the two genuinely missing commands.
- Against `agent/codex/065-study-session-practice-ipc`, where both are registered, it
  **passes**.

A guard only ever observed passing would be worth little; this one was watched failing for
the right reason first.

**Sequencing:** this test is red on `master` on purpose, because the bug it detects is real
and present. It goes green when `065` merges. It should therefore land *after* `065`, not
before.

Left out: cross-boundary validation of argument and return *shapes*. A command can be
registered and still disagree with its caller about payload structure — `065`'s review had to
check the `describeAttempt` DTO by hand, field by field. Worth doing, much larger, and filed
below rather than folded in.

## Review

## Follow-ups

- **Cross-boundary payload validation.** This guard proves a command is reachable, not that
  the caller and handler agree on its arguments or response shape. Today that agreement is
  verified only by a human reading both sides. Generating TS types from the Rust command
  signatures (e.g. `ts-rs`, `specta`) would make the compiler enforce it — a real design
  decision with a dependency attached, so it needs its own brainstorm rather than a drive-by.
- **`mockBackend.ts` drift.** The mock implements commands independently of the real backend,
  which is what let this breakage stay invisible. Nothing checks that the mock's command list
  matches the registered one, or that its behaviour tracks the handlers'. Same class of gap,
  one layer over.
