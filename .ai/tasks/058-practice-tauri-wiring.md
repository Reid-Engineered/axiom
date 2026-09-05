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

- 2026-09-05 — Task 2 started: registry tests and todo body added before implementation.

- 2026-09-05 — Task 2 red confirmed: both registry tests panic at todo!(); implemented the planned helper. Frontend baseline passed: 58 files / 144 tests, including GoalEditingSheet.

- 2026-09-05 — Task 2 complete: both registry tests pass; blocking_write works on the installed Tauri runtime.

- 2026-09-05 — Task 3 started: added the plan's three command tests before command implementation to confirm an actual red result.

- 2026-09-05 — Task 3 red confirmed: 12 compiler errors for the absent command types/handlers. Added the plan's full command implementation next; checking it against the actual capability types.

- 2026-09-05 — Task 3 compile finding: all three capability request types lack Serialize and all three responses lack Deserialize (E0277). Added only those reciprocal derives in practice/types.rs; field names and snake_case wire contract are unchanged. This matches task 057's earlier math_verify derive fix. Cargo cache moved to /var/tmp/axiom-058-target after confirming Windows-mounted cache timestamps caused unchanged dependencies to rebuild.

- 2026-09-05 — Content finding: the real knowledge-package has concepts/objectives/examples but zero canonical problem families (confirmed by knowledge/tests/migration.rs and package contents). Startup loads this real package as approved; successful generation is covered by the canonical test fixture. Authoring production families remains a content follow-up, not a test-fixture fallback or a scope expansion.

- 2026-09-05 — Task 3 compatibility regressions confirmed red: actual generated Tauri command name was generate_attempt, and symbolic-expression JSON was rejected in favor of symbolicExpression. Added explicit command rename attributes and kebab-case response variants, preserving camelCase field names. Tests also cover numeric translation and nonempty missing-provider errors.

- 2026-09-05 — Task 3 complete: all 8 command tests and 279 crate tests pass; cargo clippy --lib -- -D warnings and cargo fmt --check pass. Async Tauri State wrappers compile without workaround. No new dependency was needed.

- 2026-09-05 — Task 4 started: wired startup/managed state and registered commands; explicit resource destination map corrects Tauri's _up_ mapping. Updating ARCHITECTURE.md for the new capability-to-IPC data path as required by the structural gate.

- 2026-09-05 — Task 4 complete: cargo check/test (279)/clippy --lib -- -D warnings/fmt --check all pass. Verified the build-generated debug/knowledge-package tree exists and diff -qr matches the real source package. Native launch will be checked by the final E2E gate. ARCHITECTURE.md now records the startup, storage, and capability-to-IPC path.

- 2026-09-05 — Task 5 started: added the approved frontend Practice types and barrel export.

- 2026-09-05 — Task 5 complete: npm run typecheck passed; all shared type consumers remain valid.

- 2026-09-05 — Task 6 started: added generateAttempt, evaluateAttempt, and requestHint service functions exactly as planned.

- 2026-09-05 — Task 6 complete: npm run typecheck and npm run lint passed with no warnings.

- 2026-09-05 — Task 7 started: added the exact planned mock family, resettable attempt map, and three IPC cases.

- 2026-09-05 — Task 7 complete: npm run typecheck passed; mock attempt state resets with the existing IPC fixtures.

- 2026-09-05 — Task 8 started: added the plan's five service tests. As Task 8 Step 2 explicitly notes, Tasks 6–7 already implement the behavior, so the initial test run is expected to pass; no artificial failure is introduced.

- 2026-09-05 — Task 8 complete: npm run test -- practiceService passed all 5 tests on both prescribed runs through handleMockInvoke, never a mocked service module. For final native validation, cache now uses /var/tmp/axiom-058/target: Tauri recognizes a literal target directory when resolving unbundled resources. Matching WebKit driver extracted to /tmp/axiom-058-webdriver; no system package changes.

## What was built / tested / left out

(filled in at the final task)

## Review

(filled in by reviewer)

## Follow-ups

Add canonical ProblemFamily content to the real knowledge-package before Study Session generation is exposed; the existing package currently contains only worked examples.
