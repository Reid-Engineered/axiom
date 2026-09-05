---
id: 058
title: Practice Tauri command + frontend service wiring
status: in-progress
owner: codex
stage: 8
depends_on: [57]
---

## Scope

Wire `practice.generate@1`/`practice.evaluate@1`/`practice.hint@1` — real capabilities on
the module-capability runtime since task 057, invoked so far only from test fixtures —
through to something the frontend can call: real app-startup construction of the
`ModuleRegistry` against the bundled `knowledge-package/`, `#[tauri::command]` handlers
translating to/from `practice::types`' snake_case contract, and matching
`src/services/practiceService.ts` + `src/test/mockBackend.ts` wiring. Does not build: Study
Session UI, per-workspace module enable/disable wired into capability resolution — see
`docs/superpowers/specs/2026-09-04-practice-tauri-command-wiring-design.md` §1/§8.

## Plan

- `src-tauri/src/practice/mod.rs` (export `PracticeStore`)
- `src-tauri/tauri.conf.json` (bundle `knowledge-package/` as a resource)
- `src-tauri/src/commands/practice.rs` (new), `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`
- `src/types/practice.ts` (new), `src/types/index.ts`
- `src/services/practiceService.ts` (new), `src/services/practiceService.test.ts` (new)
- `src/test/mockBackend.ts`

See `docs/superpowers/plans/2026-09-04-practice-tauri-wiring.md` for the task-by-task plan.

## Worklog

- 2026-09-05 — Task 1 started, claimed by codex. Fresh branch from origin/master 6e32b0f; copied only the approved spec and plan. No previous wiring carried over. Execution is sequential per user instruction; task file is the durable progress record.

- 2026-09-05 — Preflight compatibility findings confirmed against installed Tauri 2.11.5 / tauri-macros 2.6.3: command `rename_all` affects arguments, not command names (existing `commands/note.rs` uses explicit `rename`); Task 3 will add the matching camelCase command names. The approved frontend sends `symbolic-expression`, so the response-value enum must use kebab-case variants with a camelCase `responseType` tag. Tauri array resource paths preserve `..` as `_up_`, so Task 4 will use an explicit resource destination map to retain the planned `knowledge-package` runtime directory. These are contract compatibility corrections; capability types remain untouched.
- 2026-09-05 — Environment: Windows has no Cargo; using the existing WSL Rust toolchain against this checkout. Frontend baseline could not start before installing the lockfile dependencies (`vitest` absent). Native WebKitWebDriver is not on PATH or in the searched system directories; tauri-driver and xvfb-run are installed.
- 2026-09-05 — Task 1 complete: exported PracticeStore; WSL cargo check passed. Task file owner corrected from the plan's claude-code placeholder to codex per repo roles.

## What was built / tested / left out

(filled in at the final task)

## Review

(filled in by reviewer)

## Follow-ups

(filled in if anything is noticed during implementation/review)
