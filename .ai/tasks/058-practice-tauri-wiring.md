---
id: 058
title: Practice Tauri command + frontend service wiring
status: review
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
- Compatibility additions: `src-tauri/src/practice/types.rs` (reciprocal serde derives only), `ARCHITECTURE.md` (required structural/data-flow documentation).

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

- 2026-09-05 — Task 9 started: running both complete gate sets, static structural checks, and the native E2E gate before changing status to review. No push or PR will be attempted, per user instruction.

- 2026-09-05 — Task 9 backend gates passed (cargo check, 279 tests, clippy --lib with warnings denied, fmt check); frontend typecheck/lint/build passed and full tests are running. Static checks: no hardcoded design values, no production component-to-service imports, git diff --check clean. Independent read-only plan check found no unintended deviations in Tasks 1–8; formal human/Claude review is still pending.

- 2026-09-05 — Task 9 complete: all backend/frontend gates and both native E2E flows passed, plus the real-app Practice IPC smoke. Status changed from in-progress to review now, after validation. Review remains pending; local branch is retained for the user. Task 9 is the final commit, with no push or PR.

## What was built / tested / left out

Built: `build_practice_registry` (`commands/practice.rs`) — a testable helper that
constructs the `ModuleRegistry` + fixed `ModuleInstallation`, registering `math_verify`
then `practice` in that order; `#[tauri::command]` handlers `generate_attempt`,
`evaluate_attempt`, `request_hint`, each translating between `practice::types`' snake_case
capability contract and a camelCase wire shape; real startup wiring in `lib.rs` (bundles
`knowledge-package/` as a Tauri resource, loads it via `load_knowledge_package` for the
first time outside a test, manages the registry + installation as Tauri state); the
frontend triad `src/types/practice.ts` + `src/services/practiceService.ts` +
`src/test/mockBackend.ts` wiring, tested through mocked IPC per `ARCHITECTURE.md` §5
rule 2.

Tested: `cargo test` across `commands::practice::tests` (registry construction, command
translation including a structural camelCase-key assertion, a full
generate→hint→evaluate sequence through the command layer); `npm run test` across
`practiceService.test.ts` (generate/evaluate open+solved/hint sequencing/unknown-attempt
errors), exercised through `handleMockInvoke`, never mocking the service module itself.
Gates run: `cargo check`/`test`/`clippy --lib -- -D warnings`/`fmt --check`,
`npm run typecheck`/`lint`/`build`/`test` — both sides, since this task touches `src-tauri/`
and `src/`.

Left out (per spec §1/§8, by design): Study Session UI (no page calls `practiceService.ts`
yet — this ships the contract, not a consumer); per-workspace module enable/disable wired
into capability resolution (fixed global `ModuleInstallation` instead); any `seed`
parameter on the generate command; the network-disabled offline acceptance test (depends
on Study Session UI existing first).

Final validation: Rust 279 tests (including 8 command tests); frontend 59 files / 149
tests, including all 5 Practice service tests. `GoalEditingSheet.test.tsx` passed on
both the baseline and final runs; no flake rerun was needed. `npm run test:e2e:linux`
passed both release-native flows: first launch → create workspace → home and restart
persistence. A temporary, ignored `src-tauri/target/058-practice-ipc-smoke.test.mjs`
also passed against the same release binary: `generateAttempt`, `evaluateAttempt`
(symbolic and numeric payloads), and `requestHint` reached their real async handlers
and returned the expected missing-family/attempt errors. This validates command names,
serde input translation, managed state extraction, and real resource loading without
substituting test content into the app.

All requested backend and frontend gates passed. Native validation used WSL,
`CARGO_TARGET_DIR=/var/tmp/axiom-058/target`,
`AXIOM_E2E_APP=/var/tmp/axiom-058/target/release/axiom`,
`TAURI_DRIVER_BIN=/home/marcus/.cargo/bin/tauri-driver`, and the matching WebKit driver
on PATH from `/tmp/axiom-058-webdriver/extracted/usr/bin`. The native build reported
the pre-existing `com.axiom.app` bundle-identifier warning; no new warnings were emitted
by lint or clippy. No hardcoded design values or production component-to-service
imports were introduced. ARCHITECTURE.md documents the new startup and IPC data path;
all consumers of the new shared types typecheck.

The real knowledge package currently has zero canonical ProblemFamily entries.
Successful generation therefore remains fixture-tested until the content follow-up
lands. No dependency, UI, module enablement policy, or capability wire naming changed.
No PR or push was attempted, per the user's explicit local-only instruction.

## Review

(filled in by reviewer)

## Follow-ups

Add canonical ProblemFamily content to the real knowledge-package before Study Session generation is exposed; the existing package currently contains only worked examples.
