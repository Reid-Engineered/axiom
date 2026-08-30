---
id: 047
title: Module + capability registry runtime
status: done
owner: codex
stage: 8
depends_on: [046]
---

## Scope

The registry itself: registering validated `ModuleManifest`s (046's output) paired with
their Rust implementations, resolving a capability requirement to a provider per a
workspace's enabled-module order, and invoking it through a boundary shaped for a future
out-of-process module.

Explicitly not in scope: the full conformance/regression test suite and its fixture-module
library (048 — this task's own tests only need to prove the registry's own mechanics
correctly, not stand in for 048's broader harness); persisting `ModuleInstallation` to
SQLite (stays in-memory/test-fixture data for the whole runtime sub-project, per the design
spec §3 and §13); any Tauri command surface.

Contract locked below by Claude, per
`docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md` §6–§8; implementation
and tests are Codex's.

## Plan

- `src-tauri/src/modules/registry.rs` — `ModuleRegistry`, `CapabilityProvider`,
  `CapabilityCall`, `CallEnvelope`, `CapabilityHandle`, `ModuleInstallation`,
  `RegistryError`, `InvocationError`
- `src-tauri/src/modules/mod.rs` — re-export the new public types alongside 046's
- `src-tauri/Cargo.toml` — add `async-trait`, exact-pinned

## Locked contract

```rust
// registry.rs
#[async_trait::async_trait]
pub trait CapabilityProvider: Send + Sync {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, InvocationError>;
}
// async now, even though every Stage 8 provider computes and returns immediately — this is
// what lets a future out-of-process module implement this trait without a breaking
// signature change. Don't "simplify" it to a sync fn.

pub struct CapabilityCall<Input> {
    pub envelope: CallEnvelope,
    pub input: Input,
}
pub struct CallEnvelope {
    pub workspace_id: String,
    pub capability_id: CapabilityId,
    pub version: u32,
    pub calling_module_id: ModuleId,
}
pub struct CapabilityHandle {
    // fields private — opaque to callers, only constructible by ModuleRegistry::resolve
    module_id: ModuleId,
    capability_id: CapabilityId,
    version: u32,
}

pub struct ModuleInstallation {
    pub workspace_id: String,
    pub enabled_module_ids: Vec<ModuleId>, // order IS resolution priority (CORE.md §5)
}

pub struct ModuleRegistry {
    // internal: manifests keyed by ModuleId, each paired with its Box<dyn CapabilityProvider>
}

impl ModuleRegistry {
    pub fn new() -> Self { /* ... */ }

    /// Registers everything a ManifestSource discovers. Returns one Result per discovered
    /// module — a Rejected manifest (046's ManifestError, or a duplicate id) does NOT
    /// prevent the rest from registering. Pairs a validated manifest with its provider by
    /// matching ModuleId; the caller supplies providers (registration isn't just "load a
    /// source" — Rust has no dynamic module loading here, per the design spec §6).
    pub fn register(
        &mut self,
        manifest: ModuleManifest,
        provider: Box<dyn CapabilityProvider>,
    ) -> Result<ModuleId, RegistryError>; // Err(DuplicateModuleId) if already registered

    /// Walks `installation.enabled_module_ids` in order; returns the first registered,
    /// enabled module whose `provides` satisfies `requirement` (id matches, its version >=
    /// requirement.min_version). Err(NoCompatibleProvider) if none do — this single error
    /// covers both "nothing provides this capability at all" and "something provides it,
    /// but at too low a version," per the design spec §7 (both collapse to "nothing usable
    /// was found").
    pub fn resolve(
        &self,
        installation: &ModuleInstallation,
        requirement: &CapabilityRequirement,
    ) -> Result<CapabilityHandle, RegistryError>;

    /// Re-checks the handle's module is still enabled in `installation` (Err(ModuleDisabled)
    /// if not — a handle can outlive an enablement change) before dispatching to that
    /// module's CapabilityProvider::invoke. Wraps a provider Err in
    /// RegistryError::InvocationFailed rather than letting it propagate raw.
    pub async fn invoke<Input: Serialize, Output: DeserializeOwned>(
        &self,
        handle: &CapabilityHandle,
        installation: &ModuleInstallation,
        call: CapabilityCall<Input>,
    ) -> Result<Output, RegistryError>;
}

#[derive(Debug)]
pub enum RegistryError {
    DuplicateModuleId(ModuleId),
    NoCompatibleProvider { capability_id: CapabilityId, min_version: u32 },
    ModuleDisabled(ModuleId),
    InvocationFailed { module_id: ModuleId, capability_id: CapabilityId, cause: InvocationError },
}

#[derive(Debug)]
pub enum InvocationError {
    UnknownCapability { capability_id: CapabilityId, version: u32 }, // manifest declared it,
                                                                       // this provider
                                                                       // doesn't actually
                                                                       // handle it
    InvalidInput { capability_id: CapabilityId, message: String },   // input didn't
                                                                       // deserialize into
                                                                       // the provider's
                                                                       // expected type
    Failed { message: String }, // the provider's own internal failure
}
```

`RegistryError::InvocationFailed` wraps a `CapabilityProvider`'s `InvocationError` rather
than replacing it — a caller sees both "which module/capability failed" (registry-level
context) and "why" (this enum).

**Not an error, don't add a variant for it**: two modules both providing
`practice.generate@1` is the expected multi-provider case. `resolve()` picking the first
match by `enabled_module_ids` order is correct behavior, not something to reject.

## Worklog

- 2026-08-30 (codex, changes requested): Started the single blocking review fix. Updating
  `ARCHITECTURE.md`'s `modules/` responsibility to include the capability registry runtime;
  no registry code or 048 scope changes are needed.
- 2026-08-30 (codex, changes requested): Updated the backend folder inventory to describe
  `modules/` as owning validated manifests, discovery, and the capability registry runtime.
  `git diff --check` passes; returning 047 to review with 048 still untouched and blocked.
- 2026-08-30 (codex): Claimed the task after confirming 046 is done and archived, reading
  the locked contract, design spec §6–§8, and the manifest parser/validator implementation.
  Beginning the registry implementation with the locked trait/struct shapes; the task-local
  tests will cover Loaded/Rejected registration outcomes and Enabled/Disabled installation
  behavior without expanding into 048's conformance suite.
- 2026-08-30 (codex): Implemented `invoke<Input: Serialize, Output: DeserializeOwned>`
  exactly as sketched: caller input is converted with `serde_json::to_value`, providers see
  only `serde_json::Value`, and provider output is converted with
  `serde_json::from_value`. Serialization/deserialization failures stay within the locked
  error taxonomy as wrapped `InvalidInput`/`Failed` causes. Task-local tests now exercise a
  successful Loaded registration, a Rejected duplicate followed by another successful
  registration, resolution across Enabled and Disabled installations, a stale handle after
  disablement, typed invocation, and wrapped provider failure. These signature and lifecycle
  notes remain guidance for 048's eventual conformance coverage, not blockers for 047.
- 2026-08-30 (codex): Implementation and task-local tests are complete. All standalone
  Rust gates pass. The required native E2E command built the frontend and release Tauri
  binary, then both flows stopped before application launch because `tauri-driver` is not
  installed (`spawn tauri-driver ENOENT`); recorded below as an environment blocker, not an
  E2E pass. Moving 047 to review without starting blocked task 048.
- 2026-08-30 (claude-code): Contract locked from the design spec §6–§8. `invoke`'s exact
  generic signature above (`Input: Serialize, Output: DeserializeOwned`, serializing to/from
  `serde_json::Value` internally before calling `CapabilityProvider::invoke`) is a
  reasonable shape but not pinned as tightly as the trait/struct definitions — if a cleaner
  signature emerges during implementation that preserves "callers work with typed
  input/output, the registry and providers only ever see `serde_json::Value`," that's fine;
  note the actual shape used in this task's Worklog so 048's conformance tests use the real
  signature.
- 2026-08-30 (claude-code): Module lifecycle (spec §8) for this task's tests to exercise:
  `Discovered` (found by a `ManifestSource`, handled entirely in 046) → `Loaded`
  (`register()` succeeds) or `Rejected` (`register()` returns `Err`, doesn't block other
  modules) → per-workspace `Enabled`/`Disabled` via `ModuleInstallation`, meaningful only
  for `Loaded` modules.

## What was built / tested / left out

**Built**

- Added `ModuleRegistry`, backed by validated manifests paired with
  `Box<dyn CapabilityProvider>`, with duplicate-ID rejection that leaves subsequent
  registrations unaffected.
- Added ordered, workspace-installation-scoped capability resolution with minimum-version
  matching and opaque `CapabilityHandle`s.
- Added typed async invocation across the JSON-only provider boundary, including stale
  enablement checks and `InvocationFailed` wrapping that retains module, capability, and
  provider-error context.
- Added the locked call/envelope, installation, provider, handle, and error types; re-exported
  them from `modules`; and exact-pinned `async-trait = 0.1.92`.
- Added five registry unit tests covering registration isolation, ordered resolution,
  disabled/incompatible providers, typed dispatch, stale handles, and provider failures.
- Updated `ARCHITECTURE.md`'s backend inventory to include the registry runtime responsibility.

**Tested**

- `cargo fmt --all --check` — passed.
- `cargo check --locked` — passed.
- `cargo test --locked` — passed, 30 tests total including five registry tests.
- `cargo clippy --all-targets --locked -- -D warnings` — passed with zero warnings.
- `git diff --check` — passed.
- Changes-requested documentation fix: `git diff --check` — passed.
- `npm run test:e2e:linux` — release frontend and Tauri binary built successfully; both
  native flows were blocked before launch because `tauri-driver` is unavailable on `PATH`
  (`spawn tauri-driver ENOENT`). This is not claimed as an E2E pass.

**Left out**

- The reusable conformance harness, broader regression fixtures, and every row in 048's
  locked coverage table remain in blocked task 048.
- `ModuleInstallation` persistence, any Tauri command surface, dynamic module loading, and
  real first-party module implementations remain outside this task's scope.

## Review

Reviewer: claude-code
Date: 2026-08-30

- [x] Correctness — pass. `register`/`resolve`/`invoke` match the locked contract exactly
      (types, error taxonomy, `NoCompatibleProvider` collapsing missing-vs-incompatible per
      spec §7). `resolve()` walks `enabled_module_ids` — a `Vec`, not the internal
      `HashMap` — so first-match ordering is deterministic, which is the one easy way to get
      this silently wrong. The five tests are real assertions on outcomes (registration
      isolation after a rejected duplicate, priority flip on reordering, stale-handle
      `ModuleDisabled` after disablement, wrapped `InvocationFailed` with cause intact), not
      happy-path-only.
- [x] Architecture conformance (`ARCHITECTURE.md` §5 rules) — N/A, no frontend data-flow
      rules apply to `src-tauri/`; no new global state, no `src/types/` changes.
- [x] UI rules (`AGENTS.md`) — N/A, backend-only change, no markup/tokens touched.
- [ ] Process — FAIL: `ARCHITECTURE.md:72` still describes `modules/` as "validated module
      manifests + discovery boundary" (046's scope). This task added the capability registry
      — resolution and typed async invocation across the provider boundary — which is a
      distinct runtime responsibility the current one-liner doesn't mention. Per
      `.ai/quality-gates.md` "Structural changes" and this checklist's own "ARCHITECTURE.md
      updated if structure changed," that line needs a one-word-ish addition (e.g.
      "validated module manifests, discovery boundary, and the capability registry
      runtime"). Everything else under Process is solid: gates actually ran (`cargo fmt`,
      `cargo check --locked`, `cargo test --locked` — 30 tests, `clippy -D warnings`, `git
      diff --check`, all recorded truthfully); E2E was correctly recorded as an environment
      blocker (`tauri-driver` missing) rather than claimed as a pass, consistent with
      `.ai/quality-gates.md`'s explicit carve-out and the 040/042/044 precedent; worklog is
      detailed enough to follow without the diff; scope stayed inside 047, 048 untouched.

Verdict: changes-requested

### Re-review (f586abb)

- [x] Process — PASS: `ARCHITECTURE.md:72–73` now reads "validated module manifests,
      discovery boundary, and capability registry runtime," column-aligned with the rest of
      the tree's wrapped descriptions (e.g. `hooks/` at line 58). No registry code or 048
      scope touched — fix matched exactly what was requested. `git diff --check` re-run,
      passes.

All four checklist sections now pass. Note for the human: per `.ai/merge-strategy.md` §"Who
can merge what," the `ARCHITECTURE.md` edit means this merge needs human sign-off — a
reviewing agent can approve the task but not merge it unilaterally.

Verdict: done

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
