---
id: 046
title: module.toml schema + manifest parser/validator
status: done
owner: codex
stage: 8
depends_on: [045]
---

## Scope

Parse and semantically validate `module.toml` into a typed `ModuleManifest`. Raw TOML must
never propagate past this boundary — everything downstream works only with the validated
types below. Also ships the `ManifestSource` abstraction and its one Stage 8 implementation,
`EmbeddedManifestSource`.

Explicitly not in scope: the registry, resolution, or invocation (047); the conformance/
regression test suite and fixture provider modules (048); any Tauri command surface (Stage
8's runtime sub-project has no UI). This task is provable entirely through `cargo test`.

Contract locked below by Claude (`AGENTS.md` §Roles) per
`docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md` §4–§7; implementation
and tests are Codex's.

## Plan

- `src-tauri/src/modules/mod.rs` — module root, re-exports (also add `mod modules;` to
  `src-tauri/src/lib.rs` or `main.rs`, wherever the crate root currently declares its
  top-level modules)
- `src-tauri/src/modules/identifier.rs` — `ModuleId`, `CapabilityId`
- `src-tauri/src/modules/manifest.rs` — `RawModuleManifest`, `ModuleManifest`,
  `CapabilityDescriptor`, `CapabilityRequirement`, `ManifestError`, the
  raw-parse-then-validate pipeline
- `src-tauri/src/modules/source.rs` — `ManifestSource` trait, `EmbeddedManifestSource`
- `src-tauri/src/modules/tests/fixtures/` — a handful of `.toml` fixtures for this task's own
  tests (valid manifest, and one fixture per `ManifestError` variant below); 048 grows this
  directory further, doesn't replace it
- `src-tauri/Cargo.toml` — add `toml` and `semver`, exact-pinned to current stable
  (`=X.Y.Z`, matching every existing dependency in this file — do not use a caret range)

## Locked contract

Identifier grammar (both `ModuleId` and `CapabilityId` use it — dot-segmented, lowercase, at
least two segments):

```
^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$
```

`practice.generate`, `org.axiom.practice` match; `Practice`, `practice`,
`practice..generate` don't.

```rust
// identifier.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(String);
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(String);

impl ModuleId {
    pub fn new(value: impl Into<String>) -> Result<Self, ManifestError> { /* grammar check */ }
}
impl CapabilityId {
    pub fn new(value: impl Into<String>) -> Result<Self, ManifestError> { /* grammar check */ }
}
```

```rust
// manifest.rs
#[derive(Debug, Clone, Deserialize)]
pub struct RawModuleManifest {
    pub manifest_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub minimum_axiom_version: String,
    pub offline: String,
    #[serde(default)]
    pub provides: Vec<RawCapabilityDescriptor>,
    #[serde(default)]
    pub requires: Vec<RawCapabilityRequirement>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct RawCapabilityDescriptor { pub id: String, pub version: u32 }
#[derive(Debug, Clone, Deserialize)]
pub struct RawCapabilityRequirement { pub id: String, pub min_version: u32 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub id: ModuleId,
    pub name: String,
    pub version: semver::Version,
    pub minimum_axiom_version: semver::Version,
    pub offline: OfflineCapability,
    pub provides: Vec<CapabilityDescriptor>,
    pub requires: Vec<CapabilityRequirement>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDescriptor { pub id: CapabilityId, pub version: u32 }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityRequirement { pub id: CapabilityId, pub min_version: u32 }

/// Deliberately distinct from src/types/common.ts's OfflineStatus ("Works offline" /
/// "Online enhanced" / "Internet required") — that's human-facing catalog display copy for
/// ModuleMetadata; this is a machine contract value for the manifest and shouldn't couple
/// to UI wording. Checked: today's Rust DTOs (src-tauri/src/commands/models.rs) represent
/// the catalog's offline_status as a plain String, not an existing enum — there is no type
/// to accidentally collide with here, this is a genuinely new one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OfflineCapability { Full, Enhanced, Required }

pub fn validate(raw: RawModuleManifest) -> Result<ModuleManifest, ManifestError> { /* ... */ }

#[derive(Debug)]
pub enum ManifestError {
    UnsupportedManifestVersion { found: u32, supported: &'static [u32] }, // supported = &[1]
    MissingModuleId,
    MalformedVersion { field: &'static str, value: String },
    InvalidIdentifier { value: String },
    DuplicateCapability { module_id: String, capability_id: String }, // two [[provides]]
                                                                        // with the same
                                                                        // id + version,
                                                                        // within ONE manifest
    TomlSyntax(toml::de::Error),
}
```

```rust
// source.rs
pub trait ManifestSource {
    fn discover(&self) -> Result<Vec<(ModuleId, String /* raw toml text */)>, ManifestError>;
}
pub struct EmbeddedManifestSource {
    // wraps a fixed set of (&'static str id, &'static str toml) pairs built with
    // include_str!; Stage 8's only real bundled manifest doesn't exist until task 052
    // (Practice), so for this task EmbeddedManifestSource ships with zero or one trivial
    // test-fixture manifest, not a real one — proving the mechanism, not shipping content.
}
```

`minimum_axiom_version` compares against `env!("CARGO_PKG_VERSION")` parsed as
`semver::Version`, read through one small accessor function (e.g.
`pub fn axiom_version() -> semver::Version` in `modules/mod.rs`) rather than called inline —
so the eventual reconciliation of `package.json`/`Cargo.toml`/`tauri.conf.json`'s three
independent version strings only requires changing that one function later. This task does
not do that reconciliation.

`manifest_version`'s supported set is `&[1]` for Stage 8 — anything else is
`UnsupportedManifestVersion`.

## Worklog

- 2026-08-30 (claude-code): Contract locked from
  `docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md` §4–§7. Two things
  worth flagging before implementation starts:
  1. Syntactic failures (malformed TOML, wrong field types in `RawModuleManifest`) surface
     as `ManifestError::TomlSyntax` via the underlying `toml`/`serde` error. Semantic
     failures only checkable after a structurally-valid parse (empty `id`, an `id` or
     `capability id` that fails the identifier grammar, a `version`/`minimum_axiom_version`
     string that doesn't parse as semver, an unsupported `manifest_version`, two
     `[[provides]]` entries with the same `id` + `version`) get their own specific variants.
     This means `id`/`name`/etc. are likely `String` (not `Option<String>`) on
     `RawModuleManifest` so a genuinely absent field is a `TomlSyntax` error, while an
     empty-string `id` is caught explicitly as `MissingModuleId` during `validate()` — your
     call if a different split reads cleaner, but keep the two failure modes distinguishable
     in tests either way.
  2. `EmbeddedManifestSource` genuinely has no real content to embed yet — Practice's actual
     `module.toml` doesn't exist until task 052. Don't invent placeholder subject-specific
     content to make this task feel more complete; a minimal, clearly-test-only fixture
     manifest (e.g. `org.axiom.test-fixture`) is correct and sufficient here.
- 2026-08-30 (codex): Implemented the boundary as `parse(&str)`, which maps TOML and raw
  deserialization failures to `TomlSyntax`, followed by the contract's public `validate()`
  step for semantic checks. `EmbeddedManifestSource::default()` embeds only the clearly
  test-only `org.axiom.test_fixture` manifest; `new()` accepts a static fixture set so later
  conformance tests can exercise the same source implementation without production content.
- 2026-08-30 (codex): Kept `ManifestError` exactly as locked. The minimum Axiom version is
  parsed into `semver::Version` and compared in tests through `axiom_version()`. A
  well-formed minimum newer than the running package cannot currently be rejected without
  adding an error variant that the locked contract does not define; recorded under
  Follow-ups instead of misclassifying it as `MalformedVersion` or expanding the contract.
- 2026-08-30 (codex, changes requested): Rebased onto contract commit `8918f79` and added
  the newly locked `IncompatibleAxiomVersion { required, running }` variant. `validate()`
  now compares the parsed minimum against one `axiom_version()` value and rejects a newer
  requirement before constructing `ModuleManifest`; `incompatible-axiom-version.toml`
  permanently covers the case.

## What was built / tested / left out

**Built**

- Added the `modules` crate surface with grammar-validating `ModuleId`/`CapabilityId`, the
  raw-TOML-to-validated-manifest pipeline, the locked serializable manifest types and error
  taxonomy, and the `axiom_version()` accessor.
- Added `ManifestSource` and an `EmbeddedManifestSource` containing only a compile-time
  embedded test fixture, plus a constructor for later static fixture sets.
- Added eleven named TOML regression fixtures covering every `ManifestError` variant, both
  malformed semver fields, an absent required id, a wrong field type, and malformed TOML.
- Exact-pinned `semver = 1.0.28` (with serde support) and `toml = 1.1.4`, updated the lockfile,
  exposed `modules` from the crate root, and documented the new backend folder in
  `ARCHITECTURE.md`.

**Tested**

- `cargo check --locked` — passed.
- `cargo test --locked` — passed, 25 tests total including 11 manifest/source tests.
- `cargo fmt --all --check` — passed.
- `cargo clippy --all-targets --locked -- -D warnings` — passed with zero warnings.
- `git diff --check` — passed.
- `npm run test:e2e:linux` — not run: neither `WebKitWebDriver` nor `tauri-driver` is
  available on `PATH` in this environment. The task's locked standalone gate set is fully
  covered above; this is not claimed as an E2E pass.

**Left out**

- Registry, resolution, invocation, fixture providers, broader conformance coverage, Tauri
  commands, and real bundled module content remain in their explicitly scoped later tasks.

## Review
Reviewer: claude-code
Date: 2026-08-30
- [x] Correctness — pass, with one gap (below). `parse`/`validate` match the locked pipeline
      exactly; the syntactic/semantic split matches the Worklog's guidance precisely
      (`toml-syntax.toml`, `wrong-field-type.toml`, `absent-id-field.toml` all correctly
      bucket to `TomlSyntax`, not a semantic variant). Identifier grammar implementation
      matches the regex exactly, verified by hand against both the positive and negative
      test cases. `EmbeddedManifestSource`'s embedded fixture is genuinely test-only per the
      Worklog's instruction not to invent placeholder subject content.
- [ ] Correctness — **FAIL (contract gap, not an implementation bug)**: no way to reject a
      well-formed `minimum_axiom_version` newer than `axiom_version()`. Codex found this and
      correctly declined to guess at an unauthorized variant — filed as a Follow-up instead,
      exactly the right call on a contract it doesn't own. Resolved on my end: added
      `ManifestError::IncompatibleAxiomVersion { required, running }` to `CORE.md` §3 and
      the design spec (commit `8918f79`, this session) — same shape as
      `UnsupportedManifestVersion`, checked as a pure function of the manifest plus
      `axiom_version()`, so it belongs in `ManifestError`, not `RegistryError`.
      **What's needed to close this**: `validate()` checks
      `minimum_axiom_version <= axiom_version()` right where `parse_version` produces it
      (same spot `UnsupportedManifestVersion`'s check lives), returning
      `Err(IncompatibleAxiomVersion { required: minimum_axiom_version, running: axiom_version() })`
      when it doesn't hold — plus a fixture/test proving it (a manifest with, e.g.,
      `minimum_axiom_version = "99.0.0"`).
- [x] Architecture conformance — pass. `ARCHITECTURE.md` updated for the new `modules/`
      folder (structural addition, correctly done in the same task per `CLAUDE.md`). Exact
      pins on both new dependencies (`semver = "=1.0.28"`, `toml = "=1.1.4"`), matching this
      repo's convention. `mod modules;` added in the correct, minimal spot.
- [x] Process — pass. Re-ran `cargo check --locked`, `cargo test --locked` (24/24),
      `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, and
      `git diff --check` myself — all clean, matching the task file's claims exactly.
      `npm run test:e2e:linux` correctly not claimed as a pass (this task has no frontend
      surface to exercise anyway — N/A more than blocked, but the honest non-claim is the
      right instinct to keep). Worklog and "What was built" are detailed and accurate.

Verdict: changes-requested — the one gap above, now unblocked with a locked fix to
implement. Everything else in this task is solid work; re-request review once the
`IncompatibleAxiomVersion` check and its test are in.

### Re-review
Reviewer: claude-code
Date: 2026-08-30
- [x] Correctness — pass. `validate()`'s new check sits exactly where specified (right
      after `parse_version` produces `minimum_axiom_version`, same spot
      `UnsupportedManifestVersion`'s check lives), compares `>` against `axiom_version()`
      (correctly permissive at equality), and `incompatible-axiom-version.toml` +
      `newer_minimum_axiom_version_is_incompatible` prove it, asserting both `required` and
      `running` fields.
- [x] Process — pass. Re-ran `cargo check --locked`, `cargo test --locked` (25/25),
      `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, and
      `git diff --check` myself — all clean, matching the task file's claims.

Verdict: pass. All prior findings closed.

## Follow-ups

None from implementation. The minimum-version contract gap identified in the first review
was resolved by contract commit `8918f79` and implemented in this changes-requested pass.
