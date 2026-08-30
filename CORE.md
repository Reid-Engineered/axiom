# Axiom Core — the module contract

What "Axiom Core" means, and the contract between it and everything that isn't it. This
document was originally forward-looking, describing a target shape before Stage 8 was
scheduled, the same way `ARCHITECTURE.md` described the whole app before Stage 0 existed.
It is now **Stage 8 sub-project 1's active contract**: tasks 046–048 implement the manifest
format, parser/validator, and capability registry this document defines, against
`docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md`. Its runtime types
are Rust (`src-tauri/src/modules/`) — wherever this document shows a concrete type, that's
what changed from an earlier draft written before the runtime's language was decided.

Scope of this document: the module contract only — what a module is, what it declares, and
how Core invokes it. Three related subsystems are named but deliberately **not** designed
here, each because it's a large enough surface to need its own pass:

- **The event bus** — how capabilities react to other capabilities' activity (a practice
  attempt triggering tutor/mastery/analytics/history updates). Needs the module contract
  settled first, since events are just another kind of message crossing the same boundary.
- **Core's storage abstraction** — how Core persists workspaces/goals/concepts/sessions
  without modules reaching into storage directly. Overlaps with the persistence layer
  Stage 7 built and deserves its own design pass, not scheduled yet.
- **Permissions** — what a module can see or do when Core invokes it. Depends on the
  invocation model this document defines, so it comes after.

---

## 1. The boundary

**Axiom Core** owns workspaces, goals, concepts, sessions, and the module registry itself.
It has no knowledge of any subject or capability's internals — no "Calculus," no "shell
method," no "Socratic method" anywhere in Core code. Core's job is to host capabilities, not
understand them.

**A module** is a bundle that declares capabilities. **A capability** is a structured,
versioned contract, not a category label: `tutoring.socratic@1`, not "Tutor." This mirrors a
pattern already in this codebase — `VisualizationScene` (`src/types/visualization.ts`) models
a 3D scene as plain, verified primitives specifically so a real rendering engine can slot in
later without a page-level rewrite. Capabilities apply the same discipline to *any* module
output, not just visualization.

Instead of Core asking "what type of module is this?", it asks "what capabilities does this
provide, and what does it require?" A module that provides `tutoring.socratic@1` and a module
that provides `tutoring.coach@1` are both just tutoring-shaped modules to Core — it never
special-cases either by name.

## 2. Real isolation, designed for now

Modules run in-process today, inside Core's own Rust process. This document designs as
though they won't always: every value that crosses the Core/module boundary — capability
input, capability output, and (in a future event-bus design) event payloads — must be plain,
structurally-clonable data. No Rust references, no closures, nothing that only makes sense
inside the same address space. A capability that wants to render UI describes *what* to
render as data (again, the `VisualizationScene` pattern); it does not hand Core a component
to mount.

This is the one property every other section in this document exists to protect, and it is
now a real mechanism, not just a rule: `CapabilityProvider::invoke` (§4) takes and returns
`serde_json::Value` over an `async fn`, so the one thing a future out-of-process module would
need is a new `CapabilityProvider` implementation (IPC- or subprocess-backed) — nothing in
today's trait signature assumes same-process execution. See
`docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md` §10 for the full
reasoning.

## 3. Capability descriptor, module manifest, and the `module.toml` format

Every module ships a `module.toml` — the canonical, human-authored representation of a
manifest. Raw TOML never propagates past the parsing boundary; everything downstream works
only with the validated types below.

```toml
manifest_version = 1          # exact-match against a small supported set (currently {1});
                               # anything else is a structured error, not a silent skip

id = "org.axiom.practice"     # required; reverse-DNS-style, validated against the
                               # identifier grammar below
name = "Axiom Practice"       # required; display-only, no semantic meaning to the runtime
version = "0.1.0"             # required; the module's own semver

minimum_axiom_version = "0.8.0"   # required; semver, compared against the running Axiom
                                   # build's own version
offline = "full"                  # enum: "full" | "enhanced" | "required" — a machine
                                   # contract value, deliberately distinct from
                                   # src/types/common.ts's OfflineStatus, which is
                                   # human-facing catalog display copy (§7)

[[provides]]
id = "practice.generate"      # capability id, validated against the identifier grammar
version = 1                   # positive integer, bumped on breaking input/output changes

[[requires]]
id = "knowledge.query"
min_version = 1
```

**Identifier grammar** — both capability IDs and module IDs, dot-segmented, lowercase, at
least two segments:

```
^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$
```

`practice.generate`, `org.axiom.practice` match; `Practice`, `practice`,
`practice..generate` don't.

**Rust types** (`src-tauri/src/modules/`):

```rust
pub struct ModuleId(String);          // validated at construction against the grammar above
pub struct CapabilityId(String);      // validated at construction against the grammar above

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
```

A module declares both what it offers (`provides`) and what it depends on (`requires`) —
this still makes "this tutor module needs a visualization provider" a checkable fact at
registration time instead of a runtime surprise, same as the original draft. What changed
from that draft: `CapabilityDescriptor` no longer carries placeholder `input`/`output`
fields. A struct field can't hold a type, only a value — the original sketch used
`input: unknown` as a way of saying "each capability author defines this concretely later."
The Rust design achieves the same deferral through `CapabilityCall<Input>`'s generic
parameter (§4) and each `CapabilityProvider`'s own typed deserialization at the
`serde_json::Value` boundary, not through fields on the descriptor itself.

**Manifest errors** — split by *when* a failure can be known. A manifest's own validity
doesn't depend on what else is registered; resolution (§4) does.

```rust
pub enum ManifestError {
    UnsupportedManifestVersion { found: u32, supported: &'static [u32] },
    MissingModuleId,
    MalformedVersion { field: &'static str, value: String },
    InvalidIdentifier { value: String },
    DuplicateCapability { module_id: String, capability_id: String }, // two [[provides]]
                                                                        // with the same
                                                                        // id + version,
                                                                        // within ONE manifest
    IncompatibleAxiomVersion { required: semver::Version, running: semver::Version },
                              // well-formed minimum_axiom_version, but newer than this
                              // build -- same shape as UnsupportedManifestVersion (content
                              // is valid, this specific runtime can't accept it), added
                              // during task 046's review after the locked contract turned
                              // out to have no variant for it
    TomlSyntax(toml::de::Error),
}
```

Syntactic failures (bad TOML, wrong field types) surface as `TomlSyntax`; semantic failures
only checkable after a structurally-valid parse get their own specific variant, so error
messages stay legible rather than bottoming out in a generic deserialization error.

`validate()` rejects `IncompatibleAxiomVersion` outright, the same way it rejects
`UnsupportedManifestVersion` outright — there is no path to a `ModuleManifest` value whose
`minimum_axiom_version` exceeds `axiom_version()`. This keeps the compatibility check a pure
function of the manifest plus one external constant (the running build's own version), not a
registration-time concern — it doesn't need to know what else is registered, so it belongs
here, not in `RegistryError` (§4).

**Discovery** is behind an abstraction, not a concrete mechanism:

```rust
pub trait ManifestSource {
    fn discover(&self) -> Result<Vec<(ModuleId, String /* raw toml */)>, ManifestError>;
}
pub struct EmbeddedManifestSource { /* wraps include_str!'d manifests */ }
```

`EmbeddedManifestSource` — all first-party manifests ship embedded via `include_str!` at
compile time, for determinism, matching the precedent SQL migrations already set
(`src-tauri/src/db/migrations/`). `include_str!` is an implementation detail of this one
source, not part of the contract; filesystem and marketplace sources are future
implementations of the same trait, and nothing about the registry needs to change to add
them.

**Lifecycle:**

```
Discovered  — found by a ManifestSource, not yet parsed
    ↓
Loaded      — parsed + validated successfully, held in the registry
    or
Rejected    — validation failed; error recorded, terminal, does NOT block other modules

(per workspace, only meaningful for Loaded modules)
Enabled / Disabled — via ModuleInstallation.enabled_module_ids (§5)
```

One broken `module.toml` never prevents the rest of the bundle from registering.

## 4. The call

Every invocation is a shared envelope plus a capability-specific payload, and per §2, both
must be plain serializable data:

```rust
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
} // fields private -- opaque to callers, only constructible by ModuleRegistry::resolve
```

The envelope is what lets Core generically log, route, and version-check every call without
understanding what's inside `input` — the same separation an HTTP request draws between
headers and body.

**How `invoke()` reaches real code.** Manifests are declarative data; Stage 8 is entirely
in-tree/first-party (no dynamic loading — that's future work), so a module registers as a
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

`async fn` now, even though every Stage 8 provider computes and returns immediately — see
§2's isolation note for why. `serde_json::Value` at this boundary, not a generic, since the
registry looks capabilities up dynamically by string id; each real capability deserializes
its own typed `Input` and serializes its own typed `Output` at the edge, the same pattern
`src-tauri/src/commands/*` already uses for Tauri commands.

Invocation is async request/response, deliberately mirroring `ARCHITECTURE.md` §5 rule 2's
"services are `async` now, on purpose" — a capability call today runs in-process and resolves
immediately, but nothing about its shape changes when a future call is actually IPC- or
subprocess-backed:

```rust
pub struct ModuleRegistry { /* internal: manifests keyed by ModuleId, each paired with its
                                Box<dyn CapabilityProvider> */ }

impl ModuleRegistry {
    /// Returns one Result per manifest a ManifestSource discovers. A Rejected manifest does
    /// NOT prevent the rest from registering.
    pub fn register(
        &mut self,
        manifest: ModuleManifest,
        provider: Box<dyn CapabilityProvider>,
    ) -> Result<ModuleId, RegistryError>; // Err(DuplicateModuleId) if already registered

    pub fn resolve(
        &self,
        installation: &ModuleInstallation,
        requirement: &CapabilityRequirement,
    ) -> Result<CapabilityHandle, RegistryError>; // Err(NoCompatibleProvider) if nothing
                                                    // registered+enabled satisfies it

    pub async fn invoke<Input: Serialize, Output: DeserializeOwned>(
        &self,
        handle: &CapabilityHandle,
        installation: &ModuleInstallation,
        call: CapabilityCall<Input>,
    ) -> Result<Output, RegistryError>;
}

pub enum RegistryError {
    DuplicateModuleId(ModuleId),
    NoCompatibleProvider { capability_id: CapabilityId, min_version: u32 },
    ModuleDisabled(ModuleId), // resolved handle's module isn't enabled here -- a handle
                              // can outlive an enablement change
    InvocationFailed {
        module_id: ModuleId,
        capability_id: CapabilityId,
        cause: InvocationError,
    },
}
```

## 5. Resolving multiple providers

If more than one enabled module in a workspace provides a matching capability at a
sufficient version, `resolve` returns the first match walking the workspace's enabled
module order. This is a deliberate, simple default so the contract is fully specified — not
a claim that it's the right long-term UX. A real user-facing choice (a settings surface for
"which tutor module handles Socratic prompts") is future work, tracked here so it isn't
lost, not designed now.

```rust
pub struct ModuleInstallation {
    pub workspace_id: String,
    pub enabled_module_ids: Vec<ModuleId>, // order IS resolution priority
}
```

For Stage 8 sub-project 1, `ModuleInstallation` is in-memory/test-fixture data — wiring it
to real SQLite persistence is later work, once a workspace actually needs configurable
module installs (Practice/UI integration, a later Stage 8 sub-project).

## 6. First-party modules are third-party modules

Any module Axiom itself ships — Tutor, Visualizer, Practice, CAS, Notes, Review — is built
against exactly this contract, with no back door into Core's internals. If a first-party
module needs something this contract can't express, that's a finding against this document,
worth fixing here, not a private exception for first-party code. The discipline this buys:
Core can't quietly accumulate assumptions that only hold because "we control both sides
today" — assumptions that break the moment a real third-party module shows up.

Enforced, not just stated: task 048 builds a conformance harness any module — fixture or
real — must pass, and the bundled Practice module (task 052, later) runs through the
identical suite, not a bespoke one.

## 7. Relationship to `src/types/module.ts`

Resolved. Three separate types, not two:

| Layer | Type | Lives | Purpose |
|---|---|---|---|
| Catalog | `ModuleMetadata` | `src/types/module.ts` (today's `Module`) | What humans see — marketplace rows, trust badges, screen 10 copy. Backed by the `modules`/`workspace_modules` SQL tables. |
| Runtime contract | `ModuleManifest` | Rust, `src-tauri/src/modules/` | What Core understands — parsed and validated from `module.toml` (§3). |
| Per-workspace config | `ModuleInstallation` | Rust | How a workspace has a module enabled (§5). |

`Module` keeps its current name in `src/types/module.ts` for now — renaming it to
`ModuleMetadata` in code is a separate task, done whenever something actually needs the two
concepts distinguished in practice, not required by this document alone. The `modules`/
`workspace_modules` SQL tables stay exactly as Stage 7 built them; this document doesn't
touch persistence for the catalog layer.

## 8. What "done" looks like for this document

§§1–5's manifest format, types, and registry contract are implemented by tasks 046–048 —
this document is no longer purely aspirational for those sections; a gap between this
document and that code is a finding against whichever is wrong, resolved the way a Stage 6
page task is reviewed against `ARCHITECTURE.md`, not routed around silently.

The three subsystems named in the introduction — the event bus, Core's storage abstraction,
and permissions — stay aspirational until their own design pass gives them the same
treatment §§1–5 just got.
