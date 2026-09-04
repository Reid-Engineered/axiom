---
id: 057
title: Practice Core Utility
status: in-progress
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

## What was built / tested / left out

(filled in at Task 9)

## Review

(filled in by reviewer)

## Follow-ups

(filled in if anything is noticed during implementation/review)
