---
id: 057
title: Practice Core Utility
status: done
owner: claude-code
stage: 8
depends_on: [45, 46, 47, 48, 54, 55, 56]
---

## Scope

Add a first-party Practice module (`org.axiom.practice`) providing `practice.generate@1`,
`practice.evaluate@1`, `practice.hint@1` on the module-capability runtime, backed by real
SQLite persistence (`practice_attempts`, `practice_submissions`). Assembles the Knowledge
Package, canonical Problem schema, `math.verify`, and problem generation (tasks 049-056)
into a generate -> attempt -> evaluate flow. Does not build: any Tauri command or frontend
wiring, Study Session UI, adaptive family/difficulty selection, or adaptive hint selection —
see `docs/superpowers/specs/2026-09-04-practice-core-utility-design.md` §1/§9.

## Plan

- `src-tauri/src/db/migrations/0002_practice.sql`, `src-tauri/src/db/schema.rs` (new tables)
- `src-tauri/src/practice/module.toml`, `mod.rs`, `types.rs`, `error.rs`, `store.rs`,
  `provider.rs`, `tests/mod.rs`
- `src-tauri/src/lib.rs` (register `pub mod practice;`)

See `docs/superpowers/plans/2026-09-04-practice-core-utility.md` for the task-by-task
implementation plan.

## Worklog

- 2026-09-04 — started, claimed by claude-code
- 2026-09-04 — Task 2 complete: added migration 0002 and verified the new Practice tables;
  updated the existing schema regression test from one migration/24 tables to the current
  migration count/26 tables. Cargo was unavailable on the Windows `PATH`, so Rust checks
  run through the installed WSL toolchain with a WSL-native target cache.
- 2026-09-04 — Task 3 complete: added Practice request/response contracts and six passing
  serialization tests. The plan's initial red command passed with zero tests because the
  unwired source file was not compiled; declared `pub mod practice;` first to confirm the
  intended missing-module failure, then added the planned `mod.rs` skeleton.
- 2026-09-04 — Task 4 complete: added `PracticeError` and three passing focused tests. As
  in Task 3, the undeclared source file produced zero tests instead of the plan's expected
  red compile failure. The plan's `-p axiom_lib` Clippy selector is also invalid because
  `axiom_lib` is a library target, not a package; used `-p axiom --lib`. Interim dead-code
  warnings remain for private Task 3 types until their planned public exports in Task 6.
- 2026-09-04 — Task 5 complete: added the SQLite-backed attempt/submission store and six
  passing persistence tests, including an on-disk reopen. The plan's workspace fixture
  violated the existing deferred workspace/goal foreign-key cycle by inserting only the
  workspace; fixed it by inserting both rows in one transaction. Removed unused serde
  imports from the plan snippet.
- 2026-09-04 — Task 6 complete: added the Practice manifest and `practice.generate@1` with
  five passing provider tests. Reused the schema-valid workspace/goal fixture and removed
  the plan snippet's unused `ProblemFamilyId` import. Deferred `#[cfg(test)] mod tests;`
  until Task 9 because Rust cannot declare the planned test module before its file exists.
- 2026-09-04 — Task 7 complete: added `practice.evaluate@1`; nine focused provider tests
  and all 265 crate tests pass. `tauri::async_runtime::RwLock::blocking_write` is available.
  The first typed inter-capability call exposed that existing `VerifyRequest` was
  deserialize-only and `VerifyResult` serialize-only; added the inverse serde derives so
  both satisfy `ModuleRegistry::invoke`'s typed input/output bounds.
- 2026-09-04 — Task 8 complete: added `practice.hint@1`; all 13 provider tests, all 269
  crate tests, and strict Clippy pass. The plan declared `hint` as synchronous but wrapped
  its direct test calls in `block_on`; kept the specified synchronous API and removed those
  invalid wrappers.
- 2026-09-04 — Task 9 complete: added two full-registry tests and moved the task to review.
  Adapted the workspace fixture to seed its required guiding goal transactionally and
  declared the test module alongside its file (an undeclared Rust source file is ignored,
  not a compile failure). The first formatting check found drift; ran `cargo fmt`, then the
  full gate passed with 271 tests. Used `cargo clippy -p axiom --lib -- -D warnings`, the
  valid equivalent of the plan's invalid package selector `-p axiom_lib`.

## What was built / tested / left out

Built: `src-tauri/src/practice/` (types, error, store, provider, module.toml), a new
migration (`practice_attempts`, `practice_submissions`), and `pub mod practice;` in
`lib.rs`. `practice.generate@1`, `practice.evaluate@1`, `practice.hint@1` registered on
the module-capability runtime; `practice.evaluate` resolves and invokes `math.verify`
through `Arc<RwLock<ModuleRegistry>>` rather than calling `math_verify`'s Rust code
directly (spec §4) -- the first real inter-capability call the runtime has carried.

Tested: `cargo test` across `practice::types`, `practice::error`, `practice::store`,
`practice::provider`, `practice::tests` (round-trip through a real registry with both
`math_verify` and `practice` registered) -- generation-matches-the-engine, hidden
canonical-solution/hints in every outward response, workspace isolation on all three
capabilities, multi-submission-until-solved, `AlreadySolved`/`NoMoreHints` edge cases,
attempt persistence surviving a fresh connection to the same on-disk database. Gates run:
`cargo check`, `cargo test`, `cargo clippy -p axiom --lib -- -D warnings`,
`cargo fmt --check` (all `src-tauri/` changes; no `src/` changes in this task, so the
npm-side gates in `.ai/quality-gates.md` don't apply).

Left out (per spec §1/§9, by design): any `#[tauri::command]` or frontend service
wiring, Study Session UI, adaptive family/difficulty selection, adaptive hint selection,
the network-disabled offline acceptance test (depends on Study Session UI existing first).

## Review

Reviewer: claude-code
Date: 2026-09-04

- [x] Correctness — pass. `provider.rs`, `store.rs`, `types.rs`, `error.rs` match the plan's
      contracts and the design spec §4-§8. `practice.evaluate` resolves and invokes
      `math.verify` through `registry.resolve`/`registry.invoke` (`provider.rs:139-165`),
      never calling `MathVerifyProvider`'s Rust code directly, per spec §4. `canonical_solution`
      and unrevealed hint text never leave `GenerateResponse`/`HintResponse`
      (`types.rs:29-33,58-62`, verified by both the serde-structural tests and
      `generate_response_never_exposes_the_canonical_solution`/`hint_response_never_exposes_the_full_hint_list`).
      Workspace isolation is enforced by `load_attempt`'s `WHERE id = ?1 AND workspace_id =
      ?2` (`store.rs:60-64`) on every one of the three capability handlers. Edge cases
      (`AlreadySolved`, `NoMoreHints`, `ResponseTypeMismatch`, wrong-workspace `AttemptNotFound`)
      are each covered by a dedicated test, not just the happy path.
- [x] Independent re-verification — pass. Re-ran the full gate myself from a clean checkout
      rather than trusting the task file: `cargo test` → 271 passed, 0 failed;
      `cargo clippy -p axiom --lib -- -D warnings` → clean; `cargo fmt --check` → clean.
      Matches the worklog's claims exactly.
- [x] Inter-capability lock correctness — pass, and worth calling out since it's the one
      genuinely new mechanism this task adds to the runtime. The plan originally specified
      `std::sync::RwLock`, which would not compile here (`evaluate()` holds the guard across
      an inner `.await`, and `std::sync::RwLockReadGuard` isn't `Send`, which `async_trait`'s
      default `Send`-future bound on `CapabilityProvider::invoke` requires). The plan was
      corrected before handoff to `tauri::async_runtime::RwLock`; the shipped code
      (`provider.rs:4,29,144,160`) uses it correctly, with `.read().await` inside the async
      path and `.blocking_write()`/`.blocking_read()` confined to sync test helpers
      (`provider.rs:329`, `tests/mod.rs`). Confirmed this compiles and passes in Rust as
      written, not just in the plan's pseudocode.
- [ ] Architecture conformance (`ARCHITECTURE.md`) — N/A. That document governs the
      frontend (`src/`); this task is entirely `src-tauri/`. The equivalent backend
      contract is `CORE.md`, which this task conforms to (module registers via
      `module.toml` + `CapabilityProvider`, no first-party shortcut around the capability
      boundary — see the inter-capability point above).
- [ ] UI rules (`AGENTS.md`) — N/A, no `src/` or UI changes in this task.
- [x] Process — pass. Worklog is dated and specific, not batched (per-task entries with
      honest deviation notes: the FK-cycle fixture fix in Task 5, the `axiom_lib` →
      `-p axiom --lib` Clippy selector correction, the sync/async `hint()` cleanup in Task
      8). Scope matches the task's own stated scope — no Tauri command, frontend, or UI
      code snuck in. `math_verify/types.rs`'s two added serde derives
      (`Serialize` on `VerifyRequest`, `Deserialize` on `VerifyResult`) are the one file
      touched outside `practice/` — legitimate and minimal: `ModuleRegistry::invoke`'s
      generic bounds (`Input: Serialize`, `Output: DeserializeOwned`) require both, and
      this is the first real caller to need them: Practice is the first module to invoke
      `math.verify` as a typed capability rather than raw JSON. No `ARCHITECTURE.md` update
      needed (no frontend structural change).

Verdict: approved, no blocking findings. Merged to `master` (fast-forward from
`agent/codex/057-practice-core-utility`) per the user's explicit instruction; frontmatter
`status` set to `done` and this file archived, per `.ai/lifecycle.md`.

## Follow-ups

None identified during review. Next Stage 8 sub-projects (per `ROADMAP.md`): Tauri command
+ frontend service wiring for `practice.*`, Study Session UI integration, adaptive
family/difficulty selection, and the network-disabled offline acceptance test — none
required or attempted here, per spec §1/§9.
