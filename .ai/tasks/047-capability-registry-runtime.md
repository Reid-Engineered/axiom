---
id: 047
title: Module + capability registry runtime
status: proposed
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

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
