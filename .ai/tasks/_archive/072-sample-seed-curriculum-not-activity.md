---
id: 072
title: Sample seed carries curriculum, not activity
status: done
owner: codex
stage: 8
depends_on: []
---

## Scope

Implement §6.2 of `docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.

The sample import currently writes four sessions, 40 tutor exchanges, settled conclusions, a
`progress` of `0.58` and a `lastActivityAt` into SQLite — activity records for things no
learner did. The seeded session is what Home's "Resume session" resolves to, and it has no
bound attempt, which is the dead end the Windows walk hit.

Removes seeded *activity* from the sample. Curriculum stays: workspaces, concepts, goals,
modules, workspace templates, material, and the crosswalk that makes Shell method practiceable.

Does **not** build: the selection fix (`071`), the boot fix (`070`), or the wider seed hygiene
— mastery states, diagnostics, notes, heuristics and activity events are also fabricated
history but cause neither the dead end nor a boot misselection, and belong to `077`.

## Plan

- `src-tauri/src/commands/models.rs` — drop `sessions` from `SampleWorkspaceSeed`.
- `src-tauri/src/commands/seed.rs` — drop `insert_sessions` and its `tutor_exchanges` /
  `session_settled_conclusions` writes from `import_seed`.
- `src/services/sampleWorkspaceService.ts` — stop sending `sessions` in the seed payload.
- `src/services/mockData/workspaces.ts` — `progress: 0` and no `lastActivityAt` on all three
  sample workspaces. `070`'s boot ordering reads `last_activity_at`, so a seeded value would
  fabricate an active workspace.
- `src/test/mockBackend.ts` — **in this task, not a follow-up.** Line 83 seeds the double from
  `mockSessions`, which is exactly the divergence that let the frontend suite stay green
  through this defect (design §4.3). A double richer than the backend preserves the blind spot.
- `src-tauri/src/commands/tests.rs`, `src/pages/HomePage.test.tsx` — tests below.

`src/services/mockData/sessions.ts` is **not** deleted. `mockBackend` and individual tests may
still construct sessions; the file simply stops being shipped as seed.

**Tests.**

- After sample import, `getActiveSessionByWorkspace` returns `None` for all three sample
  workspaces.
- After sample import, zero rows exist in `sessions`, `tutor_exchanges` and
  `session_settled_conclusions`.
- After sample import, the sample workspace has `progress = 0` and `last_activity_at IS NULL`.
- The Shell method concept still resolves `shell.method_vertical_axis`; the existing crosswalk
  guard still passes.
- Frontend: Home renders no Continue card immediately after import.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, `proposed` for codex.
- 2026-09-09 — Claimed by codex on `agent/codex/072-sample-seed-curriculum-not-activity`; beginning TDD removal of seeded activity while retaining sample curriculum.
- 2026-09-09 — Added acceptance assertions before implementation. Initial execution was blocked by the worktree's missing Node dependencies and unusable native Rust linkers; after installing the locked dependencies and switching Rust gates to WSL, the targeted tests passed against the minimal implementation.
- 2026-09-09 — Removed sessions from the Rust and TypeScript seed contracts and made the test IPC baseline session-free. Tests that genuinely exercise prior learner activity now load session fixtures explicitly rather than inheriting fabricated seed state.
- 2026-09-09 — Local gates green; moved to `review`. PR creation and the required CI/e2e result are pending the coordinating agent.
- 2026-09-09 — Gates independently re-run by claude on a Linux toolchain rather than taken
  from the report: `cargo test` 305 passed / 0 failed, `cargo clippy --all-targets -- -D
  warnings` clean, `cargo fmt --check` clean, `npm run typecheck`, `npm run lint`, `npm run
  build`, `npx vitest run` (61 files / 165 tests), design-token grep empty. Every number
  codex recorded above matched. claude made no code change to this branch.

## What was built / tested / left out

- Removed `sessions` from `SampleWorkspaceSeed`, the frontend import payload, and the Rust
  import transaction. The importer can no longer write seeded sessions, tutor exchanges, or
  settled conclusions.
- Set all three retained sample workspaces to `progress: 0` with no `lastActivityAt` and made
  `resetMockBackend` start with no sessions. `mockData/sessions.ts` remains available only for
  tests that explicitly load learner activity.
- Expanded the Rust sample-import test fixture to all three workspaces and asserted zero
  progress, null activity, no active session for each, zero rows in all three session tables,
  and the retained Shell-method crosswalk/curriculum coverage.
- Updated direct consumer tests. `sampleWorkspaceService.test.ts` was not named in the Plan,
  but changing it was required because it verifies the complete seed payload contract.
- Tested locally: `npm run typecheck`; `npm run lint`; `npm run build`; `npm test` (61 files,
  165 tests); `cargo check`; `cargo test` (305 tests); `cargo clippy --all-targets -- -D
  warnings`; `cargo fmt --check`; design-value grep over the `src/` diff (no matches).
- Native Rust gates ran under WSL with a task-scoped target directory because the Windows MSVC
  linker was unavailable and the GNU linker cannot link this Tauri cdylib. The required CI
  e2e job was not run locally and remains pending on the PR.
- Left out as scoped: active-session ordering/reuse, startup restoration, study-screen cleanup,
  workspace provisioning, native restart coverage, and the wider fixture/affordance purge.

## Review

Reviewer: claude
Date: 2026-09-10

Gates re-run independently on a Linux toolchain rather than taken from the PR body; every
figure codex recorded matched exactly, including `cargo test` at 305.

- [x] **Correctness — pass.** The Rust acceptance test asserts the real thing rather than a
      proxy: after import, three workspaces at `progress = 0` with `last_activity_at` NULL,
      `get_active_session_by_workspace` returning `None` for each, and zero rows across all
      three of `sessions`, `tutor_exchanges` and `session_settled_conclusions`. The Shell
      method crosswalk still resolves, so removing activity did not take curriculum with it.
- [x] **Architecture conformance — pass.** `sessions` removed from the `SampleWorkspaceSeed`
      contract on both sides of the IPC boundary, so the shape cannot drift. No hook, service
      signature, or global state changed.
- [x] **UI rules — pass.** No component or CSS touched. Copy in the rewritten tests follows the
      handoff rules.
- [x] **Process — pass.** All seven checks green on the merged head. `sampleWorkspaceService.test.ts`
      was outside the Plan's file list and is disclosed in the task file rather than folded in
      silently — correctly, since it verifies the seed payload contract this task changes.

### What makes this one right

`resetMockBackend` now starts session-free and `loadMockSessionsForTest()` is an explicit
opt-in. That is the correct shape: the double's *baseline* no longer carries fabricated
activity, which is precisely the divergence that let a green frontend suite coexist with a
dead end on Home (design §4.3). Nine tests were adjusted rather than deleted, and the key one
is inverted rather than dropped — "renders no Continue card immediately after sample import"
asserts absence, and the others create real activity through `startSession`.

### Observations (none blocking)

1. **The long-absence test's `mockIPC` override becomes redundant once `071` lands.** It injects
   `lastActivityAt` onto a workspace because this task removed the seeded value; `071` makes
   `startSession` write that column itself. Worth removing next time the file is open.
2. **`workspaceActivity`, notes, diagnostics, heuristics and mastery states are still seeded.**
   Correct for this task's scope — none causes the dead end or is read by the boot ordering —
   but they remain fabricated history and are `077`'s to remove.

Verdict: approve

## Follow-ups
