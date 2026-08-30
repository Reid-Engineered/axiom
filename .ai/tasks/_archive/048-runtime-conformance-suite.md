---
id: 048
title: Runtime + manifest conformance and regression test suite
status: done
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

- 2026-08-30 (codex): Claimed the task after confirming 046 and 047 are done and archived,
  then reading the actual manifest/source and registry implementations. I will add one named
  test function for every locked coverage row. The conformance entry point will accept only
  a caller-supplied `(ModuleManifest, Box<dyn CapabilityProvider>)`; fixture construction
  stays outside it so task 052 can pass Practice through the identical harness unchanged.
  Existing case-named TOML regressions will remain intact, and every new manifest fixture
  will be named for the behavior it permanently covers.
- 2026-08-30 (codex): The generic harness will probe each declared `(CapabilityId, version)`
  with JSON `null`. `Ok` proves a successful generic invocation; a matching `InvalidInput`
  also proves the provider recognized a typed capability for which the manifest carries no
  valid sample payload. `UnknownCapability` or `Failed` for a declared capability fails
  conformance. A dynamically chosen undeclared ID must return `UnknownCapability`, after
  which the same provider is registered and every manifest declaration must resolve. This
  behavior depends only on the locked manifest/provider contract, never on fixture or
  first-party identities.
- 2026-08-30 (codex): Implemented all ten locked rows as named test functions across
  `tests/registry.rs` and `tests/conformance.rs`. The conformance test retains an invocation
  log outside the consumed trait object and proves the harness called every declaration plus
  its dynamically selected undeclared ID. The harness itself has exactly the reusable
  `(ModuleManifest, Box<dyn CapabilityProvider>)` input and contains no fixture, module-ID,
  or first-party branch. Added test-only echo/failure providers and permanent case-named
  fixtures for duplicate IDs/providers, workspace isolation, parse/register partial failure,
  and multi-capability conformance; no existing regression fixture was removed or renamed.
- 2026-08-30 (codex): All applicable standalone gates pass. The native E2E command built
  the frontend and release Tauri binary, then both flows stopped before application launch
  because `tauri-driver` is not installed (`spawn tauri-driver ENOENT`); recorded below as
  an environment blocker, not an E2E pass. The staged diff contains all ten locked test
  functions, no code TODOs, nine new permanent TOML fixtures, and no fixture deletions.
  Moving 048 to review.
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

**Built**

- Added ten named tests, one for every row in the locked coverage table: duplicate module
  ID, missing dependency, incompatible version, duplicate-provider ordering, disabled
  module, workspace isolation, invocation failure, serialization, parse/validate/register
  partial-failure isolation, and contract conformance.
- Added the reusable `assert_provider_conforms(ModuleManifest,
  Box<dyn CapabilityProvider>)` harness. It probes every declared capability without knowing
  module identity or input types, requires an undeclared capability to fail as
  `UnknownCapability`, and registers/resolves every declaration through `ModuleRegistry`.
- Added two test-only provider implementations: an echo provider with optional invocation
  logging and a deliberately failing provider. Neither is reachable from non-test code.
- Added nine case-named, permanently referenced TOML fixtures covering duplicate IDs,
  duplicate providers, workspace isolation, parse/register valid and malformed cases, and
  multi-capability conformance. All eleven fixtures inherited from 046 remain in place.

**Tested**

- `cargo fmt --all --check` — passed.
- `cargo check --locked` — passed.
- `cargo test --locked` — passed, 40 tests total; the ten new locked-coverage tests all ran.
- `cargo clippy --all-targets --locked -- -D warnings` — passed with zero warnings.
- `git diff --cached --check` — passed.
- Audited the new test files for `TODO`, `todo!`, and `unimplemented!` — none present.
- `npm run test:e2e:linux` — frontend and release Tauri binary built successfully; both
  native flows were blocked before launch because `tauri-driver` is unavailable on `PATH`
  (`spawn tauri-driver ENOENT`). This is not claimed as an E2E pass.

**Left out**

- Real Practice behavior, bundled-module content, and any production registration path
  remain in task 052 or later work. This task adds test-only providers and fixtures only.
- No module-specific input fixture was added to the generic harness. A declared provider may
  return a matching `InvalidInput` for the harness's JSON-null probe because valid payloads
  are capability-specific and absent from `ModuleManifest`; `UnknownCapability` and `Failed`
  still fail conformance.
- No existing regression fixture was renamed or deleted. Future bug fixtures remain subject
  to the same permanent, case-named convention.

## Review

Reviewer: claude-code
Date: 2026-08-30

- [x] Correctness — pass. Checked all ten locked-coverage rows against their actual test
      bodies, not just their names:
      - Duplicate module ID: registers `duplicate-module-id-{first,second}.toml` (same id,
        different capability version), asserts `DuplicateModuleId` on the second, then
        resolves+invokes to confirm the *first* registration is still intact and unaffected.
      - Missing dependency / incompatible version: two genuinely distinct scenarios, not one
        test copy-pasted — the incompatible-version case registers a real provider at
        `practice.generate@1` (confirmed in `duplicate-provider-first.toml`) and requires
        `min_version: 2`, correctly exercising the "something exists but too low" path
        rather than "nothing exists at all," even though both collapse to the same
        `RegistryError` variant per spec.
      - Duplicate-provider: reorders `enabled_module_ids` between two installations and
        asserts the *invoked* provider's label actually flips — proves resolution order, not
        just that `resolve()` returns something.
      - Disabled-module / workspace-isolation: two separate tests, using a second fixture
        (`workspace-secondary-provider.toml`, distinct capability) so isolation is proven
        against real separate `ModuleInstallation`s rather than an empty one twice.
      - Invocation failure: uses the fixture `FailingProvider`, asserts the exact wrapped
        `RegistryError::InvocationFailed{module_id, capability_id, cause}`, not just "is an
        error."
      - Serialization: round-trips `ModuleManifest`, `CallEnvelope`, and
        `CapabilityCall<SerializableInput>` (a real struct with real field values, not an
        empty/trivial payload) through `serde_json`, checking structural equality.
      - Parse→validate→register integration: `EmbeddedManifestSource::new(...)` (an existing
        046 constructor, not something added for this task) over 3 fixtures including one
        with an actual unclosed-quote TOML syntax error; asserts exactly the broken one is
        rejected as `TomlSyntax` while the other two register and resolve.
      - Contract/conformance: `assert_provider_conforms` in `conformance.rs` takes only
        `(ModuleManifest, Box<dyn CapabilityProvider>)`, iterates the manifest's own
        `provides` to prove every declared capability is invokable (accepting `Ok` or a
        matching `InvalidInput` — reasonable, since a null probe payload can legitimately
        fail typed deserialization — but treating `UnknownCapability`/`Failed` on a
        *declared* capability as a hard failure), then synthesizes a guaranteed-undeclared
        capability id and asserts it fails cleanly with `UnknownCapability`. **No fixture
        name, module id, or first-party branch anywhere in the harness function** — this is
        the one thing the task file explicitly said to check hardest, and it holds up. I
        re-ran `cargo test --locked` myself (40 passed), `cargo fmt --all --check`, and
        `cargo clippy --all-targets --locked -- -D warnings` independently rather than
        trusting the worklog — all clean.
- [x] Architecture conformance — pass. Confirmed via diff that zero non-test files changed
      (`registry.rs`, `manifest.rs`, `source.rs`, `mod.rs`, `identifier.rs`, `Cargo.toml` all
      untouched) — this is purely additive test/fixture work, exactly the task's scope. Test
      providers live under `tests/fixtures/`, gated behind `modules/mod.rs`'s existing
      `#[cfg(test)] mod tests;`, unreachable from any production path.
- [x] UI rules — N/A, backend test code only.
- [x] Process — pass. Fixture count confirmed by directory listing: 20 total (11 inherited
      from 046, 9 new), all case-named, none renamed or deleted. Gates recorded truthfully;
      E2E `tauri-driver` blocker recorded honestly as an environment limitation rather than
      claimed as a pass, consistent with `.ai/quality-gates.md` and the established
      040/042/044/047 precedent. Worklog is detailed enough to follow without the diff.

Verdict: done

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
