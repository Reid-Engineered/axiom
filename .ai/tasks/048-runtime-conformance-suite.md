---
id: 048
title: Runtime + manifest conformance and regression test suite
status: proposed
owner: codex
stage: 8
depends_on: [046, 047]
---

## Scope

The full test bar from `docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md`
§9: every test class the manifest parser (046) and registry (047) didn't already cover in
their own unit tests, plus a reusable conformance harness any module — fixture or real —
can be run through, plus a permanent regression-fixture directory for `module.toml` cases.

This is the task that proves the sub-project's central architectural claim — "bundled
doesn't mean privileged" — so its review (Claude, a different agent than the implementer,
per `AGENTS.md` §Roles, same as every task in this repo) should specifically check that the
conformance harness doesn't quietly special-case anything about being first-party.

Explicitly not in scope: real Practice logic. Every fixture module here is small,
test-only, and would never be labeled as available to a learner — the bundled Axiom
Practice module (task 052, a later Stage 8 sub-project) is what actually runs the
conformance harness this task builds "for real."

## Plan

- `src-tauri/src/modules/tests/conformance.rs` — the reusable conformance harness (a
  function or macro taking a `ModuleManifest` + `Box<dyn CapabilityProvider>` and running
  the shared checks any module must pass)
- `src-tauri/src/modules/tests/registry.rs` — the cross-cutting registry tests below that
  046/047's own unit tests didn't cover
- `src-tauri/src/modules/tests/fixtures/` — extends 046's directory; add the malformed-
  manifest cases and multi-provider fixtures this task needs
- `src-tauri/src/modules/tests/fixtures/providers.rs` (or similar) — 2–3 tiny test-only
  `CapabilityProvider` implementations (e.g. an echo provider, a deliberately-failing
  provider), clearly named and doc-commented as fixtures, never referenced from any
  non-test code path

## Locked test coverage

Each row must have a real test function, not a TODO — this is the definition of done for
this task, not a checklist to leave partially checked:

| Test class | What it proves |
|---|---|
| Duplicate module ID | Registering two manifests with the same `id` → `RegistryError::DuplicateModuleId`, second registration doesn't corrupt the first |
| Missing dependency | `resolve()` for a capability nothing registered provides → `NoCompatibleProvider` |
| Incompatible version | A provider registered at `version: 1`, requirement asks `min_version: 2` → `NoCompatibleProvider` (same variant as missing dependency — both are "nothing usable was found") |
| Duplicate-provider | Two fixture modules both provide `practice.generate@1`; `resolve()` picks the one earlier in `ModuleInstallation.enabled_module_ids`; reordering the list changes which one wins |
| Disabled-module | A module registered globally but absent from a workspace's `enabled_module_ids` → `resolve()` for that workspace skips it |
| Workspace-isolation | Two `ModuleInstallation`s for different `workspace_id`s with different `enabled_module_ids`; a capability enabled in one is not resolvable in the other |
| Invocation failure | A fixture provider whose `invoke()` returns `Err` → `invoke()` on the registry surfaces `RegistryError::InvocationFailed` cleanly, no panic |
| Serialization | `ModuleManifest`, `CapabilityCall<T>`, `CallEnvelope` round-trip through `serde_json::to_string`/`from_str` with no data loss (structural equality, not just "it compiles") |
| Parse → validate → register integration | Full pipeline through `EmbeddedManifestSource` against a fixture set that includes at least one deliberately-broken manifest; asserts the broken one is `Rejected` while the valid ones still register |
| Contract/conformance | The harness itself: given any `(ModuleManifest, Box<dyn CapabilityProvider>)`, asserts every capability the manifest's `provides` declares is actually invokable, and that invoking an undeclared capability id fails cleanly rather than panicking |

## Worklog

- 2026-08-30 (claude-code): Regression-fixture convention, per the design spec §9's own
  note extending it to the runtime (not just Practice's future generator): every
  `module.toml` fixture under `src-tauri/src/modules/tests/fixtures/` is named by the case
  it exercises (`missing-id.toml`, `duplicate-capability.toml`, `unsupported-version.toml`,
  …), referenced by name from its test, and never deleted once a bug that motivated it is
  fixed — it stays as the permanent regression check for that exact failure mode.
- 2026-08-30 (claude-code): The conformance harness (locked-test-coverage table, last row)
  is deliberately generic over `(ModuleManifest, Box<dyn CapabilityProvider>)` rather than
  hardcoded to any fixture — that genericity is the actual architectural proof this task
  exists for. When task 052 builds the real Practice module later, running Practice through
  this exact harness (not a new bespoke test) is what confirms "bundled doesn't mean
  privileged" in practice, not just in the design doc's prose.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
