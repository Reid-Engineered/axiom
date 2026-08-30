# Module & Capability Runtime — design

Sub-project 1 of Stage 8 (Capability Runtime + Practice Core Utility). Covers Stage 8's
sections 8.0 (Architecture lock) and 8.1 (Capability Runtime) — `.ai/tasks/` 045–048.
Everything from Stage 8 §8.2 onward (Knowledge Package, Problem schema, Practice Utility,
verification capability, reference package, deterministic generation, UI integration,
offline proof) is explicitly **out of scope** here and gets its own brainstorm-and-spec pass
once this contract is locked and stable — those pieces are consumers of this one, not
independent of it, so designing them now would mean designing against a guess.

Source material: Marcus's Stage 8 proposal (pasted in full into the originating
conversation), `CORE.md` (the existing TS-first draft contract this design supersedes for
the runtime pieces), `ARCHITECTURE.md`, and `src/types/module.ts` / the existing
`modules`/`workspace_modules` SQLite tables (Stage 6/7's catalog layer, untouched by this
design).

---

## 1. Scope

Build the module manifest format, its parser/validator, and the capability
registry/resolution runtime — entirely in Rust, entirely first-party/in-tree, with **no**
UI and **no** real subject-specific module yet. The deliverable is a Rust subsystem that can:

- discover `module.toml` manifests from an abstract source
- parse and semantically validate them into a typed `ModuleManifest`
- register modules, rejecting individually-broken manifests without blocking the rest
- resolve a capability requirement to a provider, respecting per-workspace enablement and
  provider ordering
- invoke a resolved capability through a boundary shaped for a future out-of-process module,
  even though every Stage 8 provider runs in-process

Practice itself (the first real consumer) is built later, against this contract, once it's
locked. Task 048's conformance suite uses small fixture modules, not real subject logic.

## 2. Decisions carried in from earlier in the session

These were confirmed before the section-by-section design and aren't re-litigated below:

- **The runtime lives in Rust** (`src-tauri/`), not TypeScript. The frontend will eventually
  call it through Tauri commands, the same pattern `src/services/*` already uses for Stage 7
  persistence. This makes CORE.md §2's "plain, structurally-clonable data" boundary a
  property the IPC layer enforces, not just a convention.
- **Manifest discovery is behind an abstraction.** The registry depends on a `ManifestSource`
  trait, not a concrete mechanism. Stage 8's only implementation is `EmbeddedManifestSource`
  — all first-party manifests ship embedded via `include_str!` at compile time, for
  determinism, matching the precedent set by SQL migrations
  (`src-tauri/src/db/migrations/`). `include_str!` is an implementation detail of that one
  source, not part of the contract. Filesystem and marketplace sources are future
  implementations of the same trait; nothing about the registry needs to change to add them.
- **`minimum_axiom_version` compares against `CARGO_PKG_VERSION`**, read through one small
  accessor rather than called inline at every use site, so that if/when `package.json`,
  `Cargo.toml`, and `tauri.conf.json`'s three independent `"0.1.0"` strings ever get
  reconciled into one source of truth, only that accessor changes. Reconciling those three
  files is explicitly out of scope here.

## 3. Layering: three types, not two

Your framing, with one rename to avoid a collision:

| Layer | Type | Lives | Purpose |
|---|---|---|---|
| Catalog | `ModuleMetadata` | `src/types/module.ts` (today's `Module`, renamed) | What humans see — marketplace rows, trust badges, screen 10 copy. Backed by the existing `modules`/`workspace_modules` SQL tables, untouched by this design. |
| Runtime contract | `ModuleManifest` | Rust, `src-tauri/src/modules/` (new) | What Core understands — parsed and validated from `module.toml`. |
| Per-workspace config | `ModuleInstallation` | Rust | How a workspace has a module enabled/configured. |

`ModuleInstallation` replaces your proposed `WorkspaceModule` name — the SQL table
`workspace_modules` already exists for the *catalog* layer (`workspace_id, module_id,
enabled, visibility`, matching `Module` column-for-column). Reusing the name for this
unrelated runtime concept would be confusing even though a Rust type and a SQL table can
never technically collide.

**Scoping call**: for 045–048, `ModuleInstallation` stays in-memory/test-fixture data.
`resolve(workspace_id, requirement)` needs *something* to walk per CORE.md §5's "first match
by `enabledModuleIds` order" rule, but wiring it to real SQLite persistence is a later task's
job, once Practice/UI actually needs a workspace to have configurable installs.

## 4. `module.toml` schema

```toml
manifest_version = 1          # exact-match against a small supported set (currently just
                               # {1}); anything else -> ManifestError::UnsupportedManifestVersion

id = "org.axiom.practice"     # required; reverse-DNS-style; validated against the
                               # identifier grammar in §5
name = "Axiom Practice"       # required; display-only, no semantic meaning to the runtime
version = "0.1.0"             # required; the module's own semver

minimum_axiom_version = "0.8.0"   # required; semver, compared per §2
offline = "full"                  # enum: "full" | "enhanced" | "required" — a machine
                                   # contract value, deliberately distinct from
                                   # src/types/common.ts's OfflineStatus ("Works offline" /
                                   # "Online enhanced" / "Internet required"), which is
                                   # human-facing display copy for the catalog layer (§3).
                                   # The manifest shouldn't couple to UI wording; see §6's
                                   # OfflineCapability.

[[provides]]
id = "practice.generate"      # capability id; validated against the identifier grammar
version = 1                   # positive integer

[[requires]]
id = "knowledge.query"
min_version = 1
```

## 5. Identifier grammar

Both capability IDs and module IDs: dot-segmented, lowercase, at least two segments —
matches every example in the source spec.

```
^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$
```

`practice.generate`, `org.axiom.practice` match; `Practice`, `practice`,
`practice..generate` don't → `ManifestError::InvalidIdentifier`.

## 6. Rust type contracts

**How `invoke()` reaches real code** — the piece the original spec's diagram leaves
implicit. Manifests are declarative data; Stage 8 is entirely in-tree/first-party (no
dynamic loading — explicitly excluded), so a module registers as a
`(ModuleManifest, Box<dyn CapabilityProvider>)` pair: the manifest declares *what*, the
provider is the Rust code that does it.

```rust
#[async_trait::async_trait]
pub trait CapabilityProvider: Send + Sync {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, InvocationError>;
}
```

`async fn` now, even though every Stage 8 provider computes and returns immediately — this
is what makes the exit gate (§10) true rather than aspirational. `serde_json::Value` at this
boundary, not a generic, since the registry looks capabilities up dynamically by string id;
each real capability (Practice's `generate`, later) deserializes its own typed `Input` and
serializes its own typed `Output` at the edge, the same pattern
`src-tauri/src/commands/*` already uses for Tauri commands.

```rust
pub enum InvocationError {
    UnknownCapability { capability_id: CapabilityId, version: u32 }, // manifest declared it,
                                                                       // provider doesn't
                                                                       // actually handle it
    InvalidInput { capability_id: CapabilityId, message: String },   // input didn't
                                                                       // deserialize into
                                                                       // the provider's
                                                                       // expected type
    Failed { message: String }, // the provider's own internal failure — a fixture provider
                                 // built purely to test this path returns it deliberately
}
```

A `CapabilityProvider`'s own failure; `RegistryError::InvocationFailed` (§7) wraps it rather
than replacing it, so a caller can see both "which module/capability failed" (registry-level
context) and "why" (this enum).

```rust
pub struct ModuleId(String);          // validated at construction against §5's grammar
pub struct CapabilityId(String);      // validated at construction against §5's grammar

/// Deliberately distinct from src/types/common.ts's OfflineStatus ("Works offline" /
/// "Online enhanced" / "Internet required"), which is human-facing catalog display copy.
/// A manifest is a machine contract and shouldn't couple to UI wording.
#[serde(rename_all = "lowercase")]
pub enum OfflineCapability { Full, Enhanced, Required }

pub struct ModuleManifest {
    pub id: ModuleId,
    pub name: String,
    pub version: semver::Version,
    pub minimum_axiom_version: semver::Version,
    pub offline: OfflineCapability,
    pub provides: Vec<CapabilityDescriptor>,
    pub requires: Vec<CapabilityRequirement>,
}

pub struct CapabilityDescriptor { pub id: CapabilityId, pub version: u32 }
pub struct CapabilityRequirement { pub id: CapabilityId, pub min_version: u32 }

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
    module_id: ModuleId,
    capability_id: CapabilityId,
    version: u32,
} // opaque to callers

pub trait ManifestSource {
    fn discover(&self) -> Result<Vec<(ModuleId, String /* raw toml */)>, ManifestError>;
}
pub struct EmbeddedManifestSource { /* wraps include_str!'d manifests */ }

pub struct ModuleInstallation {
    pub workspace_id: String,
    pub enabled_module_ids: Vec<ModuleId>, // order is resolution priority, per CORE.md §5
}
```

## 7. Error taxonomy

Two enums, split by *when* the failure can be known — a manifest's own validity doesn't
depend on what else is registered, but resolution inherently does.

```rust
pub enum ManifestError {
    UnsupportedManifestVersion { found: u32, supported: &'static [u32] },
    MissingModuleId,
    MalformedVersion { field: &'static str, value: String },
    InvalidIdentifier { value: String },
    DuplicateCapability { module_id: String, capability_id: String }, // two [[provides]]
                                                                        // with the same
                                                                        // id + version
    IncompatibleAxiomVersion { required: semver::Version, running: semver::Version },
                              // well-formed minimum_axiom_version, but newer than this
                              // build -- added during task 046's review, same shape as
                              // UnsupportedManifestVersion (valid content, this runtime
                              // can't accept it); validate() rejects it outright, so no
                              // ModuleManifest value can carry an incompatible version
    TomlSyntax(toml::de::Error),
}

pub enum RegistryError {
    DuplicateModuleId(ModuleId),          // registering an id already registered
    NoCompatibleProvider { capability_id: CapabilityId, min_version: u32 }, // resolve()
                                                                              // found nothing
    ModuleDisabled(ModuleId),             // resolved handle's module isn't enabled here
    InvocationFailed {
        module_id: ModuleId,
        capability_id: CapabilityId,
        cause: InvocationError,
    },
}
```

**Not an error**: multiple modules providing the same capability is the expected
multi-provider case (§3's `ModuleInstallation` ordering resolves it via CORE.md §5's
first-match rule), not a `DuplicateProvider` failure. The source spec's "duplicate-provider
tests" verify correct *resolution behavior*, not rejection.

Syntactic failures (bad TOML, wrong field types) surface as `ManifestError::TomlSyntax`;
semantic failures that are only checkable after a structurally-valid parse (empty id, bad
version string, invalid identifier grammar, an unsupported `manifest_version`, a duplicate
capability within one manifest's own `provides`) get their own specific variants, so error
messages stay legible rather than bottoming out in a generic deserialization error.

## 8. Module lifecycle

```
Discovered  — found by a ManifestSource, not yet parsed
    ↓
Loaded      — parsed + validated successfully, held in the registry
    or
Rejected    — validation failed; error recorded, terminal, does NOT block other modules

(per workspace, only meaningful for Loaded modules)
Enabled / Disabled — via ModuleInstallation.enabled_module_ids
```

One broken `module.toml` never prevents the rest of the bundle from registering.

## 9. Testing plan

| Test class (from source spec) | Concrete shape |
|---|---|
| valid/malformed TOML | `EmbeddedManifestSource` fixture set; malformed → `TomlSyntax` |
| unsupported manifest-version, missing field, invalid identifier, duplicate capability | Own `ManifestError` variant + fixture each |
| duplicate module ID | Two discovered manifests sharing an `id` → `DuplicateModuleId` |
| missing dependency / incompatible version | `resolve()` finds nothing satisfying a requirement → `NoCompatibleProvider` (both cases collapse to "nothing usable was found," which is the honest state either way) |
| duplicate-provider | Two fixtures both provide `practice.generate@1`; asserts `ModuleInstallation` ordering picks the right one, and that reordering changes the winner |
| disabled-module, workspace-isolation | Two `ModuleInstallation` fixtures with different `enabled_module_ids`; same capability resolves in one workspace, not the other |
| invocation failure | A fixture provider that deliberately returns `Err` → `InvocationFailed`, no panic |
| serialization | `ModuleManifest`/`CapabilityCall`/envelope round-trip through `serde_json` with no data loss — what a future Tauri command carrying these types depends on |
| parse → validate → register integration | Full pipeline via `EmbeddedManifestSource` against a fixture set including one deliberately-broken manifest, proving partial-failure isolation |
| contract tests | A reusable conformance harness any module — fixture or real — runs through; the bundled Practice module (task 052, later) must pass the identical suite — the literal proof of "bundled doesn't mean privileged" |

Regression fixtures extend to the runtime itself, per the source spec's own note:
`src-tauri/src/modules/tests/fixtures/` holds both valid and known-bad `module.toml` files,
named by case (`missing-id.toml`, `duplicate-capability.toml`, …), growing permanently as
issues are found.

## 10. Exit gate

**"Claude signs off that a module can theoretically be moved out-of-process later without
changing the public contract."**

`CapabilityProvider::invoke`'s boundary is already `serde_json::Value` in,
`Result<Value, _>` out, `async` — no Rust references, no closures, nothing that only makes
sense in the same process. `ManifestSource` already abstracts *where* a manifest comes from.
The one thing that would need a new implementation for an out-of-process module is
`CapabilityProvider` itself (an IPC/subprocess-backed impl instead of a direct Rust call),
and nothing in its trait signature assumes same-process execution today — which is the
actual test the exit gate is asking for.

## 11. Dependencies

New Rust crates, exact-pinned per this repo's existing `Cargo.toml` convention (`=X.Y.Z`,
not caret ranges): `toml`, `semver`, `async-trait`. Pinned to current stable at
implementation time (046/047).

## 12. Task mapping

- **045 (Claude)** — this design becomes `CORE.md`'s rewrite: from "forward-looking, changes
  nothing today" into Stage 8's active contract, replacing its TS-sketch with the Rust types
  in §6, and resolving its own §5/§7 open questions the way §3 and §9 here already do.
- **046 (Codex)** — `module.toml` parsing/validation: `RawModuleManifest` → `ModuleManifest`,
  §5's identifier grammar, `ManifestSource`/`EmbeddedManifestSource`, `ManifestError`.
- **047 (Codex)** — the registry: `ModuleRegistry`, `CapabilityProvider`,
  `resolve()`/`invoke()`, `ModuleInstallation`, `RegistryError`, §8's lifecycle enforcement.
- **048 (Codex implements, Claude reviews)** — §9's full table, the regression-fixture
  directory, and small labeled test-fixture provider modules (explicitly not real Practice
  logic) to exercise the runtime mechanically.

## 13. Explicitly out of scope here

Everything Stage 8 §8.2 onward names as excluded from Stage 8 itself (Tutor/LLM, mastery
engine, event bus, full visualization, content import, marketplace/downloads, sandboxing,
signing, AI-generated problems) is also out of scope for *this* sub-project by construction
— none of it is reachable until the runtime this document defines exists. Additionally, out
of scope specifically for 045–048, to be picked up by later Stage 8 sub-projects once this
contract is locked:

- Knowledge Package schema, Problem schema, Practice Utility itself, the verification
  capability, deterministic generation, the reference Calc II package, Study Session UI
  integration, and the offline acceptance test (§8.2–8.10 of the source spec).
- `ModuleInstallation` persistence to SQLite (stays in-memory/fixture data for 045–048).
- Reconciling `package.json`/`Cargo.toml`/`tauri.conf.json`'s three independent version
  strings into one source of truth.
