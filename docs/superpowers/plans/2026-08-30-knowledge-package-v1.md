# Knowledge Package v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust loader/validator for Knowledge Package v1 and migrate the existing `knowledge-package/` Calc II reference content against it.

**Architecture:** A new `src-tauri/src/knowledge/` module, structured like `src-tauri/src/modules/` (raw parse-layer structs → validated domain structs, `identifier.rs`/`error.rs`/entity-specific parser files/`tests/`). One public entry point, `load_knowledge_package(path) -> Result<KnowledgePackage, KnowledgeError>`, performing atomic, offline, deterministic validation per the spec. No capability/module-registry integration, no Canonical Problem, Practice, or `math.verify` code in this plan.

**Tech Stack:** Rust, `toml` and `serde`/`serde_json` (both already exact-pinned in `src-tauri/Cargo.toml`), `semver`. No new crate dependencies.

**Spec:** `docs/superpowers/specs/2026-08-30-knowledge-package-v1-spec.md` (commit `5d3f283`), built on `docs/superpowers/specs/2026-08-30-knowledge-package-v1-design.md`. This plan argues from both; executors should read the spec's relevant section before each task.

## Global Constraints

- Identifier grammar (spec §2): each dot-segment matches `^[a-z][a-z0-9_]*$`; full value MUST match `^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$` — at least two segments, no length limit.
- `schema_version = 1` is the only value a v1 loader accepts (spec §14).
- Package validation is atomic: one malformed/unresolved entity invalidates the whole package, no partial load (spec §12).
- Unknown keys in `package.toml`, `sources.toml`, and all entity frontmatter MUST be rejected (spec §4–§6, §15).
- Every `Concept`/`Objective`/`Example` MUST carry ≥1 `ProvenanceRef` (spec §11).
- `Example.objective_ids` entries MUST reference an `Objective` whose `concept_id` equals the `Example`'s own `concept_id` (spec §9).
- `prerequisite_ids` is directed and MUST form a DAG; `related_ids` is symmetric, authored on at most one side, normalized in memory (spec §10).
- No learner state, problem-generation machinery, verifier configuration, or editorial/review-status fields anywhere in this module (spec §1).
- Offline only: no network, no runtime LLM calls (spec §16).

---

### Task 1: Knowledge error taxonomy and identifier types

**Files:**
- Create: `src-tauri/src/knowledge/error.rs`
- Create: `src-tauri/src/knowledge/identifier.rs`
- Create: `src-tauri/src/knowledge/mod.rs`
- Modify: `src-tauri/src/modules/identifier.rs:35` (widen `validate_identifier` visibility)
- Modify: `src-tauri/src/modules/mod.rs:1` (widen the `identifier` module's own visibility — see Step 1)
- Modify: `src-tauri/src/lib.rs:3` (register the new module)
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/identifier.rs`

**Interfaces:**
- Produces: `KnowledgeError` — starts with exactly one variant, `InvalidIdentifier { value: String }`, the only one this task's own tests exercise. **`KnowledgeError` is not fully declared here.** Tasks 4, 5, 6, 7, 9, 10, and 11 each extend it with only the variants *that task* needs, as part of that task's own "write the implementation" step (each says exactly which ones and shows the exact code) — this keeps every variant introduced in the same task that has a test constructing it, per this plan's own Task Right-Sizing rule ("each task ends with an independently testable deliverable"). No task pre-declares a variant it doesn't itself use. Also produces `KnowledgePackageId`, `ConceptId`, `ObjectiveId`, `ExampleId`, `SourceId`, each with `fn new(value: impl Into<String>) -> Result<Self, KnowledgeError>` and `fn as_str(&self) -> &str`.

`modules::identifier::validate_identifier` currently returns `Result<(), ManifestError>` — a module-domain error type. Knowledge's ID types must not leak that type into their own errors, so this task widens `validate_identifier`'s visibility only (no signature change) and adapts its `ManifestError::InvalidIdentifier` result into `KnowledgeError::InvalidIdentifier` at the call site.

- [ ] **Step 1: Widen `validate_identifier`'s visibility — the function *and* its containing module**

In `src-tauri/src/modules/identifier.rs:35`, change:

```rust
fn validate_identifier(value: &str) -> Result<(), ManifestError> {
```

to:

```rust
pub(crate) fn validate_identifier(value: &str) -> Result<(), ManifestError> {
```

This alone is not sufficient: `src-tauri/src/modules/mod.rs:1` declares `mod identifier;` with default (private) visibility, and Rust module privacy is transitive along the whole path — a private module is reachable only from its defining module and that module's own descendants. `knowledge` is a *sibling* of `modules`, not a descendant, so `crate::modules::identifier::validate_identifier` is unreachable from `knowledge` no matter how visible the function itself is; the path fails at the `identifier` segment (`E0603`). In `src-tauri/src/modules/mod.rs:1`, change:

```rust
mod identifier;
```

to:

```rust
pub(crate) mod identifier;
```

No other change to either file — `modules/mod.rs`'s existing `pub use identifier::{CapabilityId, ModuleId};` re-export, and `ModuleId`/`CapabilityId`'s own public API, are both unaffected. This keeps `use crate::modules::identifier::validate_identifier;` (Step 4 below) working exactly as written, with no import-path change needed anywhere else in this task.

- [ ] **Step 2: Write the failing identifier tests**

Create `src-tauri/src/knowledge/identifier.rs` with just the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identifiers_are_accepted() {
        for valid in ["shell.method_vertical_axis", "org.axiom.calculus_shells", "a.b", "a0.b_1"] {
            assert!(ConceptId::new(valid).is_ok(), "{valid} should be valid");
            assert!(SourceId::new(valid).is_ok(), "{valid} should be valid");
        }
    }

    #[test]
    fn invalid_identifiers_are_rejected() {
        for invalid in [
            "Shell.Method",
            "shell",
            "shell..method",
            ".shell",
            "shell.",
            "shell.Method",
            "shell-method.vertical",
        ] {
            assert!(matches!(
                ConceptId::new(invalid),
                Err(KnowledgeError::InvalidIdentifier { .. })
            ));
        }
    }

    #[test]
    fn distinct_entity_kinds_may_share_a_lexical_value() {
        assert!(ConceptId::new("shell.basic").is_ok());
        assert!(ObjectiveId::new("shell.basic").is_ok());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::identifier`
Expected: FAIL to compile — `KnowledgeError`, `ConceptId`, `SourceId`, `ObjectiveId` not defined yet.

- [ ] **Step 3: Write `KnowledgeError`, one variant**

Create `src-tauri/src/knowledge/error.rs`:

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum KnowledgeError {
    InvalidIdentifier { value: String },
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { value } => write!(f, "invalid Knowledge identifier: {value}"),
        }
    }
}

impl Error for KnowledgeError {}
```

This is deliberately not the final shape of `KnowledgeError`. Tasks 4, 5, 6, 7, 9, 10, and 11 each add the specific variants they need — a `path: PathBuf` field, an `Io` variant with a `source`, `Display`/`Error` impl updates — as part of that task's own implementation step, at the point each is first needed. Do not add `path`, `Io`, or any other field/variant here; `InvalidIdentifier { value: String }` is everything Task 1's tests exercise.

- [ ] **Step 4: Write the identifier wrapper types**

Prepend to `src-tauri/src/knowledge/identifier.rs` (above the `#[cfg(test)]` module already written):

```rust
use serde::{Deserialize, Serialize};

use crate::modules::identifier::validate_identifier;

use super::error::KnowledgeError;

macro_rules! knowledge_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeError> {
                let value = value.into();
                validate_identifier(&value).map_err(|_| KnowledgeError::InvalidIdentifier {
                    value: value.clone(),
                })?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

knowledge_id!(KnowledgePackageId);
knowledge_id!(ConceptId);
knowledge_id!(ObjectiveId);
knowledge_id!(ExampleId);
knowledge_id!(SourceId);
```

A macro is used deliberately here, unlike `modules::identifier`'s two hand-written impls — five near-identical types is past the point where duplication reads as clarity rather than repetition; `modules::identifier` has only two and stays hand-written, which is consistent, not a contradiction.

- [ ] **Step 5: Wire up `mod.rs` and `lib.rs`**

Create `src-tauri/src/knowledge/mod.rs`:

```rust
mod error;
mod identifier;

pub use error::KnowledgeError;
pub use identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};
```

In `src-tauri/src/lib.rs:3`, add after `pub mod modules;`:

```rust
pub mod knowledge;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::identifier && cargo clippy --all-targets --locked -- -D warnings`
Expected: all `knowledge::identifier` tests PASS; clippy clean.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/knowledge/ src-tauri/src/modules/identifier.rs src-tauri/src/lib.rs
git commit -m "feat(knowledge): add error taxonomy and identifier types"
```

---

### Task 2: Domain types

**Files:**
- Create: `src-tauri/src/knowledge/types.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/types.rs`

**Interfaces:**
- Consumes: `ConceptId`, `ObjectiveId`, `ExampleId`, `SourceId`, `KnowledgePackageId` (Task 1).
- Produces: `KnowledgePackage`, `Concept`, `Objective`, `Example`, `Source`, `ProvenanceRef`, `ProvenanceKind`, `SourceLocator` — the validated domain structs every later task builds, resolves references against, or returns.

These are pure data types with no parsing/validation logic — exactly spec §4/§5/§7/§8/§9/§11's shapes, no more, no fewer fields.

- [ ] **Step 1: Write the failing round-trip test**

Create `src-tauri/src/knowledge/types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::identifier::{ConceptId, ExampleId, ObjectiveId, SourceId};

    #[test]
    fn example_round_trips_through_json() {
        let example = Example {
            id: ExampleId::new("shell.example_basic").unwrap(),
            concept_id: ConceptId::new("shell.method_vertical_axis").unwrap(),
            objective_ids: vec![ObjectiveId::new("shell.setup_radius_height").unwrap()],
            problem: "Find the volume...".to_owned(),
            solution: "V = 8pi/3".to_owned(),
            hints: vec!["Identify the radius first.".to_owned()],
            provenance_refs: vec![ProvenanceRef {
                source_id: SourceId::new("src.openstax_calc2").unwrap(),
                locator: Some(SourceLocator {
                    section: Some("2.3".to_owned()),
                    pages: None,
                    label: Some("Example 2.13".to_owned()),
                }),
                kind: ProvenanceKind::Direct,
            }],
        };

        let json = serde_json::to_string(&example).unwrap();
        let round_tripped: Example = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, example);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::types`
Expected: FAIL to compile — none of the domain types exist yet.

- [ ] **Step 3: Write the domain types**

Prepend to `src-tauri/src/knowledge/types.rs`:

```rust
use semver::Version;
use serde::{Deserialize, Serialize};

use super::identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgePackage {
    pub id: KnowledgePackageId,
    pub schema_version: u32,
    pub version: Version,
    pub title: String,
    pub description: String,
    pub concepts: Vec<Concept>,
    pub objectives: Vec<Objective>,
    pub examples: Vec<Example>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    pub id: ConceptId,
    pub name: String,
    pub topic: Option<String>,
    pub description: String,
    pub prerequisite_ids: Vec<ConceptId>,
    pub related_ids: Vec<ConceptId>,
    pub provenance_refs: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub id: ObjectiveId,
    pub concept_id: ConceptId,
    pub description: String,
    pub provenance_refs: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Example {
    pub id: ExampleId,
    pub concept_id: ConceptId,
    pub objective_ids: Vec<ObjectiveId>,
    pub problem: String,
    pub solution: String,
    pub hints: Vec<String>,
    pub provenance_refs: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub title: String,
    pub authors: Vec<String>,
    pub edition: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProvenanceKind {
    Direct,
    Derived,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRef {
    pub source_id: SourceId,
    pub locator: Option<SourceLocator>,
    pub kind: ProvenanceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocator {
    pub section: Option<String>,
    pub pages: Option<String>,
    pub label: Option<String>,
}
```

`KnowledgePackage` carries `concepts`/`objectives`/`examples`/`sources` as `Vec`s — this is the *validated, loaded* representation (§13 step 12 of the spec: "produce the typed validated KnowledgePackage value"), distinct from `package.toml` itself, which never hand-authors these inventories (spec §3, §4). Task 9 (discovery) and Task 12 (loader) are what actually populate these fields from disk.

- [ ] **Step 4: Re-export from `mod.rs`**

In `src-tauri/src/knowledge/mod.rs`, replace the file with:

```rust
mod error;
mod identifier;
mod types;

pub use error::KnowledgeError;
pub use identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};
pub use types::{
    Concept, Example, KnowledgePackage, Objective, ProvenanceKind, ProvenanceRef, Source,
    SourceLocator,
};
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge:: && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/types.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): add domain types"
```

---

### Task 3: Raw serialization structs

**Files:**
- Create: `src-tauri/src/knowledge/raw.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/raw.rs`

**Interfaces:**
- Produces: `RawKnowledgePackage`, `RawSourcesFile`, `RawSource`, `RawConceptFrontmatter`, `RawObjectiveFrontmatter`, `RawExampleFrontmatter`, `RawProvenanceRef`, `RawSourceLocator` — parse-layer-only structs. Never exported from `mod.rs`; consumed only by Tasks 4, 6, 8. Carries a temporary `#![allow(dead_code)]` (see Step 3) since this task's own tests don't exercise every struct/field and no production consumer exists yet — Task 8 removes it once all three consuming tasks are done.

Mirrors `modules::manifest::RawModuleManifest`'s pattern exactly: raw structs are plain `Deserialize`, string-typed where the domain type is a validated wrapper, and never propagate past the parse boundary (spec §1 constraint: "raw form never propagates past this boundary"). `#[serde(deny_unknown_fields)]` is what mechanically enforces spec §4/§5/§6's "unknown keys MUST be rejected" — a TOML deserialization simply fails if an unrecognized key is present, folding that requirement into the existing `TomlSyntax` error path rather than needing hand-rolled key-checking.

- [ ] **Step 1: Write the failing unknown-key-rejection test**

Create `src-tauri/src/knowledge/raw.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_top_level_key_is_rejected() {
        let toml = r#"
            id = "org.axiom.calculus_shells"
            schema_version = 1
            version = "1.0.0"
            title = "Cylindrical Shells"
            description = "A tiny reference package."
            unexpected = "typo"
        "#;
        assert!(toml::from_str::<RawKnowledgePackage>(toml).is_err());
    }

    #[test]
    fn valid_package_toml_parses() {
        let toml = r#"
            id = "org.axiom.calculus_shells"
            schema_version = 1
            version = "1.0.0"
            title = "Cylindrical Shells"
            description = "A tiny reference package."
        "#;
        let raw: RawKnowledgePackage = toml::from_str(toml).unwrap();
        assert_eq!(raw.id, "org.axiom.calculus_shells");
        assert_eq!(raw.schema_version, 1);
    }

    #[test]
    fn concept_frontmatter_defaults_relationship_fields_to_empty() {
        let toml = r#"
            id = "shell.method_vertical_axis"
            name = "The Method of Cylindrical Shells"
        "#;
        let raw: RawConceptFrontmatter = toml::from_str(toml).unwrap();
        assert!(raw.prerequisite_ids.is_empty());
        assert!(raw.related_ids.is_empty());
        assert!(raw.provenance_refs.is_empty());
    }

    #[test]
    fn source_authors_default_to_empty() {
        let toml = r#"
            id = "src.axiom_original"
            title = "Axiom Original Content"
        "#;
        let raw: RawSource = toml::from_str(toml).unwrap();
        assert!(raw.authors.is_empty());
        assert!(raw.edition.is_none());
        assert!(raw.license.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::raw`
Expected: FAIL to compile — none of the raw structs exist yet.

- [ ] **Step 3: Write the raw structs**

This is the first task whose `pub(crate)` items have no production caller yet — these eight
structs are consumed by Tasks 4, 6, and 8, not this one, and Task 3's own tests only
construct three of them (`RawKnowledgePackage`, `RawConceptFrontmatter`, `RawSource`),
reading only some of each one's fields. `pub(crate)` items are genuinely subject to
dead-code analysis (unlike the `pub`, publicly-re-exported items Tasks 1–2 produced, which
are exempt because the compiler can't prove a public API path is unreachable) — so
`cargo clippy --all-targets --locked -- -D warnings` will fail here with `dead_code`/"field
is never read" errors on both the lib and lib-test targets. This is not specific to Task 3:
it recurs for every task through Task 11, each of which produces `pub(crate)` functions or
types whose only real caller doesn't land until a *later* task (mostly Task 9 for Tasks 6/8's
parsers, Task 12 for almost everything else). Handling this per-file, per-task would mean
juggling several separate `#[allow(...)]`s with separate removal points — fragile, and easy
to under- or over-scope. Instead, Task 3 adds one suppression at the `knowledge` module
root, covering every descendant module for the whole incremental build-out; Task 12 removes
it once the loader wires every `pub(crate)` item to a real caller and the lint is satisfied
on its own. Do not export any of these types, and do not implement Tasks 4/6/8 early, to
work around this instead.

Add this as the first line of `src-tauri/src/knowledge/mod.rs` (Step 4 below adds the rest
of this task's wiring below it):

```rust
// Several tasks in this plan produce pub(crate) items with no production caller until a
// later task (see docs/superpowers/plans/2026-08-30-knowledge-package-v1.md Task 3 Step 3
// for the full reasoning). This is removed in Task 12, once the loader wires everything
// together and every item has a real caller.
#![allow(dead_code)]
```

Then write `src-tauri/src/knowledge/raw.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawKnowledgePackage {
    pub id: String,
    pub schema_version: u32,
    pub version: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSourcesFile {
    #[serde(default)]
    pub sources: Vec<RawSource>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSource {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub edition: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawConceptFrontmatter {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub prerequisite_ids: Vec<String>,
    #[serde(default)]
    pub related_ids: Vec<String>,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawObjectiveFrontmatter {
    pub id: String,
    pub concept_id: String,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawExampleFrontmatter {
    pub id: String,
    pub concept_id: String,
    #[serde(default)]
    pub objective_ids: Vec<String>,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProvenanceRef {
    pub source_id: String,
    #[serde(default)]
    pub locator: Option<RawSourceLocator>,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSourceLocator {
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub pages: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}
```

`provenance_refs` itself defaults to an empty `Vec` at this parse layer deliberately — the *semantic* "MUST have at least one" rule (spec §11) is a validation-layer concern (Task 12), not a parse-layer one, exactly mirroring how `modules::manifest`'s `manifest_version` is accepted as any `u32` at parse time and only checked against `SUPPORTED_MANIFEST_VERSIONS` afterward.

- [ ] **Step 4: Register the module**

Replace `src-tauri/src/knowledge/mod.rs` in full:

```rust
// Several tasks in this plan produce pub(crate) items with no production caller until a
// later task (see docs/superpowers/plans/2026-08-30-knowledge-package-v1.md Task 3 Step 3
// for the full reasoning). This is removed in Task 12, once the loader wires everything
// together and every item has a real caller.
#![allow(dead_code)]

mod error;
mod identifier;
mod raw;
mod types;

pub use error::KnowledgeError;
pub use identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};
pub use types::{
    Concept, Example, KnowledgePackage, Objective, ProvenanceKind, ProvenanceRef, Source,
    SourceLocator,
};
```

No `pub use` for `raw` — raw types stay crate-internal, per spec §1.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge:: && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS (8 tests: the 4 from Tasks 1–2 plus this task's 4); clippy clean — the
`#![allow(dead_code)]` just added is what makes it clean despite these structs having no
production caller yet.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/raw.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): add raw parse-layer structs"
```

---

### Task 4: `package.toml` / `sources.toml` parsing and validation

**Files:**
- Create: `src-tauri/src/knowledge/package.rs`
- Modify: `src-tauri/src/knowledge/error.rs` (add `TomlSyntax`, `UnsupportedSchemaVersion`, `MalformedVersion`, `EmptyField`, `DuplicateSourceId`)
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/package.rs`

**Interfaces:**
- Consumes: `RawKnowledgePackage`, `RawSourcesFile`, `RawSource` (Task 3); `KnowledgePackageId`, `SourceId` (Task 1); `Source` (Task 2); `KnowledgeError` (Task 1, extended by this task — see Step 3).
- Produces: `pub(crate) fn parse_package_toml(raw_toml: &str) -> Result<PackageIdentity, KnowledgeError>` where `PackageIdentity { id: KnowledgePackageId, schema_version: u32, version: semver::Version, title: String, description: String }`; `pub(crate) fn parse_sources_toml(raw_toml: &str) -> Result<Vec<Source>, KnowledgeError>`. Task 12 (loader) calls both and assembles the final `KnowledgePackage`.

`PackageIdentity` exists because `KnowledgePackage` (Task 2) also carries `concepts`/`objectives`/`examples`/`sources`, which this task doesn't have yet — Task 12 combines `PackageIdentity` with the results of Tasks 9–11 to build the final value.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/package.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PACKAGE: &str = r#"
        id = "org.axiom.calculus_shells"
        schema_version = 1
        version = "1.0.0"
        title = "Cylindrical Shells"
        description = "A tiny reference package."
    "#;

    #[test]
    fn valid_package_toml_parses_into_identity() {
        let identity = parse_package_toml(VALID_PACKAGE).unwrap();
        assert_eq!(identity.id.as_str(), "org.axiom.calculus_shells");
        assert_eq!(identity.schema_version, 1);
        assert_eq!(identity.version, semver::Version::new(1, 0, 0));
        assert_eq!(identity.title, "Cylindrical Shells");
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let toml = VALID_PACKAGE.replace("schema_version = 1", "schema_version = 2");
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::UnsupportedSchemaVersion { found: 2 })
        ));
    }

    #[test]
    fn missing_schema_version_is_a_structural_failure() {
        let toml = r#"
            id = "org.axiom.calculus_shells"
            version = "1.0.0"
            title = "Cylindrical Shells"
            description = "A tiny reference package."
        "#;
        assert!(matches!(
            parse_package_toml(toml),
            Err(KnowledgeError::TomlSyntax { .. })
        ));
    }

    #[test]
    fn non_integer_schema_version_is_a_structural_failure() {
        let toml = VALID_PACKAGE.replace("schema_version = 1", r#"schema_version = "1""#);
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::TomlSyntax { .. })
        ));
    }

    #[test]
    fn malformed_package_version_is_rejected() {
        let toml = VALID_PACKAGE.replace(r#"version = "1.0.0""#, r#"version = "not-semver""#);
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::MalformedVersion { field: "version", .. })
        ));
    }

    #[test]
    fn empty_title_is_rejected() {
        let toml = VALID_PACKAGE.replace(r#"title = "Cylindrical Shells""#, r#"title = """#);
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::EmptyField { field: "title", .. })
        ));
    }

    #[test]
    fn duplicate_source_ids_are_rejected() {
        let toml = r#"
            [[sources]]
            id = "src.openstax_calc2"
            title = "Calculus Volume 2"

            [[sources]]
            id = "src.openstax_calc2"
            title = "Calculus Volume 2 (duplicate)"
        "#;
        assert!(matches!(
            parse_sources_toml(toml),
            Err(KnowledgeError::DuplicateSourceId { .. })
        ));
    }

    #[test]
    fn sources_toml_may_declare_zero_sources() {
        assert_eq!(parse_sources_toml("").unwrap().len(), 0);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::package`
Expected: FAIL to compile — `parse_package_toml`, `parse_sources_toml`, `PackageIdentity` not defined yet.

- [ ] **Step 3: Extend `KnowledgeError`, then write the implementation**

This task is the first to need path context and TOML-syntax wrapping. Replace `src-tauri/src/knowledge/error.rs` in full:

```rust
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum KnowledgeError {
    InvalidIdentifier {
        value: String,
    },
    TomlSyntax {
        path: PathBuf,
        source: toml::de::Error,
    },
    UnsupportedSchemaVersion {
        found: u32,
    },
    MalformedVersion {
        field: &'static str,
        value: String,
    },
    EmptyField {
        path: PathBuf,
        field: &'static str,
    },
    DuplicateSourceId {
        id: String,
    },
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { value } => write!(f, "invalid Knowledge identifier: {value}"),
            Self::TomlSyntax { path, source } => write!(f, "invalid TOML in {}: {source}", path.display()),
            Self::UnsupportedSchemaVersion { found } => write!(f, "unsupported schema_version {found}; only 1 is accepted"),
            Self::MalformedVersion { field, value } => write!(f, "{field} is not a semantic version: {value}"),
            Self::EmptyField { path, field } => write!(f, "{} has an empty required field: {field}", path.display()),
            Self::DuplicateSourceId { id } => write!(f, "duplicate Source id: {id}"),
        }
    }
}

impl Error for KnowledgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TomlSyntax { source, .. } => Some(source),
            _ => None,
        }
    }
}
```

Then prepend to `src-tauri/src/knowledge/package.rs`:

```rust
use std::collections::HashSet;

use semver::Version;

use super::error::KnowledgeError;
use super::identifier::{KnowledgePackageId, SourceId};
use super::raw::{RawKnowledgePackage, RawSource, RawSourcesFile};
use super::types::Source;

const SUPPORTED_SCHEMA_VERSION: u32 = 1;

pub(crate) struct PackageIdentity {
    pub id: KnowledgePackageId,
    pub schema_version: u32,
    pub version: Version,
    pub title: String,
    pub description: String,
}

pub(crate) fn parse_package_toml(raw_toml: &str) -> Result<PackageIdentity, KnowledgeError> {
    let raw: RawKnowledgePackage =
        toml::from_str(raw_toml).map_err(|source| KnowledgeError::TomlSyntax {
            path: "package.toml".into(),
            source,
        })?;

    if raw.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(KnowledgeError::UnsupportedSchemaVersion {
            found: raw.schema_version,
        });
    }

    let id = KnowledgePackageId::new(raw.id)?;
    let version = Version::parse(&raw.version).map_err(|_| KnowledgeError::MalformedVersion {
        field: "version",
        value: raw.version,
    })?;

    if raw.title.trim().is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: "package.toml".into(),
            field: "title",
        });
    }
    if raw.description.trim().is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: "package.toml".into(),
            field: "description",
        });
    }

    Ok(PackageIdentity {
        id,
        schema_version: raw.schema_version,
        version,
        title: raw.title,
        description: raw.description,
    })
}

pub(crate) fn parse_sources_toml(raw_toml: &str) -> Result<Vec<Source>, KnowledgeError> {
    let raw: RawSourcesFile =
        toml::from_str(raw_toml).map_err(|source| KnowledgeError::TomlSyntax {
            path: "sources.toml".into(),
            source,
        })?;

    let mut seen_ids = HashSet::new();
    let mut sources = Vec::with_capacity(raw.sources.len());
    for RawSource {
        id,
        title,
        authors,
        edition,
        license,
    } in raw.sources
    {
        let id = SourceId::new(id)?;
        if !seen_ids.insert(id.clone()) {
            return Err(KnowledgeError::DuplicateSourceId {
                id: id.as_str().to_owned(),
            });
        }
        sources.push(Source {
            id,
            title,
            authors,
            edition,
            license,
        });
    }
    Ok(sources)
}
```

- [ ] **Step 4: Register the module**

In `src-tauri/src/knowledge/mod.rs`, add `mod package;` after `mod raw;`. No `pub use` — `PackageIdentity`, `parse_package_toml`, `parse_sources_toml` stay crate-internal.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::package && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/package.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): parse and validate package.toml and sources.toml"
```

---

### Task 5: TOML frontmatter splitter

**Files:**
- Create: `src-tauri/src/knowledge/frontmatter.rs`
- Modify: `src-tauri/src/knowledge/error.rs` (add `Bom`, `MissingFrontmatterDelimiter`, `UnterminatedFrontmatter`)
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/frontmatter.rs`

**Interfaces:**
- Produces: `pub(crate) fn split_frontmatter(path: &Path, raw: &str) -> Result<(String, String), KnowledgeError>` — returns `(toml_text, body_text)`, both owned (CRLF normalization requires allocating). Consumed by Tasks 6 and 8 (Concept/Objective/Example parsing), each of which separately `toml::from_str`s the first string into its own `Raw*Frontmatter` type and treats the second as Markdown body text.

This is the one piece of genuinely custom text-scanning in the whole module — spec §6's exact rules, no general Markdown/frontmatter library.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/frontmatter.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn p() -> &'static Path {
        Path::new("concepts/shell.method_vertical_axis.md")
    }

    #[test]
    fn splits_toml_and_body() {
        let raw = "+++\nid = \"shell.a\"\n+++\n\nBody text.\n";
        let (toml, body) = split_frontmatter(p(), raw).unwrap();
        assert_eq!(toml, "id = \"shell.a\"\n");
        assert_eq!(body, "Body text.\n");
    }

    #[test]
    fn trims_leading_blank_lines_from_body_but_preserves_internal_blank_lines() {
        let raw = "+++\nid = \"shell.a\"\n+++\n\n\nFirst paragraph.\n\nSecond paragraph.\n";
        let (_, body) = split_frontmatter(p(), raw).unwrap();
        assert_eq!(body, "First paragraph.\n\nSecond paragraph.\n");
    }

    #[test]
    fn accepts_crlf_line_endings() {
        let raw = "+++\r\nid = \"shell.a\"\r\n+++\r\n\r\nBody.\r\n";
        let (toml, body) = split_frontmatter(p(), raw).unwrap();
        assert_eq!(toml, "id = \"shell.a\"\n");
        assert_eq!(body, "Body.\n");
    }

    #[test]
    fn rejects_bom() {
        let raw = "\u{FEFF}+++\nid = \"shell.a\"\n+++\n\nBody.\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::Bom { .. })
        ));
    }

    #[test]
    fn rejects_missing_opening_delimiter() {
        let raw = "id = \"shell.a\"\n+++\n\nBody.\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::MissingFrontmatterDelimiter { .. })
        ));
    }

    #[test]
    fn rejects_leading_blank_line_before_opening_delimiter() {
        let raw = "\n+++\nid = \"shell.a\"\n+++\n\nBody.\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::MissingFrontmatterDelimiter { .. })
        ));
    }

    #[test]
    fn rejects_unterminated_frontmatter() {
        let raw = "+++\nid = \"shell.a\"\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::UnterminatedFrontmatter { .. })
        ));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::frontmatter`
Expected: FAIL to compile — `split_frontmatter` not defined yet.

- [ ] **Step 3: Extend `KnowledgeError`, then write the implementation**

Add three variants to the `KnowledgeError` enum in `src-tauri/src/knowledge/error.rs` (after `InvalidIdentifier`, alongside Task 4's other additions), and a matching arm to `impl fmt::Display`:

```rust
    Bom {
        path: PathBuf,
    },
    MissingFrontmatterDelimiter {
        path: PathBuf,
    },
    UnterminatedFrontmatter {
        path: PathBuf,
    },
```

```rust
            Self::Bom { path } => write!(f, "{} starts with a byte-order mark, which is rejected", path.display()),
            Self::MissingFrontmatterDelimiter { path } => write!(f, "{} does not start with the '+++' frontmatter delimiter", path.display()),
            Self::UnterminatedFrontmatter { path } => write!(f, "{} opens frontmatter with '+++' but never closes it", path.display()),
```

Then prepend to `src-tauri/src/knowledge/frontmatter.rs`:

```rust
use std::path::Path;

use super::error::KnowledgeError;

const DELIMITER: &str = "+++";

pub(crate) fn split_frontmatter(
    path: &Path,
    raw: &str,
) -> Result<(String, String), KnowledgeError> {
    if raw.starts_with('\u{FEFF}') {
        return Err(KnowledgeError::Bom {
            path: path.to_owned(),
        });
    }

    let normalized = raw.replace("\r\n", "\n");
    // split_terminator, not split: `split('\n')` on a newline-terminated string yields a
    // trailing empty element (the input ends in the delimiter, not just contains it), which
    // survives into `remaining`, gets folded into `body_text` below, and then collides with
    // the `+ "\n"` appended after it — producing two trailing newlines instead of one.
    // split_terminator drops that spurious trailing element.
    let mut lines = normalized.split_terminator('\n');

    match lines.next() {
        Some(DELIMITER) => {}
        _ => {
            return Err(KnowledgeError::MissingFrontmatterDelimiter {
                path: path.to_owned(),
            })
        }
    }

    let mut toml_lines: Vec<&str> = Vec::new();
    let mut closed = false;
    let mut remaining: Vec<&str> = Vec::new();
    for line in lines.by_ref() {
        if line == DELIMITER {
            closed = true;
            break;
        }
        toml_lines.push(line);
    }
    if !closed {
        return Err(KnowledgeError::UnterminatedFrontmatter {
            path: path.to_owned(),
        });
    }
    remaining.extend(lines);

    let mut body_start = 0;
    while body_start < remaining.len() && remaining[body_start].trim().is_empty() {
        body_start += 1;
    }

    let toml_text = toml_lines.join("\n") + "\n";
    let body_text = if body_start >= remaining.len() {
        String::new()
    } else {
        remaining[body_start..].join("\n") + "\n"
    };

    Ok((toml_text, body_text))
}
```

Callers pass `path` purely for error context — the function itself never touches the filesystem, keeping it independently unit-testable with in-memory strings, matching this task's own test style above.

- [ ] **Step 4: Register the module**

In `src-tauri/src/knowledge/mod.rs`, add `mod frontmatter;` after `mod package;`. No `pub use`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::frontmatter && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/frontmatter.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): add bounded TOML frontmatter splitter"
```

---

### Task 6: `Concept` and `Objective` entity parsers

**Files:**
- Create: `src-tauri/src/knowledge/concept.rs`
- Create: `src-tauri/src/knowledge/objective.rs`
- Create: `src-tauri/src/knowledge/provenance.rs`
- Modify: `src-tauri/src/knowledge/error.rs` (add `MissingProvenance`, `DuplicateProvenanceRef`, `UnknownProvenanceKind`, `EmptySourceLocator`; reuses Task 4's `EmptyField`)
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in each new file

**Interfaces:**
- Consumes: `split_frontmatter` (Task 5); `RawConceptFrontmatter`, `RawObjectiveFrontmatter`, `RawProvenanceRef`, `RawSourceLocator` (Task 3); `Concept`, `Objective`, `ProvenanceRef`, `ProvenanceKind`, `SourceLocator` (Task 2); `ConceptId`, `ObjectiveId`, `SourceId` (Task 1).
- Produces: `pub(crate) fn parse_concept_file(path: &Path, raw: &str) -> Result<Concept, KnowledgeError>`; `pub(crate) fn parse_objective_file(path: &Path, raw: &str) -> Result<Objective, KnowledgeError>`. Both used by Task 9 (discovery). Neither resolves cross-entity references yet (`concept_id` pointing at a real `Concept`, `provenance_refs[].source_id` pointing at a real `Source`) — that's Task 10; this task only produces a well-formed, internally-valid single entity.

Both entities share one non-trivial piece of logic — parsing `RawProvenanceRef`/`RawSourceLocator` into `ProvenanceRef`/`SourceLocator`, including the `kind` string → `ProvenanceKind` mapping, the "non-empty locator" rule, and exact-duplicate-ref rejection (spec §11) — so it's factored into one shared `convert_provenance_refs` function both files call, rather than duplicated. `SourceId` resolution (whether a `source_id` actually names a `Source` that exists) is deliberately *not* done here — it needs the whole package's `Source` list, which this task doesn't have; that's Task 10.

- [ ] **Step 1: Write the failing Concept tests**

Create `src-tauri/src/knowledge/concept.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ProvenanceKind is only needed here, in this test module, not by
    // parse_concept_file's own body — imported here rather than at the file's outer
    // scope, which would trigger unused_imports on a non-test build (the #[cfg(test)]
    // module that actually uses it doesn't exist in that compilation).
    use crate::knowledge::ProvenanceKind;

    const VALID_CONCEPT: &str = r#"+++
id = "shell.method_vertical_axis"
name = "The Method of Cylindrical Shells"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = ["shell.method_horizontal_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"
+++

A method for calculating the volume of a solid of revolution.
"#;

    #[test]
    fn valid_concept_parses() {
        let concept = parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), VALID_CONCEPT).unwrap();
        assert_eq!(concept.id.as_str(), "shell.method_vertical_axis");
        assert_eq!(concept.related_ids.len(), 1);
        assert_eq!(concept.description, "A method for calculating the volume of a solid of revolution.\n");
        assert_eq!(concept.provenance_refs.len(), 1);
        assert_eq!(concept.provenance_refs[0].kind, ProvenanceKind::Direct);
    }

    #[test]
    fn empty_description_is_rejected() {
        let raw = VALID_CONCEPT.replace("A method for calculating the volume of a solid of revolution.\n", "");
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::EmptyField { field: "description", .. })
        ));
    }

    #[test]
    fn missing_provenance_is_rejected() {
        let raw = VALID_CONCEPT.replace(
            "[[provenance_refs]]\nsource_id = \"src.openstax_calc2\"\nkind = \"direct\"\n[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n",
            "",
        );
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::MissingProvenance { .. })
        ));
    }

    #[test]
    fn unknown_provenance_kind_is_rejected() {
        let raw = VALID_CONCEPT.replace(r#"kind = "direct""#, r#"kind = "inferred""#);
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::UnknownProvenanceKind { .. })
        ));
    }

    #[test]
    fn empty_locator_table_is_rejected() {
        let raw = VALID_CONCEPT.replace(
            "[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n",
            "[provenance_refs.locator]\n",
        );
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::EmptySourceLocator { .. })
        ));
    }

    #[test]
    fn exact_duplicate_provenance_ref_is_rejected() {
        let raw = VALID_CONCEPT.replace(
            "+++\n\nA method",
            "\n[[provenance_refs]]\nsource_id = \"src.openstax_calc2\"\nkind = \"direct\"\n[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n+++\n\nA method",
        );
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::DuplicateProvenanceRef { .. })
        ));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::concept`
Expected: FAIL to compile — `parse_concept_file` not defined yet.

- [ ] **Step 3: Extend `KnowledgeError`, then write the shared provenance-conversion helper**

Add four variants to `KnowledgeError` in `src-tauri/src/knowledge/error.rs` (this task reuses Task 4's `EmptyField` for the empty-description check below — do not add a second variant for that):

```rust
    MissingProvenance {
        entity_id: String,
    },
    DuplicateProvenanceRef {
        entity_id: String,
        source_id: String,
    },
    UnknownProvenanceKind {
        entity_id: String,
        value: String,
    },
    EmptySourceLocator {
        entity_id: String,
    },
```

```rust
            Self::MissingProvenance { entity_id } => write!(f, "{entity_id} has no provenance_refs; at least one is required"),
            Self::DuplicateProvenanceRef { entity_id, source_id } => write!(f, "{entity_id} declares a duplicate provenance reference to {source_id}"),
            Self::UnknownProvenanceKind { entity_id, value } => write!(f, "{entity_id} declares unknown ProvenanceKind: {value}"),
            Self::EmptySourceLocator { entity_id } => write!(f, "{entity_id} declares a provenance locator with no section, pages, or label"),
```

Then create `src-tauri/src/knowledge/provenance.rs`:

```rust
use super::error::KnowledgeError;
use super::identifier::SourceId;
use super::raw::{RawProvenanceRef, RawSourceLocator};
use super::types::{ProvenanceKind, ProvenanceRef, SourceLocator};

pub(crate) fn convert_provenance_refs(
    entity_id: &str,
    raw_refs: Vec<RawProvenanceRef>,
) -> Result<Vec<ProvenanceRef>, KnowledgeError> {
    if raw_refs.is_empty() {
        return Err(KnowledgeError::MissingProvenance {
            entity_id: entity_id.to_owned(),
        });
    }

    let refs: Vec<ProvenanceRef> = raw_refs
        .into_iter()
        .map(|raw| convert_one(entity_id, raw))
        .collect::<Result<_, _>>()?;

    for i in 0..refs.len() {
        for j in (i + 1)..refs.len() {
            if refs[i].source_id == refs[j].source_id
                && refs[i].locator == refs[j].locator
                && refs[i].kind == refs[j].kind
            {
                return Err(KnowledgeError::DuplicateProvenanceRef {
                    entity_id: entity_id.to_owned(),
                    source_id: refs[i].source_id.as_str().to_owned(),
                });
            }
        }
    }

    Ok(refs)
}

fn convert_one(entity_id: &str, raw: RawProvenanceRef) -> Result<ProvenanceRef, KnowledgeError> {
    let source_id = SourceId::new(raw.source_id)?;
    let kind = match raw.kind.as_str() {
        "direct" => ProvenanceKind::Direct,
        "derived" => ProvenanceKind::Derived,
        other => {
            return Err(KnowledgeError::UnknownProvenanceKind {
                entity_id: entity_id.to_owned(),
                value: other.to_owned(),
            })
        }
    };
    let locator = match raw.locator {
        None => None,
        Some(RawSourceLocator {
            section,
            pages,
            label,
        }) => {
            if section.is_none() && pages.is_none() && label.is_none() {
                return Err(KnowledgeError::EmptySourceLocator {
                    entity_id: entity_id.to_owned(),
                });
            }
            Some(SourceLocator {
                section,
                pages,
                label,
            })
        }
    };

    Ok(ProvenanceRef {
        source_id,
        locator,
        kind,
    })
}
```

- [ ] **Step 4: Write the Concept parser**

Prepend to `src-tauri/src/knowledge/concept.rs`:

```rust
use std::path::Path;

use super::error::KnowledgeError;
use super::frontmatter::split_frontmatter;
use super::identifier::ConceptId;
use super::provenance::convert_provenance_refs;
use super::raw::RawConceptFrontmatter;
use super::types::Concept;

pub(crate) fn parse_concept_file(path: &Path, raw: &str) -> Result<Concept, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let raw_frontmatter: RawConceptFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;

    let id = ConceptId::new(raw_frontmatter.id)?;
    let prerequisite_ids = raw_frontmatter
        .prerequisite_ids
        .into_iter()
        .map(ConceptId::new)
        .collect::<Result<Vec<_>, _>>()?;
    let related_ids = raw_frontmatter
        .related_ids
        .into_iter()
        .map(ConceptId::new)
        .collect::<Result<Vec<_>, _>>()?;

    let description = body.trim().to_owned();
    if description.is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: path.to_owned(),
            field: "description",
        });
    }

    let provenance_refs = convert_provenance_refs(id.as_str(), raw_frontmatter.provenance_refs)?;

    Ok(Concept {
        id,
        name: raw_frontmatter.name,
        topic: raw_frontmatter.topic,
        description: body,
        prerequisite_ids,
        related_ids,
        provenance_refs,
    })
}
```

`description` is stored as the full trimmed `body` (preserving internal Markdown structure per spec §7), while the separately-computed `description` local variable exists only to check emptiness before constructing the `Concept` — this mirrors `modules::manifest::validate`'s pattern of validating a value before moving it into the final struct.

- [ ] **Step 5: Write the failing Objective tests**

Create `src-tauri/src/knowledge/objective.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const VALID_OBJECTIVE: &str = r#"+++
id = "shell.setup_radius_height"
concept_id = "shell.method_vertical_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"
+++

Identify and express the shell radius and shell height.
"#;

    #[test]
    fn valid_objective_parses() {
        let objective = parse_objective_file(Path::new("objectives/shell.setup_radius_height.md"), VALID_OBJECTIVE).unwrap();
        assert_eq!(objective.id.as_str(), "shell.setup_radius_height");
        assert_eq!(objective.concept_id.as_str(), "shell.method_vertical_axis");
        assert_eq!(objective.provenance_refs.len(), 1);
    }

    #[test]
    fn missing_provenance_is_rejected() {
        let raw = VALID_OBJECTIVE.replace(
            "[[provenance_refs]]\nsource_id = \"src.openstax_calc2\"\nkind = \"direct\"\n[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n",
            "",
        );
        assert!(matches!(
            parse_objective_file(Path::new("objectives/shell.setup_radius_height.md"), &raw),
            Err(KnowledgeError::MissingProvenance { .. })
        ));
    }
}
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::objective`
Expected: FAIL to compile — `parse_objective_file` not defined yet.

- [ ] **Step 7: Write the Objective parser**

Prepend to `src-tauri/src/knowledge/objective.rs`:

```rust
use std::path::Path;

use super::error::KnowledgeError;
use super::frontmatter::split_frontmatter;
use super::identifier::ConceptId;
use super::provenance::convert_provenance_refs;
use super::raw::RawObjectiveFrontmatter;
use super::types::Objective;

pub(crate) fn parse_objective_file(path: &Path, raw: &str) -> Result<Objective, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let raw_frontmatter: RawObjectiveFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;

    let id = super::identifier::ObjectiveId::new(raw_frontmatter.id)?;
    let concept_id = ConceptId::new(raw_frontmatter.concept_id)?;

    let description = body.trim().to_owned();
    if description.is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: path.to_owned(),
            field: "description",
        });
    }

    let provenance_refs = convert_provenance_refs(id.as_str(), raw_frontmatter.provenance_refs)?;

    Ok(Objective {
        id,
        concept_id,
        description: body,
        provenance_refs,
    })
}
```

- [ ] **Step 8: Register the modules**

In `src-tauri/src/knowledge/mod.rs`, add `mod concept;`, `mod objective;`, `mod provenance;` after `mod frontmatter;`. No `pub use` for any of the three.

- [ ] **Step 9: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge:: && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/knowledge/concept.rs src-tauri/src/knowledge/objective.rs src-tauri/src/knowledge/provenance.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): parse Concept and Objective entity files"
```

---

### Task 7: `Example` body grammar parser

**Files:**
- Create: `src-tauri/src/knowledge/example_body.rs`
- Modify: `src-tauri/src/knowledge/error.rs` (add `MissingExampleSection`, `DuplicateExampleSection`, `OutOfOrderExampleSection`, `UnknownExampleSection`, `ContentBeforeProblem`, `InvalidHintLine`, `EmptyHintsSection`)
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/example_body.rs`

**Interfaces:**
- Produces: `pub(crate) struct ParsedExampleBody { pub problem: String, pub solution: String, pub hints: Vec<String> }`; `pub(crate) fn parse_example_body(entity_id: &str, body: &str) -> Result<ParsedExampleBody, KnowledgeError>`. Consumed by Task 8.

This is the most custom parsing logic in the module (spec §9's closed `## Problem`/`## Solution`/`## Hints` grammar) and gets the deepest test coverage of any single task, per the spec's own instruction that it "deserves its own focused tests."

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/example_body.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const VALID_BODY: &str = "## Problem\n\nFind the volume.\n\n## Solution\n\nV = 8pi/3.\n\n## Hints\n\n- Identify the radius first.\n- Then the height.\n";

    #[test]
    fn valid_body_parses_all_three_sections() {
        let parsed = parse_example_body("shell.example_basic", VALID_BODY).unwrap();
        assert_eq!(parsed.problem, "Find the volume.");
        assert_eq!(parsed.solution, "V = 8pi/3.");
        assert_eq!(parsed.hints, vec!["Identify the radius first.", "Then the height."]);
    }

    #[test]
    fn hints_is_optional() {
        let body = "## Problem\n\nFind the volume.\n\n## Solution\n\nV = 8pi/3.\n";
        let parsed = parse_example_body("shell.example_basic", body).unwrap();
        assert!(parsed.hints.is_empty());
    }

    #[test]
    fn hint_order_is_preserved() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n- first\n- second\n- third\n";
        let parsed = parse_example_body("shell.example_basic", body).unwrap();
        assert_eq!(parsed.hints, vec!["first", "second", "third"]);
    }

    #[test]
    fn missing_problem_is_rejected() {
        let body = "## Solution\n\nV = 8pi/3.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::MissingExampleSection { section: "Problem", .. })
        ));
    }

    #[test]
    fn missing_solution_is_rejected() {
        let body = "## Problem\n\nFind the volume.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::MissingExampleSection { section: "Solution", .. })
        ));
    }

    #[test]
    fn empty_problem_is_rejected() {
        let body = "## Problem\n\n## Solution\n\nV = 8pi/3.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::MissingExampleSection { section: "Problem", .. })
        ));
    }

    #[test]
    fn duplicate_heading_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS1.\n\n## Solution\n\nS2.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::DuplicateExampleSection { section: "Solution", .. })
        ));
    }

    #[test]
    fn out_of_order_heading_is_rejected() {
        let body = "## Solution\n\nS.\n\n## Problem\n\nP.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::OutOfOrderExampleSection { section: "Problem", .. })
        ));
    }

    #[test]
    fn unknown_heading_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Notes\n\nExtra.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::UnknownExampleSection { .. })
        ));
    }

    #[test]
    fn content_before_problem_is_rejected() {
        let body = "Stray intro text.\n\n## Problem\n\nP.\n\n## Solution\n\nS.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::ContentBeforeProblem { .. })
        ));
    }

    #[test]
    fn whitespace_before_problem_is_accepted() {
        let body = "\n\n## Problem\n\nP.\n\n## Solution\n\nS.\n";
        assert!(parse_example_body("shell.example_basic", body).is_ok());
    }

    #[test]
    fn non_list_hints_content_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\nJust a paragraph, not a list.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::InvalidHintLine { .. })
        ));
    }

    #[test]
    fn multiline_hint_continuation_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n- first line\n  continuation line\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::InvalidHintLine { .. })
        ));
    }

    #[test]
    fn empty_hints_section_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::EmptyHintsSection { .. })
        ));
    }

    #[test]
    fn opaque_content_below_recognized_sections_is_preserved_verbatim() {
        let body = "## Problem\n\nP with a **bold** word and a\n### sub-heading inside it.\n\n## Solution\n\nS.\n";
        let parsed = parse_example_body("shell.example_basic", body).unwrap();
        assert!(parsed.problem.contains("### sub-heading inside it."));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::example_body`
Expected: FAIL to compile — `parse_example_body`, `ParsedExampleBody` not defined yet.

- [ ] **Step 3: Extend `KnowledgeError`, then write the implementation**

Add seven variants to `KnowledgeError` in `src-tauri/src/knowledge/error.rs`:

```rust
    MissingExampleSection {
        entity_id: String,
        section: &'static str,
    },
    DuplicateExampleSection {
        entity_id: String,
        section: &'static str,
    },
    OutOfOrderExampleSection {
        entity_id: String,
        section: &'static str,
    },
    UnknownExampleSection {
        entity_id: String,
        heading: String,
    },
    ContentBeforeProblem {
        entity_id: String,
    },
    InvalidHintLine {
        entity_id: String,
        line: String,
    },
    EmptyHintsSection {
        entity_id: String,
    },
```

```rust
            Self::MissingExampleSection { entity_id, section } => write!(f, "example {entity_id} is missing required section ## {section}"),
            Self::DuplicateExampleSection { entity_id, section } => write!(f, "example {entity_id} declares ## {section} more than once"),
            Self::OutOfOrderExampleSection { entity_id, section } => write!(f, "example {entity_id}: ## {section} appears out of the required Problem/Solution/Hints order"),
            Self::UnknownExampleSection { entity_id, heading } => write!(f, "example {entity_id} contains unrecognized heading: {heading}"),
            Self::ContentBeforeProblem { entity_id } => write!(f, "example {entity_id} has non-whitespace content before ## Problem"),
            Self::InvalidHintLine { entity_id, line } => write!(f, "example {entity_id}: invalid line under ## Hints (expected \"- <hint>\"): {line}"),
            Self::EmptyHintsSection { entity_id } => write!(f, "example {entity_id} declares ## Hints with no hint items"),
```

Then prepend to `src-tauri/src/knowledge/example_body.rs`:

```rust
use super::error::KnowledgeError;

const PROBLEM: &str = "## Problem";
const SOLUTION: &str = "## Solution";
const HINTS: &str = "## Hints";
const RECOGNIZED_HEADINGS: [&str; 3] = [PROBLEM, SOLUTION, HINTS];
const EXPECTED_ORDER: [&str; 3] = [PROBLEM, SOLUTION, HINTS];

pub(crate) struct ParsedExampleBody {
    pub problem: String,
    pub solution: String,
    pub hints: Vec<String>,
}

pub(crate) fn parse_example_body(
    entity_id: &str,
    body: &str,
) -> Result<ParsedExampleBody, KnowledgeError> {
    let mut sections: Vec<(&str, Vec<&str>)> = Vec::new();
    let mut current: Option<(&str, Vec<&str>)> = None;
    let mut preamble: Vec<&str> = Vec::new();

    for line in body.lines() {
        let trimmed_end = line.trim_end_matches('\r');
        if let Some(heading) = RECOGNIZED_HEADINGS
            .iter()
            .find(|candidate| trimmed_end == **candidate)
        {
            if let Some(finished) = current.take() {
                sections.push(finished);
            }
            current = Some((*heading, Vec::new()));
        } else if trimmed_end.starts_with("## ") {
            return Err(KnowledgeError::UnknownExampleSection {
                entity_id: entity_id.to_owned(),
                heading: trimmed_end.trim().to_owned(),
            });
        } else if let Some((_, content)) = current.as_mut() {
            content.push(line);
        } else {
            preamble.push(line);
        }
    }
    if let Some(finished) = current.take() {
        sections.push(finished);
    }

    if preamble.iter().any(|line| !line.trim().is_empty()) {
        return Err(KnowledgeError::ContentBeforeProblem {
            entity_id: entity_id.to_owned(),
        });
    }

    let mut seen: Vec<&str> = Vec::new();
    for (heading, _) in &sections {
        if seen.contains(heading) {
            return Err(KnowledgeError::DuplicateExampleSection {
                entity_id: entity_id.to_owned(),
                section: section_name(heading),
            });
        }
        seen.push(heading);
    }

    let mut highest_seen = None;
    for (heading, _) in &sections {
        let position = EXPECTED_ORDER
            .iter()
            .position(|candidate| candidate == heading)
            .expect("heading was already validated as recognized");
        if let Some(highest) = highest_seen {
            if position < highest {
                return Err(KnowledgeError::OutOfOrderExampleSection {
                    entity_id: entity_id.to_owned(),
                    section: section_name(heading),
                });
            }
        }
        highest_seen = Some(position);
    }

    let problem = section_text(&sections, PROBLEM, entity_id, "Problem")?;
    let solution = section_text(&sections, SOLUTION, entity_id, "Solution")?;
    let hints = match sections.iter().find(|(heading, _)| *heading == HINTS) {
        None => Vec::new(),
        Some((_, content)) => parse_hints(entity_id, content)?,
    };

    Ok(ParsedExampleBody {
        problem,
        solution,
        hints,
    })
}

fn section_name(heading: &str) -> &'static str {
    match heading {
        PROBLEM => "Problem",
        SOLUTION => "Solution",
        HINTS => "Hints",
        _ => unreachable!("heading was already validated as recognized"),
    }
}

fn section_text(
    sections: &[(&str, Vec<&str>)],
    heading: &str,
    entity_id: &str,
    section: &'static str,
) -> Result<String, KnowledgeError> {
    let text = sections
        .iter()
        .find(|(candidate, _)| *candidate == heading)
        .map(|(_, content)| content.join("\n").trim().to_owned())
        .ok_or_else(|| KnowledgeError::MissingExampleSection {
            entity_id: entity_id.to_owned(),
            section,
        })?;
    if text.is_empty() {
        return Err(KnowledgeError::MissingExampleSection {
            entity_id: entity_id.to_owned(),
            section,
        });
    }
    Ok(text)
}

fn parse_hints(entity_id: &str, lines: &[&str]) -> Result<Vec<String>, KnowledgeError> {
    let mut hints = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        match line.strip_prefix("- ") {
            Some(text) => hints.push(text.trim_end_matches('\r').trim().to_owned()),
            None => {
                return Err(KnowledgeError::InvalidHintLine {
                    entity_id: entity_id.to_owned(),
                    line: (*line).to_owned(),
                })
            }
        }
    }
    if hints.is_empty() {
        return Err(KnowledgeError::EmptyHintsSection {
            entity_id: entity_id.to_owned(),
        });
    }
    Ok(hints)
}
```

Only exact `## Problem`/`## Solution`/`## Hints` lines are structurally recognized. Any other Markdown — deeper headings, bold/italic, code spans, nested lists — is opaque content copied verbatim into whichever section's text it falls under; the last test in Step 1 (`opaque_content_below_recognized_sections_is_preserved_verbatim`) is what proves this. Per spec §13, which specific `KnowledgeError` variant a given malformed body triggers is not part of the interoperability contract — only the final accept/reject outcome is — so it is fine, and expected, that some malformed inputs are caught by whichever check happens to run first (e.g. an entirely absent `## Problem` heading is caught by `MissingExampleSection`, not `OutOfOrderExampleSection`, even in bodies where ordering is also wrong).

- [ ] **Step 4: Register the module**

In `src-tauri/src/knowledge/mod.rs`, add `mod example_body;` after `mod objective;`. No `pub use`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::example_body && cargo clippy --all-targets --locked -- -D warnings`
Expected: all tests PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/example_body.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): add Example body grammar parser"
```

---

### Task 8: `Example` entity parser

**Files:**
- Create: `src-tauri/src/knowledge/example.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/example.rs`

**Interfaces:**
- Consumes: `split_frontmatter` (Task 5); `RawExampleFrontmatter` (Task 3); `parse_example_body`, `ParsedExampleBody` (Task 7); `convert_provenance_refs` (Task 6); `Example` (Task 2); `ConceptId`, `ObjectiveId` (Task 1).
- Produces: `pub(crate) fn parse_example_file(path: &Path, raw: &str) -> Result<Example, KnowledgeError>`. Used by Task 9. Does not yet validate the cross-concept objective constraint (needs every `Objective` in the package to check against — that's Task 10) or that `concept_id`/`objective_ids`/`provenance_refs[].source_id` resolve.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/example.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const VALID_EXAMPLE: &str = r#"+++
id = "shell.example_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.13"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 4x - x^2` and below by the x-axis over `[0, 3]` around the *y*-axis.

## Solution

V = 63*pi/2.

## Hints

- Identify the shell radius and height as functions of `x` before integrating.
"#;

    #[test]
    fn valid_example_parses() {
        let example = parse_example_file(Path::new("examples/shell.example_y_poly.md"), VALID_EXAMPLE).unwrap();
        assert_eq!(example.id.as_str(), "shell.example_y_poly");
        assert_eq!(example.concept_id.as_str(), "shell.method_vertical_axis");
        assert_eq!(example.objective_ids.len(), 1);
        assert!(example.problem.starts_with("Find the volume"));
        assert_eq!(example.solution, "V = 63*pi/2.");
        assert_eq!(example.hints.len(), 1);
        assert_eq!(example.provenance_refs.len(), 1);
    }

    #[test]
    fn objective_ids_may_be_empty() {
        let raw = VALID_EXAMPLE.replace(r#"objective_ids = ["shell.setup_radius_height"]"#, "");
        let example = parse_example_file(Path::new("examples/shell.example_y_poly.md"), &raw).unwrap();
        assert!(example.objective_ids.is_empty());
    }

    #[test]
    fn missing_problem_section_is_rejected() {
        let raw = VALID_EXAMPLE.replace(
            "## Problem\n\nFind the volume of the solid formed by revolving the region bounded above by\n`f(x) = 4x - x^2` and below by the x-axis over `[0, 3]` around the *y*-axis.\n\n",
            "",
        );
        assert!(matches!(
            parse_example_file(Path::new("examples/shell.example_y_poly.md"), &raw),
            Err(KnowledgeError::MissingExampleSection { section: "Problem", .. })
        ));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::example`
Expected: FAIL to compile — `parse_example_file` not defined yet.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/knowledge/example.rs`:

```rust
use std::path::Path;

use super::error::KnowledgeError;
use super::example_body::parse_example_body;
use super::frontmatter::split_frontmatter;
use super::identifier::{ConceptId, ObjectiveId};
use super::provenance::convert_provenance_refs;
use super::raw::RawExampleFrontmatter;
use super::types::Example;

pub(crate) fn parse_example_file(path: &Path, raw: &str) -> Result<Example, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let raw_frontmatter: RawExampleFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;

    let id = super::identifier::ExampleId::new(raw_frontmatter.id)?;
    let concept_id = ConceptId::new(raw_frontmatter.concept_id)?;
    let objective_ids = raw_frontmatter
        .objective_ids
        .into_iter()
        .map(ObjectiveId::new)
        .collect::<Result<Vec<_>, _>>()?;

    let parsed_body = parse_example_body(id.as_str(), &body)?;
    let provenance_refs = convert_provenance_refs(id.as_str(), raw_frontmatter.provenance_refs)?;

    Ok(Example {
        id,
        concept_id,
        objective_ids,
        problem: parsed_body.problem,
        solution: parsed_body.solution,
        hints: parsed_body.hints,
        provenance_refs,
    })
}
```

- [ ] **Step 4: Register the module**

In `src-tauri/src/knowledge/mod.rs`, add `mod example;` after `mod example_body;`. No `pub use`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::example && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/example.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): parse Example entity files"
```

---

### Task 9: Package discovery

**Files:**
- Create: `src-tauri/src/knowledge/discover.rs`
- Modify: `src-tauri/src/knowledge/error.rs` (add `Io`, `MissingPackageToml`, `MissingSourcesToml`, `UnexpectedEntityFile`, `NestedEntityDirectory`, `FilenameIdMismatch`)
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/discover.rs`

**Interfaces:**
- Consumes: `parse_concept_file`, `parse_objective_file`, `parse_example_file` (Tasks 6, 8); `Concept`, `Objective`, `Example` (Task 2).
- Produces: `pub(crate) struct DiscoveredEntities { pub concepts: Vec<Concept>, pub objectives: Vec<Objective>, pub examples: Vec<Example> }`; `pub(crate) fn read_package_toml(root: &Path) -> Result<String, KnowledgeError>`; `pub(crate) fn read_sources_toml(root: &Path) -> Result<String, KnowledgeError>`; `pub(crate) fn discover_entities(root: &Path) -> Result<DiscoveredEntities, KnowledgeError>`. Used by Task 12 (loader).

Enforces spec §3: sorted deterministic discovery, filename/ID agreement, rejection of non-`.md` files and nested directories inside `concepts/`/`objectives`/`examples/`, and `concepts/`/`objectives/`/`examples/` each being optional when empty. Files at the package root other than `package.toml`/`sources.toml`/the three entity directories are never touched by this code at all, which is what satisfies spec §15's "ignored, not rejected" rule for root documentation files — there's no code path that would reject them because nothing here looks at them.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/discover.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_package_dir(test_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("axiom_knowledge_discover_test_{test_name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    const VALID_CONCEPT: &str = "+++\nid = \"shell.a\"\nname = \"A\"\n\n[[provenance_refs]]\nsource_id = \"src.x\"\nkind = \"direct\"\n+++\n\nBody.\n";

    #[test]
    fn missing_concepts_directory_yields_zero_concepts() {
        let root = temp_package_dir("missing_concepts_directory_yields_zero_concepts");
        let discovered = discover_entities(&root).unwrap();
        assert!(discovered.concepts.is_empty());
    }

    #[test]
    fn discovers_and_sorts_concept_files() {
        let root = temp_package_dir("discovers_and_sorts_concept_files");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("shell.a.md"), VALID_CONCEPT).unwrap();
        let discovered = discover_entities(&root).unwrap();
        assert_eq!(discovered.concepts.len(), 1);
        assert_eq!(discovered.concepts[0].id.as_str(), "shell.a");
    }

    #[test]
    fn filename_id_mismatch_is_rejected() {
        let root = temp_package_dir("filename_id_mismatch_is_rejected");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("shell.wrong_name.md"), VALID_CONCEPT).unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::FilenameIdMismatch { .. })
        ));
    }

    #[test]
    fn two_files_declaring_the_same_id_are_rejected_via_filename_mismatch() {
        // Per-kind ID uniqueness for file-per-entity directories doesn't need a
        // separate duplicate-tracking pass: filenames are unique within a directory
        // by filesystem construction, and every file's own id MUST equal its own
        // filename (§3). So two different files can never both validly declare the
        // same id — whichever one doesn't match its own filename is rejected first.
        // This is different from sources.toml (Task 4), where multiple [[sources]]
        // blocks share one file and duplicate SourceIds are a real, separately
        // checked case.
        let root = temp_package_dir("two_files_declaring_the_same_id_are_rejected_via_filename_mismatch");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("shell.a.md"), VALID_CONCEPT).unwrap();
        // Deliberately the same content, same declared id — the point of this test is
        // that shell.b.md's filename never matches "shell.a" regardless of what id it
        // declares, so this doesn't need a distinct id to make the case.
        fs::write(concepts_dir.join("shell.b.md"), VALID_CONCEPT).unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::FilenameIdMismatch { .. })
        ));
    }

    #[test]
    fn non_md_file_in_entity_directory_is_rejected() {
        let root = temp_package_dir("non_md_file_in_entity_directory_is_rejected");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("notes.txt"), "stray file").unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::UnexpectedEntityFile { .. })
        ));
    }

    #[test]
    fn nested_directory_in_entity_directory_is_rejected() {
        let root = temp_package_dir("nested_directory_in_entity_directory_is_rejected");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(concepts_dir.join("nested")).unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::NestedEntityDirectory { .. })
        ));
    }

    #[test]
    fn missing_package_toml_is_reported() {
        let root = temp_package_dir("missing_package_toml_is_reported");
        assert!(matches!(
            read_package_toml(&root),
            Err(KnowledgeError::MissingPackageToml { .. })
        ));
    }

    #[test]
    fn missing_sources_toml_is_reported() {
        let root = temp_package_dir("missing_sources_toml_is_reported");
        assert!(matches!(
            read_sources_toml(&root),
            Err(KnowledgeError::MissingSourcesToml { .. })
        ));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::discover`
Expected: FAIL to compile — none of `discover_entities`/`read_package_toml`/`read_sources_toml`/`DiscoveredEntities` exist yet.

- [ ] **Step 3: Extend `KnowledgeError`, then write the implementation**

Add six variants to `KnowledgeError` in `src-tauri/src/knowledge/error.rs`:

```rust
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    MissingPackageToml {
        path: PathBuf,
    },
    MissingSourcesToml {
        path: PathBuf,
    },
    UnexpectedEntityFile {
        path: PathBuf,
    },
    NestedEntityDirectory {
        path: PathBuf,
    },
    FilenameIdMismatch {
        path: PathBuf,
        filename_id: String,
        frontmatter_id: String,
    },
```

```rust
            Self::Io { path, source } => write!(f, "I/O error reading {}: {source}", path.display()),
            Self::MissingPackageToml { path } => write!(f, "{} is missing package.toml", path.display()),
            Self::MissingSourcesToml { path } => write!(f, "{} is missing sources.toml", path.display()),
            Self::UnexpectedEntityFile { path } => write!(f, "{} is not a recognized entity file (expected <id>.md)", path.display()),
            Self::NestedEntityDirectory { path } => write!(f, "{} is a nested directory, not permitted inside an entity directory", path.display()),
            Self::FilenameIdMismatch { path, filename_id, frontmatter_id } => write!(
                f,
                "{}: filename implies id \"{filename_id}\" but frontmatter declares \"{frontmatter_id}\"",
                path.display()
            ),
```

This task's `Io` variant is the second one carrying its own `source` (after Task 4's `TomlSyntax`), so `impl Error for KnowledgeError`'s `fn source()` needs a second match arm — replace that whole `impl Error for KnowledgeError` block with:

```rust
impl Error for KnowledgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TomlSyntax { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
```

Then prepend to `src-tauri/src/knowledge/discover.rs`:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use super::concept::parse_concept_file;
use super::error::KnowledgeError;
use super::example::parse_example_file;
use super::objective::parse_objective_file;
use super::types::{Concept, Example, Objective};

pub(crate) struct DiscoveredEntities {
    pub concepts: Vec<Concept>,
    pub objectives: Vec<Objective>,
    pub examples: Vec<Example>,
}

pub(crate) fn read_package_toml(root: &Path) -> Result<String, KnowledgeError> {
    fs::read_to_string(root.join("package.toml"))
        .map_err(|_| KnowledgeError::MissingPackageToml { path: root.to_owned() })
}

pub(crate) fn read_sources_toml(root: &Path) -> Result<String, KnowledgeError> {
    fs::read_to_string(root.join("sources.toml"))
        .map_err(|_| KnowledgeError::MissingSourcesToml { path: root.to_owned() })
}

pub(crate) fn discover_entities(root: &Path) -> Result<DiscoveredEntities, KnowledgeError> {
    let concepts = load_entities(root, "concepts", parse_concept_file, |c: &Concept| {
        c.id.as_str().to_owned()
    })?;
    let objectives = load_entities(root, "objectives", parse_objective_file, |o: &Objective| {
        o.id.as_str().to_owned()
    })?;
    let examples = load_entities(root, "examples", parse_example_file, |e: &Example| {
        e.id.as_str().to_owned()
    })?;

    Ok(DiscoveredEntities {
        concepts,
        objectives,
        examples,
    })
}

fn list_entity_files(root: &Path, subdir: &str) -> Result<Vec<PathBuf>, KnowledgeError> {
    let dir = root.join(subdir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|source| KnowledgeError::Io {
        path: dir.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| KnowledgeError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            return Err(KnowledgeError::NestedEntityDirectory { path });
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            return Err(KnowledgeError::UnexpectedEntityFile { path });
        }
        entries.push(path);
    }
    entries.sort();
    Ok(entries)
}

// Per-kind ID uniqueness (spec §2) needs no separate duplicate-tracking pass here:
// a directory's filenames are unique by filesystem construction, and the
// filename/id check below already forces every file's own id to equal its own
// filename. Two different files can therefore never both validly declare the same
// id — whichever one doesn't match its own filename is rejected first. Contrast
// `sources.toml` (Task 4), where multiple `[[sources]]` blocks share one file and
// duplicate `SourceId`s are a real, independently-necessary check.
fn load_entities<T>(
    root: &Path,
    subdir: &str,
    parse: impl Fn(&Path, &str) -> Result<T, KnowledgeError>,
    id_of: impl Fn(&T) -> String,
) -> Result<Vec<T>, KnowledgeError> {
    let paths = list_entity_files(root, subdir)?;
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        let raw = fs::read_to_string(&path).map_err(|source| KnowledgeError::Io {
            path: path.clone(),
            source,
        })?;
        let entity = parse(&path, &raw)?;
        let id = id_of(&entity);
        let filename_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_owned();
        if filename_id != id {
            return Err(KnowledgeError::FilenameIdMismatch {
                path,
                filename_id,
                frontmatter_id: id,
            });
        }
        result.push(entity);
    }
    Ok(result)
}
```

`entries.sort()` on the `Vec<PathBuf>` before parsing is what makes discovery order deterministic regardless of raw `read_dir` iteration order (spec §3, §16) — parsing itself happens in that sorted order, so the resulting `Vec<Concept>`/`Vec<Objective>`/`Vec<Example>` are always in filename order for any two runs against the same directory contents, on any OS.

- [ ] **Step 4: Register the module**

In `src-tauri/src/knowledge/mod.rs`, add `mod discover;` after `mod example;`. No `pub use`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::discover && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/discover.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): add deterministic package discovery"
```

---

### Task 10: Cross-entity reference validation

**Files:**
- Create: `src-tauri/src/knowledge/validate.rs`
- Modify: `src-tauri/src/knowledge/error.rs` (add `UnresolvedConcept`, `UnresolvedObjective`, `UnresolvedSource`, `CrossConceptObjective`)
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/validate.rs`

**Interfaces:**
- Consumes: `DiscoveredEntities` (Task 9); `Concept`, `Objective`, `Example`, `Source`, `ProvenanceRef` (Task 2).
- Produces: `pub(crate) fn validate_references(entities: &DiscoveredEntities, sources: &[Source]) -> Result<(), KnowledgeError>`. Used by Task 12 (loader).

Implements spec §12's reference table: `Objective.concept_id`, `Example.concept_id`, `Example.objective_ids` (including the cross-concept constraint from spec §9), and every entity's `provenance_refs[].source_id`. Does not touch `prerequisite_ids`/`related_ids` — those are Task 11.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/validate.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::identifier::{ConceptId, ExampleId, ObjectiveId, SourceId};
    // Concept, Example, Objective aren't imported anywhere validate.rs's own production
    // code reaches — that code only ever touches them through DiscoveredEntities field
    // access and inference, never naming the types directly — so `use super::*` doesn't
    // bring them into this test module either. The helpers below construct them by name
    // directly, so they need their own explicit import here.
    use crate::knowledge::types::{Concept, Example, Objective, ProvenanceKind, ProvenanceRef};

    fn source(id: &str) -> Source {
        Source {
            id: SourceId::new(id).unwrap(),
            title: "Source".to_owned(),
            authors: vec![],
            edition: None,
            license: None,
        }
    }

    fn provenance(source_id: &str) -> Vec<ProvenanceRef> {
        vec![ProvenanceRef {
            source_id: SourceId::new(source_id).unwrap(),
            locator: None,
            kind: ProvenanceKind::Direct,
        }]
    }

    fn concept(id: &str) -> Concept {
        Concept {
            id: ConceptId::new(id).unwrap(),
            name: id.to_owned(),
            topic: None,
            description: "d".to_owned(),
            prerequisite_ids: vec![],
            related_ids: vec![],
            provenance_refs: provenance("src.a"),
        }
    }

    fn objective(id: &str, concept_id: &str) -> Objective {
        Objective {
            id: ObjectiveId::new(id).unwrap(),
            concept_id: ConceptId::new(concept_id).unwrap(),
            description: "d".to_owned(),
            provenance_refs: provenance("src.a"),
        }
    }

    fn example(id: &str, concept_id: &str, objective_ids: Vec<&str>) -> Example {
        Example {
            id: ExampleId::new(id).unwrap(),
            concept_id: ConceptId::new(concept_id).unwrap(),
            objective_ids: objective_ids
                .into_iter()
                .map(|o| ObjectiveId::new(o).unwrap())
                .collect(),
            problem: "p".to_owned(),
            solution: "s".to_owned(),
            hints: vec![],
            provenance_refs: provenance("src.a"),
        }
    }

    #[test]
    fn valid_references_resolve() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![objective("shell.obj", "shell.a")],
            examples: vec![example("shell.ex", "shell.a", vec!["shell.obj"])],
        };
        assert!(validate_references(&entities, &[source("src.a")]).is_ok());
    }

    #[test]
    fn unresolved_objective_concept_id_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![objective("shell.obj", "shell.missing")],
            examples: vec![],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::UnresolvedConcept { .. })
        ));
    }

    #[test]
    fn unresolved_example_objective_id_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![],
            examples: vec![example("shell.ex", "shell.a", vec!["shell.missing"])],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::UnresolvedObjective { .. })
        ));
    }

    #[test]
    fn cross_concept_objective_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a"), concept("shell.b")],
            objectives: vec![objective("shell.obj", "shell.b")],
            examples: vec![example("shell.ex", "shell.a", vec!["shell.obj"])],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::CrossConceptObjective { .. })
        ));
    }

    #[test]
    fn unresolved_provenance_source_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![],
            examples: vec![],
        };
        assert!(matches!(
            validate_references(&entities, &[]),
            Err(KnowledgeError::UnresolvedSource { .. })
        ));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::validate`
Expected: FAIL to compile — `validate_references` not defined yet.

- [ ] **Step 3: Extend `KnowledgeError`, then write the implementation**

Add four variants to `KnowledgeError` in `src-tauri/src/knowledge/error.rs`:

```rust
    UnresolvedConcept {
        entity_id: String,
        field: &'static str,
        target: String,
    },
    UnresolvedObjective {
        entity_id: String,
        field: &'static str,
        target: String,
    },
    UnresolvedSource {
        entity_id: String,
        target: String,
    },
    CrossConceptObjective {
        example_id: String,
        objective_id: String,
    },
```

```rust
            Self::UnresolvedConcept { entity_id, field, target } => write!(f, "{entity_id}.{field} references unknown Concept: {target}"),
            Self::UnresolvedObjective { entity_id, field, target } => write!(f, "{entity_id}.{field} references unknown Objective: {target}"),
            Self::UnresolvedSource { entity_id, target } => write!(f, "{entity_id} references unknown Source: {target}"),
            Self::CrossConceptObjective { example_id, objective_id } => write!(
                f,
                "example {example_id} references objective {objective_id} belonging to a different concept"
            ),
```

Then prepend to `src-tauri/src/knowledge/validate.rs`:

```rust
use std::collections::{HashMap, HashSet};

use super::discover::DiscoveredEntities;
use super::error::KnowledgeError;
use super::identifier::SourceId;
use super::types::{ProvenanceRef, Source};

pub(crate) fn validate_references(
    entities: &DiscoveredEntities,
    sources: &[Source],
) -> Result<(), KnowledgeError> {
    let concept_ids: HashSet<_> = entities.concepts.iter().map(|c| &c.id).collect();
    let objectives_by_id: HashMap<_, _> = entities.objectives.iter().map(|o| (&o.id, o)).collect();
    let source_ids: HashSet<_> = sources.iter().map(|s| &s.id).collect();

    for objective in &entities.objectives {
        if !concept_ids.contains(&objective.concept_id) {
            return Err(KnowledgeError::UnresolvedConcept {
                entity_id: objective.id.as_str().to_owned(),
                field: "concept_id",
                target: objective.concept_id.as_str().to_owned(),
            });
        }
    }

    for example in &entities.examples {
        if !concept_ids.contains(&example.concept_id) {
            return Err(KnowledgeError::UnresolvedConcept {
                entity_id: example.id.as_str().to_owned(),
                field: "concept_id",
                target: example.concept_id.as_str().to_owned(),
            });
        }
        for objective_id in &example.objective_ids {
            let Some(objective) = objectives_by_id.get(objective_id) else {
                return Err(KnowledgeError::UnresolvedObjective {
                    entity_id: example.id.as_str().to_owned(),
                    field: "objective_ids",
                    target: objective_id.as_str().to_owned(),
                });
            };
            if objective.concept_id != example.concept_id {
                return Err(KnowledgeError::CrossConceptObjective {
                    example_id: example.id.as_str().to_owned(),
                    objective_id: objective_id.as_str().to_owned(),
                });
            }
        }
    }

    validate_provenance_sources(
        entities.concepts.iter().map(|c| (c.id.as_str(), &c.provenance_refs)),
        &source_ids,
    )?;
    validate_provenance_sources(
        entities.objectives.iter().map(|o| (o.id.as_str(), &o.provenance_refs)),
        &source_ids,
    )?;
    validate_provenance_sources(
        entities.examples.iter().map(|e| (e.id.as_str(), &e.provenance_refs)),
        &source_ids,
    )?;

    Ok(())
}

fn validate_provenance_sources<'a>(
    entities: impl Iterator<Item = (&'a str, &'a Vec<ProvenanceRef>)>,
    source_ids: &HashSet<&SourceId>,
) -> Result<(), KnowledgeError> {
    for (entity_id, refs) in entities {
        for provenance_ref in refs {
            if !source_ids.contains(&provenance_ref.source_id) {
                return Err(KnowledgeError::UnresolvedSource {
                    entity_id: entity_id.to_owned(),
                    target: provenance_ref.source_id.as_str().to_owned(),
                });
            }
        }
    }
    Ok(())
}
```

Checks run in a fixed order (concept/objective references before provenance references) purely for readability — per spec §13, exact error ordering across independent invariants is not part of the interoperability contract.

- [ ] **Step 4: Register the module**

In `src-tauri/src/knowledge/mod.rs`, add `mod validate;` after `mod discover;`. No `pub use`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::validate && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/validate.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): validate cross-entity references"
```

---

### Task 11: Relationship validation (prerequisite DAG, related symmetry)

**Files:**
- Create: `src-tauri/src/knowledge/relationships.rs`
- Modify: `src-tauri/src/knowledge/error.rs` (add `SelfReference`, `DuplicateReferenceInList`, `PrerequisiteCycle`, `ReverseDuplicateRelated`)
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/relationships.rs`

**Interfaces:**
- Consumes: `Concept` (Task 2).
- Produces: `pub(crate) fn validate_relationships(concepts: &[Concept]) -> Result<(), KnowledgeError>`; `pub fn related_concepts<'a>(concepts: &'a [Concept], id: &ConceptId) -> Vec<&'a ConceptId>` (the normalized symmetric query view spec §10 requires — this is the one function in this task that *is* part of the public API, since it's how a consumer queries "related to X" without needing to know which side authored the edge). Used by Task 12 (loader calls `validate_relationships`) and re-exported from `mod.rs` for consumers.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/relationships.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::identifier::ConceptId;

    fn concept(id: &str, prerequisites: Vec<&str>, related: Vec<&str>) -> Concept {
        Concept {
            id: ConceptId::new(id).unwrap(),
            name: id.to_owned(),
            topic: None,
            description: "d".to_owned(),
            prerequisite_ids: prerequisites.into_iter().map(|p| ConceptId::new(p).unwrap()).collect(),
            related_ids: related.into_iter().map(|r| ConceptId::new(r).unwrap()).collect(),
            provenance_refs: vec![],
        }
    }

    #[test]
    fn acyclic_prerequisites_are_accepted() {
        let concepts = vec![
            concept("shell.a", vec![], vec![]),
            concept("shell.b", vec!["shell.a"], vec![]),
            concept("shell.c", vec!["shell.a", "shell.b"], vec![]),
        ];
        assert!(validate_relationships(&concepts).is_ok());
    }

    #[test]
    fn self_prerequisite_is_rejected() {
        let concepts = vec![concept("shell.a", vec!["shell.a"], vec![])];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::SelfReference { field: "prerequisite_ids", .. })
        ));
    }

    #[test]
    fn duplicate_prerequisite_id_is_rejected() {
        let concepts = vec![
            concept("shell.a", vec![], vec![]),
            concept("shell.b", vec!["shell.a", "shell.a"], vec![]),
        ];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::DuplicateReferenceInList { field: "prerequisite_ids", .. })
        ));
    }

    #[test]
    fn prerequisite_cycle_is_rejected() {
        let concepts = vec![
            concept("shell.a", vec!["shell.c"], vec![]),
            concept("shell.b", vec!["shell.a"], vec![]),
            concept("shell.c", vec!["shell.b"], vec![]),
        ];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::PrerequisiteCycle { .. })
        ));
    }

    #[test]
    fn self_related_is_rejected() {
        let concepts = vec![concept("shell.a", vec![], vec!["shell.a"])];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::SelfReference { field: "related_ids", .. })
        ));
    }

    #[test]
    fn reverse_double_authored_related_is_rejected() {
        let concepts = vec![
            concept("shell.a", vec![], vec!["shell.b"]),
            concept("shell.b", vec![], vec!["shell.a"]),
        ];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::ReverseDuplicateRelated { .. })
        ));
    }

    #[test]
    fn related_concepts_exposes_the_relation_symmetrically() {
        let concepts = vec![
            concept("shell.a", vec![], vec!["shell.b"]),
            concept("shell.b", vec![], vec![]),
        ];
        let from_a = related_concepts(&concepts, &ConceptId::new("shell.a").unwrap());
        let from_b = related_concepts(&concepts, &ConceptId::new("shell.b").unwrap());
        assert_eq!(from_a, vec![&ConceptId::new("shell.b").unwrap()]);
        assert_eq!(from_b, vec![&ConceptId::new("shell.a").unwrap()]);
    }

    #[test]
    fn a_long_related_chain_is_not_treated_as_a_cycle_error() {
        // related has no acyclic constraint (spec §10) — this must not error.
        let concepts = vec![
            concept("shell.a", vec![], vec!["shell.b"]),
            concept("shell.b", vec![], vec!["shell.c"]),
            concept("shell.c", vec![], vec!["shell.a"]),
        ];
        assert!(validate_relationships(&concepts).is_ok());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::relationships`
Expected: FAIL to compile — none of the functions exist yet.

- [ ] **Step 3: Extend `KnowledgeError`, then write the implementation**

Add four variants to `KnowledgeError` in `src-tauri/src/knowledge/error.rs` — this is the last task that extends it; after this step the enum has all 34 variants used across the plan:

```rust
    SelfReference {
        entity_id: String,
        field: &'static str,
    },
    DuplicateReferenceInList {
        entity_id: String,
        field: &'static str,
        target: String,
    },
    PrerequisiteCycle {
        cycle: Vec<String>,
    },
    ReverseDuplicateRelated {
        first: String,
        second: String,
    },
```

```rust
            Self::SelfReference { entity_id, field } => write!(f, "{entity_id}.{field} references itself"),
            Self::DuplicateReferenceInList { entity_id, field, target } => write!(f, "{entity_id}.{field} lists {target} more than once"),
            Self::PrerequisiteCycle { cycle } => write!(f, "prerequisite cycle detected: {}", cycle.join(" -> ")),
            Self::ReverseDuplicateRelated { first, second } => write!(
                f,
                "related_ids declared on both {first} and {second}; author it on exactly one side"
            ),
```

Then prepend to `src-tauri/src/knowledge/relationships.rs`:

```rust
use std::collections::{HashMap, HashSet};

use super::error::KnowledgeError;
use super::identifier::ConceptId;
use super::types::Concept;

#[derive(Clone, Copy, PartialEq)]
enum Color {
    White,
    Gray,
    Black,
}

pub(crate) fn validate_relationships(concepts: &[Concept]) -> Result<(), KnowledgeError> {
    let concept_ids: HashSet<&ConceptId> = concepts.iter().map(|c| &c.id).collect();

    for concept in concepts {
        validate_reference_list(
            &concept.id,
            "prerequisite_ids",
            &concept.prerequisite_ids,
            &concept_ids,
        )?;
        validate_reference_list(&concept.id, "related_ids", &concept.related_ids, &concept_ids)?;
    }

    validate_related_symmetry(concepts)?;
    validate_prerequisite_dag(concepts)?;

    Ok(())
}

/// Spec §10's normalized symmetric query view: returns every concept related to
/// `id`, regardless of which side authored the edge.
pub fn related_concepts<'a>(concepts: &'a [Concept], id: &ConceptId) -> Vec<&'a ConceptId> {
    let mut result = Vec::new();
    for concept in concepts {
        if concept.id == *id {
            result.extend(concept.related_ids.iter());
        } else if concept.related_ids.contains(id) {
            result.push(&concept.id);
        }
    }
    result
}

fn validate_reference_list(
    owner: &ConceptId,
    field: &'static str,
    targets: &[ConceptId],
    concept_ids: &HashSet<&ConceptId>,
) -> Result<(), KnowledgeError> {
    let mut seen = HashSet::new();
    for target in targets {
        if target == owner {
            return Err(KnowledgeError::SelfReference {
                entity_id: owner.as_str().to_owned(),
                field,
            });
        }
        if !seen.insert(target) {
            return Err(KnowledgeError::DuplicateReferenceInList {
                entity_id: owner.as_str().to_owned(),
                field,
                target: target.as_str().to_owned(),
            });
        }
        if !concept_ids.contains(target) {
            return Err(KnowledgeError::UnresolvedConcept {
                entity_id: owner.as_str().to_owned(),
                field,
                target: target.as_str().to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_related_symmetry(concepts: &[Concept]) -> Result<(), KnowledgeError> {
    let by_id: HashMap<&ConceptId, &Concept> = concepts.iter().map(|c| (&c.id, c)).collect();
    for concept in concepts {
        for related in &concept.related_ids {
            if let Some(other) = by_id.get(related) {
                if other.related_ids.contains(&concept.id) {
                    return Err(KnowledgeError::ReverseDuplicateRelated {
                        first: concept.id.as_str().to_owned(),
                        second: related.as_str().to_owned(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_prerequisite_dag(concepts: &[Concept]) -> Result<(), KnowledgeError> {
    let by_id: HashMap<&ConceptId, &Concept> = concepts.iter().map(|c| (&c.id, c)).collect();
    let mut colors: HashMap<&ConceptId, Color> =
        concepts.iter().map(|c| (&c.id, Color::White)).collect();
    let mut path: Vec<ConceptId> = Vec::new();

    for id in by_id.keys() {
        if colors.get(id) == Some(&Color::White) {
            visit(id, &by_id, &mut colors, &mut path)?;
        }
    }
    Ok(())
}

fn visit<'a>(
    id: &'a ConceptId,
    by_id: &HashMap<&'a ConceptId, &'a Concept>,
    colors: &mut HashMap<&'a ConceptId, Color>,
    path: &mut Vec<ConceptId>,
) -> Result<(), KnowledgeError> {
    match colors.get(id) {
        Some(Color::Black) => return Ok(()),
        Some(Color::Gray) => {
            let start = path.iter().position(|node| node == id).unwrap_or(0);
            let mut cycle: Vec<String> = path[start..].iter().map(|c| c.as_str().to_owned()).collect();
            cycle.push(id.as_str().to_owned());
            return Err(KnowledgeError::PrerequisiteCycle { cycle });
        }
        _ => {}
    }
    colors.insert(id, Color::Gray);
    path.push(id.clone());
    if let Some(concept) = by_id.get(id) {
        for prerequisite in &concept.prerequisite_ids {
            visit(prerequisite, by_id, colors, path)?;
        }
    }
    path.pop();
    colors.insert(id, Color::Black);
    Ok(())
}
```

Standard three-color DFS cycle detection: a `Gray` node reached again mid-traversal is a back-edge, reconstructed into the reported cycle from the current DFS `path`. `validate_reference_list` runs self-reference and duplicate checks before existence checks for both `prerequisite_ids` and `related_ids`, so a self-reference is always reported as `SelfReference`, never accidentally as `UnresolvedConcept` (a concept's own id is always "resolvable" against itself, which would otherwise mask the more specific error).

- [ ] **Step 4: Register the module**

In `src-tauri/src/knowledge/mod.rs`, add `mod relationships;` after `mod validate;`, and add `pub use relationships::related_concepts;` to the `pub use` block.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge::relationships && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/relationships.rs src-tauri/src/knowledge/error.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): validate prerequisite DAG and related symmetry"
```

(`error.rs` is included because Step 3 adds the four new `KnowledgeError` variants there;
omitting it would produce a commit that doesn't compile on its own.)

Note on provenance validation (spec §11): there is no separate task for it. `≥1 ProvenanceRef`, `ProvenanceKind` parsing, exact-duplicate-ref rejection, and empty-locator rejection are all per-entity checks already implemented in Task 6's `convert_provenance_refs` (no cross-package data needed); `SourceId` resolution needed the whole package's `Source` list, so it's in Task 10. Nothing from spec §11 is left unimplemented.

---

### Task 12: Top-level loader and public API

**Files:**
- Create: `src-tauri/src/knowledge/loader.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`
- Test: inline `#[cfg(test)]` in `src-tauri/src/knowledge/loader.rs`

**Interfaces:**
- Consumes: `read_package_toml`, `read_sources_toml`, `discover_entities` (Task 9); `parse_package_toml`, `parse_sources_toml` (Task 4); `validate_references` (Task 10); `validate_relationships` (Task 11); `KnowledgePackage` (Task 2).
- Produces: `pub fn load_knowledge_package(root: &Path) -> Result<KnowledgePackage, KnowledgeError>` — the **only** function this crate exposes for loading a package. This is the atomic boundary spec §12 requires: every step uses `?`, so no partial `KnowledgePackage` can ever be constructed — either every step succeeds and a fully valid value is returned, or the first failure propagates and nothing is returned.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/knowledge/loader.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("axiom_knowledge_loader_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_minimal_valid_package(root: &Path) {
        fs::write(
            root.join("package.toml"),
            "id = \"org.axiom.test\"\nschema_version = 1\nversion = \"1.0.0\"\ntitle = \"Test\"\ndescription = \"A test package.\"\n",
        )
        .unwrap();
        fs::write(
            root.join("sources.toml"),
            "[[sources]]\nid = \"src.a\"\ntitle = \"Source A\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("concepts")).unwrap();
        fs::write(
            root.join("concepts/shell.a.md"),
            "+++\nid = \"shell.a\"\nname = \"A\"\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nBody.\n",
        )
        .unwrap();
    }

    #[test]
    fn minimal_valid_package_loads() {
        let root = temp_root("minimal_valid_package_loads");
        write_minimal_valid_package(&root);
        let package = load_knowledge_package(&root).unwrap();
        assert_eq!(package.id.as_str(), "org.axiom.test");
        assert_eq!(package.concepts.len(), 1);
        assert_eq!(package.sources.len(), 1);
    }

    #[test]
    fn one_broken_entity_invalidates_the_whole_package() {
        let root = temp_root("one_broken_entity_invalidates_the_whole_package");
        write_minimal_valid_package(&root);
        // A second concept, structurally fine on its own, but citing a Source that
        // does not exist. This must fail the whole load — not silently drop shell.b
        // and return a package containing only shell.a.
        fs::write(
            root.join("concepts/shell.b.md"),
            "+++\nid = \"shell.b\"\nname = \"B\"\n\n[[provenance_refs]]\nsource_id = \"src.missing\"\nkind = \"direct\"\n+++\n\nBody.\n",
        )
        .unwrap();
        assert!(matches!(
            load_knowledge_package(&root),
            Err(KnowledgeError::UnresolvedSource { .. })
        ));
    }

    #[test]
    fn missing_package_toml_fails_before_touching_entities() {
        let root = temp_root("missing_package_toml_fails_before_touching_entities");
        assert!(matches!(
            load_knowledge_package(&root),
            Err(KnowledgeError::MissingPackageToml { .. })
        ));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::loader`
Expected: FAIL to compile — `load_knowledge_package` not defined yet.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/knowledge/loader.rs`:

```rust
use std::path::Path;

use super::discover::{discover_entities, read_package_toml, read_sources_toml};
use super::error::KnowledgeError;
use super::package::{parse_package_toml, parse_sources_toml};
use super::relationships::validate_relationships;
use super::types::KnowledgePackage;
use super::validate::validate_references;

/// Loads and fully validates a Knowledge Package from `root`. Atomic: either every
/// entity parses and every invariant in the spec holds, and a complete, validated
/// `KnowledgePackage` is returned, or the first failure is returned and nothing is.
/// There is no partially-loaded package.
pub fn load_knowledge_package(root: &Path) -> Result<KnowledgePackage, KnowledgeError> {
    let package_toml = read_package_toml(root)?;
    let identity = parse_package_toml(&package_toml)?;

    let sources_toml = read_sources_toml(root)?;
    let sources = parse_sources_toml(&sources_toml)?;

    let entities = discover_entities(root)?;

    validate_references(&entities, &sources)?;
    validate_relationships(&entities.concepts)?;

    Ok(KnowledgePackage {
        id: identity.id,
        schema_version: identity.schema_version,
        version: identity.version,
        title: identity.title,
        description: identity.description,
        concepts: entities.concepts,
        objectives: entities.objectives,
        examples: entities.examples,
        sources,
    })
}
```

- [ ] **Step 4: Finalize `mod.rs` as the module's public surface**

Replace `src-tauri/src/knowledge/mod.rs` in full:

```rust
mod concept;
mod discover;
mod error;
mod example;
mod example_body;
mod frontmatter;
mod identifier;
mod loader;
mod objective;
mod package;
mod provenance;
mod raw;
mod relationships;
mod types;
mod validate;

pub use error::KnowledgeError;
pub use identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};
pub use loader::load_knowledge_package;
pub use relationships::related_concepts;
pub use types::{
    Concept, Example, KnowledgePackage, Objective, ProvenanceKind, ProvenanceRef, Source,
    SourceLocator,
};
```

This is the module's entire public API: one loading function, `related_concepts` for the one query the domain model can't answer by direct field access (spec §10's symmetric `related` view), the domain types, and the error enum. No raw parser types, no `discover`/`validate`/`package`/`frontmatter`/`provenance`/`relationships`' internal validation functions beyond `related_concepts` are exported — a caller outside this module cannot construct a `KnowledgePackage` any way other than `load_knowledge_package` succeeding.

Note what's *not* in this replacement: the `#![allow(dead_code)]` line Task 3 Step 4 added is gone. That's deliberate, not an oversight — `load_knowledge_package` above is what finally calls every `pub(crate)` function this plan built (`parse_package_toml`, `parse_sources_toml`, `discover_entities`, `read_package_toml`, `read_sources_toml`, `validate_references`, `validate_relationships`), so every one of them now has a real caller and the lint should pass on its own. If `cargo clippy` still reports dead code here, that's a real signal something got wired incorrectly (a function this task's loader was supposed to call but doesn't) — fix the wiring, don't put the allow back.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --locked knowledge:: && cargo clippy --all-targets --locked -- -D warnings && cargo build --locked`
Expected: all `knowledge::` tests PASS (should be roughly 55-60 tests across Tasks 1-12 by this point); clippy clean **without** the `#![allow(dead_code)]` present; crate builds.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/knowledge/loader.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): add top-level loader and finalize public API"
```

---

### Task 13: End-to-end conformance corpus

**Files:**
- Create: `src-tauri/src/knowledge/tests/mod.rs`
- Create: `src-tauri/src/knowledge/tests/conformance.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`

**Interfaces:**
- Consumes: `load_knowledge_package`, `KnowledgeError` (Task 12, public API only — this task deliberately does not reach into `pub(crate)` internals, to prove the public surface alone is enough to observe every documented failure mode).

Tasks 1–12 already unit-test every parsing/validation function directly. This task is deliberately different: it proves each spec §18 case is correctly surfaced through the **whole pipeline**, entered only through `load_knowledge_package(root)`, the same way an external caller would hit it — catching any wiring mistake between a correct internal function and an incorrectly-assembled `loader.rs`.

Module conformance fixtures (`src-tauri/src/modules/tests/fixtures/*.toml`) are single committed files because a `module.toml` fixture is one file. A Knowledge Package fixture is an entire directory (`package.toml` + `sources.toml` + entity files), so this task adapts the same "permanent, case-named, never-deleted regression corpus" convention to Rust source instead of a committed directory tree per case: one shared valid base package, and one named mutation function per case, each producing exactly one broken package and asserting the specific `KnowledgeError` variant it must produce. The mutation functions are the permanent regression artifacts — committed, named by the case, never deleted once a bug they caught is fixed, the same discipline as the `.toml` fixtures, in a form that fits a multi-file fixture.

- [ ] **Step 1: Write the base package builder and the first four conformance cases**

Create `src-tauri/src/knowledge/tests/mod.rs`:

```rust
#[cfg(test)]
mod conformance;

#[cfg(test)]
pub(crate) mod support {
    use std::fs;
    use std::path::{Path, PathBuf};

    pub(crate) fn temp_root(case: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("axiom_knowledge_conformance_{case}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A minimal but complete valid package: 1 Source, 2 Concepts (one prerequisite
    /// of the other), 1 Objective, 1 Example — enough surface to mutate into every
    /// spec §18 case without each case needing its own bespoke shape.
    pub(crate) fn write_base_package(root: &Path) {
        fs::write(
            root.join("package.toml"),
            "id = \"org.axiom.conformance\"\nschema_version = 1\nversion = \"1.0.0\"\ntitle = \"Conformance\"\ndescription = \"Conformance base package.\"\n",
        )
        .unwrap();
        fs::write(
            root.join("sources.toml"),
            "[[sources]]\nid = \"src.a\"\ntitle = \"Source A\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("concepts")).unwrap();
        fs::write(
            root.join("concepts/shell.a.md"),
            "+++\nid = \"shell.a\"\nname = \"A\"\nprerequisite_ids = []\nrelated_ids = []\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nConcept A body.\n",
        )
        .unwrap();
        fs::write(
            root.join("concepts/shell.b.md"),
            "+++\nid = \"shell.b\"\nname = \"B\"\nprerequisite_ids = [\"shell.a\"]\nrelated_ids = []\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nConcept B body.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("objectives")).unwrap();
        fs::write(
            root.join("objectives/shell.obj.md"),
            "+++\nid = \"shell.obj\"\nconcept_id = \"shell.a\"\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nObjective body.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("examples")).unwrap();
        fs::write(
            root.join("examples/shell.ex.md"),
            "+++\nid = \"shell.ex\"\nconcept_id = \"shell.a\"\nobjective_ids = [\"shell.obj\"]\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\n## Problem\n\nP.\n\n## Solution\n\nS.\n",
        )
        .unwrap();
    }
}
```

Create `src-tauri/src/knowledge/tests/conformance.rs`:

```rust
use std::fs;

use crate::knowledge::{load_knowledge_package, KnowledgeError};

use super::support::{temp_root, write_base_package};

#[test]
fn base_package_loads() {
    let root = temp_root("base_package_loads");
    write_base_package(&root);
    assert!(load_knowledge_package(&root).is_ok());
}

#[test]
fn invalid_identifier() {
    let root = temp_root("invalid_identifier");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("id = \"shell.a\"", "id = \"Shell-A\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::InvalidIdentifier { .. })
    ));
}

#[test]
fn filename_id_mismatch() {
    let root = temp_root("filename_id_mismatch");
    write_base_package(&root);
    fs::rename(
        root.join("concepts/shell.a.md"),
        root.join("concepts/shell.renamed.md"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::FilenameIdMismatch { .. })
    ));
}

#[test]
fn unresolved_concept_reference() {
    let root = temp_root("unresolved_concept_reference");
    write_base_package(&root);
    fs::write(
        root.join("objectives/shell.obj.md"),
        fs::read_to_string(root.join("objectives/shell.obj.md"))
            .unwrap()
            .replace("concept_id = \"shell.a\"", "concept_id = \"shell.missing\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnresolvedConcept { .. })
    ));
}

#[test]
fn unresolved_objective_reference() {
    let root = temp_root("unresolved_objective_reference");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md"))
            .unwrap()
            .replace("objective_ids = [\"shell.obj\"]", "objective_ids = [\"shell.missing\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnresolvedObjective { .. })
    ));
}
```

- [ ] **Step 2: Run these five tests to verify they pass as written**

Run: `cd src-tauri && cargo test --locked knowledge::tests::conformance`
Expected: all five PASS immediately — this task adds no new production code, only proves the existing pipeline via its public entry point, so there is no red step here (a deliberate exception to the usual red/green cycle, since the behavior under test already exists and is already covered by unit tests; this task's job is proving it end-to-end, not building it).

- [ ] **Step 3: Add the remaining reference/relationship cases**

Append to `src-tauri/src/knowledge/tests/conformance.rs`:

```rust
#[test]
fn unresolved_source() {
    let root = temp_root("unresolved_source");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("source_id = \"src.a\"", "source_id = \"src.missing\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnresolvedSource { .. })
    ));
}

#[test]
fn cross_concept_objective() {
    let root = temp_root("cross_concept_objective");
    write_base_package(&root);
    fs::write(
        root.join("objectives/shell.obj.md"),
        fs::read_to_string(root.join("objectives/shell.obj.md"))
            .unwrap()
            .replace("concept_id = \"shell.a\"", "concept_id = \"shell.b\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::CrossConceptObjective { .. })
    ));
}

#[test]
fn self_prerequisite() {
    let root = temp_root("self_prerequisite");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.b.md"),
        fs::read_to_string(root.join("concepts/shell.b.md"))
            .unwrap()
            .replace("prerequisite_ids = [\"shell.a\"]", "prerequisite_ids = [\"shell.b\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::SelfReference { field: "prerequisite_ids", .. })
    ));
}

#[test]
fn prerequisite_cycle() {
    let root = temp_root("prerequisite_cycle");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("prerequisite_ids = []", "prerequisite_ids = [\"shell.b\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::PrerequisiteCycle { .. })
    ));
}

#[test]
fn self_related() {
    let root = temp_root("self_related");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("related_ids = []", "related_ids = [\"shell.a\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::SelfReference { field: "related_ids", .. })
    ));
}

#[test]
fn reverse_double_authored_related_edge() {
    let root = temp_root("reverse_double_authored_related_edge");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("related_ids = []", "related_ids = [\"shell.b\"]"),
    )
    .unwrap();
    fs::write(
        root.join("concepts/shell.b.md"),
        fs::read_to_string(root.join("concepts/shell.b.md"))
            .unwrap()
            .replace("related_ids = []", "related_ids = [\"shell.a\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::ReverseDuplicateRelated { .. })
    ));
}

#[test]
fn duplicate_id_inside_a_list() {
    let root = temp_root("duplicate_id_inside_a_list");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.b.md"),
        fs::read_to_string(root.join("concepts/shell.b.md"))
            .unwrap()
            .replace(
                "prerequisite_ids = [\"shell.a\"]",
                "prerequisite_ids = [\"shell.a\", \"shell.a\"]",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::DuplicateReferenceInList { .. })
    ));
}
```

- [ ] **Step 4: Add the provenance and schema cases**

Append to `src-tauri/src/knowledge/tests/conformance.rs`:

```rust
#[test]
fn missing_provenance() {
    let root = temp_root("missing_provenance");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace(
                "[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n",
                "",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::MissingProvenance { .. })
    ));
}

#[test]
fn unknown_provenance_kind() {
    let root = temp_root("unknown_provenance_kind");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("kind = \"direct\"", "kind = \"inferred\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnknownProvenanceKind { .. })
    ));
}

#[test]
fn duplicate_provenance_ref() {
    let root = temp_root("duplicate_provenance_ref");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace(
                "+++\n\nConcept A body.",
                "\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nConcept A body.",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::DuplicateProvenanceRef { .. })
    ));
}

#[test]
fn empty_source_locator() {
    let root = temp_root("empty_source_locator");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace(
                "kind = \"direct\"\n",
                "kind = \"direct\"\n[provenance_refs.locator]\n",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::EmptySourceLocator { .. })
    ));
}

#[test]
fn unsupported_schema_version() {
    let root = temp_root("unsupported_schema_version");
    write_base_package(&root);
    fs::write(
        root.join("package.toml"),
        fs::read_to_string(root.join("package.toml"))
            .unwrap()
            .replace("schema_version = 1", "schema_version = 2"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnsupportedSchemaVersion { found: 2 })
    ));
}

#[test]
fn unknown_schema_key() {
    let root = temp_root("unknown_schema_key");
    write_base_package(&root);
    fs::write(
        root.join("package.toml"),
        fs::read_to_string(root.join("package.toml")).unwrap() + "unexpected = \"typo\"\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::TomlSyntax { .. })
    ));
}
```

- [ ] **Step 5: Add the Example body grammar, file-layout, and frontmatter cases**

Append to `src-tauri/src/knowledge/tests/conformance.rs`:

```rust
#[test]
fn missing_example_problem() {
    let root = temp_root("missing_example_problem");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md"))
            .unwrap()
            .replace("## Problem\n\nP.\n\n", ""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::MissingExampleSection { section: "Problem", .. })
    ));
}

#[test]
fn missing_example_solution() {
    let root = temp_root("missing_example_solution");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md"))
            .unwrap()
            .replace("## Solution\n\nS.\n", ""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::MissingExampleSection { section: "Solution", .. })
    ));
}

#[test]
fn duplicate_example_heading() {
    let root = temp_root("duplicate_example_heading");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md")).unwrap() + "\n## Solution\n\nS2.\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::DuplicateExampleSection { .. })
    ));
}

#[test]
fn unknown_example_heading() {
    let root = temp_root("unknown_example_heading");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md")).unwrap() + "\n## Notes\n\nExtra.\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnknownExampleSection { .. })
    ));
}

#[test]
fn non_list_hints_content() {
    let root = temp_root("non_list_hints_content");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md")).unwrap()
            + "\n## Hints\n\nJust a paragraph.\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::InvalidHintLine { .. })
    ));
}

#[test]
fn malformed_toml_frontmatter() {
    let root = temp_root("malformed_toml_frontmatter");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        "+++\nid = \"shell.a\nname = \"A\"\n+++\n\nBody.\n", // unterminated string
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::TomlSyntax { .. })
    ));
}

#[test]
fn byte_order_mark() {
    let root = temp_root("byte_order_mark");
    write_base_package(&root);
    let content = fs::read_to_string(root.join("concepts/shell.a.md")).unwrap();
    fs::write(
        root.join("concepts/shell.a.md"),
        format!("\u{FEFF}{content}"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::Bom { .. })
    ));
}

#[test]
fn nested_entity_directory() {
    let root = temp_root("nested_entity_directory");
    write_base_package(&root);
    fs::create_dir_all(root.join("concepts/nested")).unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::NestedEntityDirectory { .. })
    ));
}

#[test]
fn unknown_entity_file_extension() {
    let root = temp_root("unknown_entity_file_extension");
    write_base_package(&root);
    fs::write(root.join("concepts/notes.txt"), "stray file").unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnexpectedEntityFile { .. })
    ));
}

#[test]
fn valid_crlf_fixture_is_accepted() {
    let root = temp_root("valid_crlf_fixture_is_accepted");
    write_base_package(&root);
    let crlf = fs::read_to_string(root.join("concepts/shell.a.md"))
        .unwrap()
        .replace('\n', "\r\n");
    fs::write(root.join("concepts/shell.a.md"), crlf).unwrap();
    assert!(load_knowledge_package(&root).is_ok());
}

#[test]
fn root_documentation_files_are_ignored() {
    let root = temp_root("root_documentation_files_are_ignored");
    write_base_package(&root);
    fs::write(root.join("synthesis-report.md"), "Human rationale, not schema data.\n").unwrap();
    fs::write(root.join("README.md"), "Non-schema documentation.\n").unwrap();
    assert!(load_knowledge_package(&root).is_ok());
}
```

- [ ] **Step 6: Register the tests module and run the full corpus**

In `src-tauri/src/knowledge/mod.rs`, add `#[cfg(test)] mod tests;` as the last line.

Run: `cd src-tauri && cargo test --locked knowledge::tests::conformance && cargo clippy --all-targets --locked -- -D warnings`
Expected: all ~24 conformance tests PASS; clippy clean.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/knowledge/tests/ src-tauri/src/knowledge/mod.rs
git commit -m "test(knowledge): add end-to-end conformance corpus"
```

---

### Task 14: Committed canonical valid fixture

**Files:**
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/package.toml`
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/sources.toml`
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/concepts/shell.method_vertical_axis.md`
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/concepts/shell.method_horizontal_axis.md`
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/concepts/shell.method_selection.md`
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/objectives/shell.setup_radius_height.md`
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/examples/shell.example_y_poly.md`
- Create: `src-tauri/src/knowledge/tests/canonical.rs`
- Modify: `src-tauri/src/knowledge/tests/mod.rs`

**Interfaces:**
- Consumes: `load_knowledge_package` (Task 12).

This is spec §17's canonical example, committed verbatim as real files (not constructed in a temp dir like Task 13's mutation cases) — it is simultaneously the happy-path integration test, the on-disk reference every future Knowledge Package author can copy from, and a permanent regression fixture. Every formula in it was independently checked by hand during the spec pass; this task transcribes that content exactly, without re-deriving it.

- [ ] **Step 1: Create the fixture files**

`src-tauri/src/knowledge/tests/fixtures/canonical/package.toml`:

```toml
id = "org.axiom.calculus_shells"
schema_version = 1
version = "1.0.0"
title = "Cylindrical Shells (Reference Example)"
description = "A minimal conforming Knowledge Package v1 example, drawn from OpenStax Calculus Volume 2 §2.3."
```

`src-tauri/src/knowledge/tests/fixtures/canonical/sources.toml`:

```toml
[[sources]]
id = "src.openstax_calc2"
title = "Calculus Volume 2"
authors = ["Gilbert Strang", "Edwin \"Jed\" Herman"]
edition = "2016"
license = "CC-BY-NC-SA-4.0"
```

`src-tauri/src/knowledge/tests/fixtures/canonical/concepts/shell.method_vertical_axis.md`:

```text
+++
id = "shell.method_vertical_axis"
name = "The Method of Cylindrical Shells (Vertical Axis)"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = ["shell.method_horizontal_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
+++

A method for calculating the volume of a solid of revolution by decomposing the
region into representative cylindrical shells and integrating with respect to `x`.
For rotation around the *y*-axis:

\[
V = \int_a^b 2\pi x f(x)\,dx
\]
```

`src-tauri/src/knowledge/tests/fixtures/canonical/concepts/shell.method_horizontal_axis.md`:

```text
+++
id = "shell.method_horizontal_axis"
name = "The Method of Cylindrical Shells (Horizontal Axis)"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = []

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule (x-axis)"
+++

The horizontal-axis counterpart: rotation around the *x*-axis, integrating with
respect to `y`:

\[
V = \int_c^d 2\pi y g(y)\,dy
\]
```

`src-tauri/src/knowledge/tests/fixtures/canonical/concepts/shell.method_selection.md`:

```text
+++
id = "shell.method_selection"
name = "Method Selection for Solids of Revolution"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = ["shell.method_vertical_axis", "shell.method_horizontal_axis"]
related_ids = []

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.17"
+++

Comparing the shells method against slicing to determine which yields a simpler
integral for a given region and axis of revolution.
```

`src-tauri/src/knowledge/tests/fixtures/canonical/objectives/shell.setup_radius_height.md`:

```text
+++
id = "shell.setup_radius_height"
concept_id = "shell.method_vertical_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"
+++

Identify and express the shell radius and shell height for a region revolved
around a vertical axis.
```

`src-tauri/src/knowledge/tests/fixtures/canonical/examples/shell.example_y_poly.md`:

```text
+++
id = "shell.example_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.13"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 4x - x^2` and below by the x-axis over `[0, 3]` around the *y*-axis.

## Solution

The shell radius is `r(x) = x`, height `h(x) = 4x - x^2`.

\[
V = \int_0^3 2\pi x(4x - x^2)\,dx = 2\pi\left[\frac{4x^3}{3} - \frac{x^4}{4}\right]_0^3
  = 2\pi\left(36 - \frac{81}{4}\right) = \frac{63\pi}{2}
\]

## Hints

- Identify the shell radius and height as functions of `x` before integrating.
```

- [ ] **Step 2: Write the failing test**

Create `src-tauri/src/knowledge/tests/canonical.rs`:

```rust
use std::path::Path;

use crate::knowledge::{load_knowledge_package, ProvenanceKind};

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical")
}

#[test]
fn canonical_fixture_loads_and_matches_spec_17() {
    let package = load_knowledge_package(&fixture_root()).unwrap();

    assert_eq!(package.id.as_str(), "org.axiom.calculus_shells");
    assert_eq!(package.schema_version, 1);
    assert_eq!(package.sources.len(), 1);
    assert_eq!(package.concepts.len(), 3);
    assert_eq!(package.objectives.len(), 1);
    assert_eq!(package.examples.len(), 1);

    let vertical = package
        .concepts
        .iter()
        .find(|c| c.id.as_str() == "shell.method_vertical_axis")
        .unwrap();
    assert_eq!(vertical.related_ids.len(), 1);
    assert!(vertical
        .provenance_refs
        .iter()
        .any(|r| r.kind == ProvenanceKind::Direct));
    assert!(vertical
        .provenance_refs
        .iter()
        .any(|r| r.kind == ProvenanceKind::Derived));

    let selection = package
        .concepts
        .iter()
        .find(|c| c.id.as_str() == "shell.method_selection")
        .unwrap();
    assert_eq!(selection.prerequisite_ids.len(), 2);

    let example = &package.examples[0];
    assert_eq!(example.solution.contains("63"), true);
    assert_eq!(example.hints.len(), 1);
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd src-tauri && cargo test --locked knowledge::tests::canonical`
Expected: FAIL — fixture files not written to disk yet at this point in a from-scratch execution. (If Step 1 was already completed, this instead serves as the pass-verification step; run it anyway to confirm.)

- [ ] **Step 4: Register the module and run to verify it passes**

In `src-tauri/src/knowledge/tests/mod.rs`, add `#[cfg(test)] mod canonical;` alongside the existing `mod conformance;`.

Run: `cd src-tauri && cargo test --locked knowledge::tests::canonical -- --nocapture && cargo clippy --all-targets --locked -- -D warnings`
Expected: PASS; clippy clean.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/knowledge/tests/fixtures/canonical/ src-tauri/src/knowledge/tests/canonical.rs src-tauri/src/knowledge/tests/mod.rs
git commit -m "test(knowledge): add committed canonical valid package fixture"
```

This closes the runtime workstream (Tasks 1–14: identifiers, error taxonomy, domain types, raw parsing, package/sources parsing, frontmatter, Concept/Objective/Example parsing, discovery, cross-reference validation, relationship validation, the atomic loader, and both the mutation-based and committed-fixture conformance suites). Every spec §18 case is covered; the canonical §17 example loads and matches. Tasks 15–17 are the second workstream: migrating the real `knowledge-package/` against this now-proven loader.

**Process note for whoever executes Task 15**: per `.ai/README.md` and `CLAUDE.md`, this migration should be tracked as its own `.ai/tasks/0NN-knowledge-package-v1-migration.md` entry (next id after whatever is highest at execution time) and reviewed by a different agent than whoever implements it, the same as every other task in this repository — this plan does not replace that process, it feeds the task's Plan section.

---

### Task 15: Migrate `knowledge-package/` to the v1 format

**Files:**
- Create: `knowledge-package/package.toml`
- Create: `knowledge-package/sources.toml`
- Create: `knowledge-package/concepts/shell.method_vertical_axis.md`
- Create: `knowledge-package/concepts/shell.method_horizontal_axis.md`
- Create: `knowledge-package/concepts/method_selection.volume_of_revolution.md`
- Create: `knowledge-package/objectives/shell.setup_radius_height_y_axis.md`
- Create: `knowledge-package/objectives/shell.compute_volume_y_axis_single_curve.md`
- Create: `knowledge-package/objectives/shell.compute_volume_y_axis_between_curves.md`
- Create: `knowledge-package/objectives/shell.compute_volume_shifted_vertical_axis.md`
- Create: `knowledge-package/objectives/shell.compute_volume_x_axis.md`
- Create: `knowledge-package/objectives/shell.select_optimal_method.md`
- Create: `knowledge-package/examples/shell.example_y_poly.md`
- Create: `knowledge-package/examples/shell.example_y_reciprocal.md`
- Create: `knowledge-package/examples/shell.example_y_between_curves.md`
- Create: `knowledge-package/examples/shell.example_shifted_vertical_axis.md`
- Create: `knowledge-package/examples/shell.example_x_axis.md`
- Create: `knowledge-package/examples/shell.example_setup_integrand.md`
- Delete: `knowledge-package/package.json`, `knowledge-package/provenance.json`, `knowledge-package/concepts/*.json` (3 files), `knowledge-package/objectives/*.json` (6 files), `knowledge-package/problem-families/` (entire directory, 7 files)
- Modify: `knowledge-package/synthesis-report.md` (rewritten, not deleted — see Task 17)

This task authors fresh content per the design brainstorm §5/§9 migration rules — it does not mechanically convert JSON to TOML+Markdown. Every ID is renamed to satisfy the identifier grammar (spec §2: no hyphens). The 11-entry `provenance.json` collapses to the package's single real `Source` (`src.openstax_calc2`) plus per-entity `SourceLocator`-based citations, per the brainstorm's headline reconciliation finding. All `problem-families/` generation/verification machinery (`generator`, `parameters`, `constraints`, `promptTemplate`, `canonicalSolution.expression`, `validator`, templated hints, `difficulty`, `status`) is dropped, not carried forward in any form — it remains available as design evidence only in this plan's own history and the brainstorm/spec documents, per spec §19. Six of the seven prior problem families become freshly-authored static `Example`s with one concrete, independently-verified parameter instantiation each; `pf-method-select-integral-count` is excluded entirely, per spec §19's corrected conclusion (it does not migrate; it remains design/source-review evidence until its unresolved source question is settled).

- [ ] **Step 1: Write `package.toml` and `sources.toml`**

`knowledge-package/package.toml`:

```toml
id = "org.axiom.reference.calculus.cylindrical_shells"
schema_version = 1
version = "0.2.0"
title = "Volumes of Revolution: Cylindrical Shells"
description = "Foundational Stage 8 reference package for computing volumes of revolution using the method of cylindrical shells, derived authoritatively from OpenStax Calculus Volume 2 Section 2.3."
```

`version` bumps from the prior `0.1.0` — this is a structural rewrite of the same content, not a patch. `knowledge-package/sources.toml`:

```toml
[[sources]]
id = "src.openstax_calc2"
title = "Calculus Volume 2"
authors = ["Gilbert Strang", "Edwin \"Jed\" Herman"]
edition = "2016"
license = "CC-BY-NC-SA-4.0"
```

This one entry replaces all 11 entries in the old `provenance.json` — the ten that described specific sections/rules/examples within the book become `SourceLocator`s on individual `ProvenanceRef`s below, not separate `Source`s.

- [ ] **Step 2: Write the three Concept files**

`knowledge-package/concepts/shell.method_vertical_axis.md`:

```text
+++
id = "shell.method_vertical_axis"
name = "The Method of Cylindrical Shells (Vertical Axis of Revolution)"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = ["shell.method_horizontal_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
+++

A method for calculating the volume of a solid of revolution by decomposing the
region into representative vertical cylindrical shells and integrating with
respect to `x`. For rotation around the *y*-axis:

\[
V = \int_a^b 2\pi x f(x)\,dx
\]

between two curves, `h(x) = f(x) - g(x)` replaces `f(x)`. For rotation around a
vertical line `x = k`, the radius is adjusted to `|x - k|`.
```

The `derived` ref (no `label`, just the section as a whole) records that separating this concept from `shell.method_horizontal_axis` as its own node — rather than leaving both under one undifferentiated §2.3 treatment the way OpenStax itself does — is Axiom's own structural synthesis, exactly as `synthesis-report.md`'s "Structural inferences" §1 already documented in prose; this makes that same fact a queryable `ProvenanceKind::Derived` ref instead.

`knowledge-package/concepts/shell.method_horizontal_axis.md`:

```text
+++
id = "shell.method_horizontal_axis"
name = "The Method of Cylindrical Shells (Horizontal Axis of Revolution)"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = []

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule (Solids of Revolution around the x-axis)"
+++

A method for calculating the volume of a solid of revolution revolved around the
*x*-axis (or horizontal lines) by decomposing the region into representative
horizontal cylindrical shells and integrating with respect to `y`:

\[
V = \int_c^d 2\pi y g(y)\,dy
\]
```

`related_ids` is empty here, not `["shell.method_vertical_axis"]` — the pair is already declared once, on `shell.method_vertical_axis` above; spec §10 requires it be authored on exactly one side.

`knowledge-package/concepts/method_selection.volume_of_revolution.md`:

```text
+++
id = "method_selection.volume_of_revolution"
name = "Method Selection for Solids of Revolution"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = ["shell.method_vertical_axis", "shell.method_horizontal_axis"]
related_ids = []

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.17"
+++

Comparative strategy to determine whether the slicing method (disks/washers) or
the cylindrical shells method yields a simpler integral formulation, based on
whether the cross-sections are perpendicular or parallel to the axis of
revolution and the number of sub-integrals required.
```

`prerequisite_ids` now carries what the old `relatedConceptIds` duplicated redundantly with — this concept genuinely depends on both shell-method concepts being understood first, which is what `prerequisite_ids` means; `related_ids` stays empty rather than repeating the same pair under a second relationship type, resolving the redundancy flagged as a non-blocking follow-up during task 049's review. This concept also assumes disk/washer-method knowledge this package has no concept for — that external dependency isn't representable in v1 (no cross-package references, spec §12) and isn't invented as a stub; it's noted here in the plan, and belongs in `synthesis-report.md`'s "Unresolved gaps" (Task 17), not in the package itself.

- [ ] **Step 3: Write the six Objective files**

`knowledge-package/objectives/shell.setup_radius_height_y_axis.md`:

```text
+++
id = "shell.setup_radius_height_y_axis"
concept_id = "shell.method_vertical_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.15"
+++

Identify and express the shell radius `r(x)` and shell height `h(x)` for a
region revolved around a vertical axis (the *y*-axis or a vertical line
`x = k`).
```

`knowledge-package/objectives/shell.compute_volume_y_axis_single_curve.md`:

```text
+++
id = "shell.compute_volume_y_axis_single_curve"
concept_id = "shell.method_vertical_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.13"
+++

Calculate the exact volume of a solid generated by revolving a region bounded
by a single continuous nonnegative function `y = f(x)`, the *x*-axis, and
vertical lines `x = a` and `x = b` around the *y*-axis.
```

`knowledge-package/objectives/shell.compute_volume_y_axis_between_curves.md`:

```text
+++
id = "shell.compute_volume_y_axis_between_curves"
concept_id = "shell.method_vertical_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.16"
+++

Calculate the exact volume of a solid generated by revolving a region bounded
between two continuous functions `y = f(x)` and `y = g(x)` over `[a, b]`
around the *y*-axis using `h(x) = f(x) - g(x)`.
```

`knowledge-package/objectives/shell.compute_volume_shifted_vertical_axis.md`:

```text
+++
id = "shell.compute_volume_shifted_vertical_axis"
concept_id = "shell.method_vertical_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.15"
+++

Calculate the exact volume of a solid generated by revolving a region around
a vertical line `x = k` other than the *y*-axis.
```

`knowledge-package/objectives/shell.compute_volume_x_axis.md`:

```text
+++
id = "shell.compute_volume_x_axis"
concept_id = "shell.method_horizontal_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule (Solids of Revolution around the x-axis)"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.14"
+++

Calculate the volume of a solid generated by revolving a region bounded by
`x = g(y)`, the *y*-axis, and horizontal lines `y = c` and `y = d` around the
*x*-axis by integrating with respect to `y`.
```

`knowledge-package/objectives/shell.select_optimal_method.md`:

```text
+++
id = "shell.select_optimal_method"
concept_id = "method_selection.volume_of_revolution"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.17"
+++

Determine the most efficient method (disks/washers vs. cylindrical shells)
for finding a volume of revolution by comparing integral count and algebraic
complexity.
```

- [ ] **Step 4: Write the six Example files**

Each is one concrete, independently-checked instantiation of a former problem family — not the general parameterized derivation, per spec §5/§19. `shell.example_y_poly` reuses the exact instantiation already verified in the spec's own §11 illustration (`coeff = 4, b = 3`); the other five are freshly chosen and independently checked here.

`knowledge-package/examples/shell.example_y_poly.md`:

```text
+++
id = "shell.example_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_y_axis_single_curve"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.13"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 4x - x^2` and below by the *x*-axis over `[0, 3]` around the *y*-axis.

## Solution

The shell radius is `r(x) = x`, height `h(x) = 4x - x^2`.

\[
V = \int_0^3 2\pi x(4x - x^2)\,dx = 2\pi\left[\frac{4x^3}{3} - \frac{x^4}{4}\right]_0^3
  = 2\pi\left(36 - \frac{81}{4}\right) = \frac{63\pi}{2}
\]

## Hints

- Identify the shell radius and height as functions of `x` for rotation around
  the *y*-axis.
- The shell radius is `r(x) = x` and the height is `h(x) = 4x - x^2`.
```

The `Example 2.13` ref is `derived`, not `direct` — the source example uses `f(x) = 2x - x^2` over `[0, 2]`; this instance uses different coefficients (`coeff = 4, b = 3`) that stay within the family Example 2.13 teaches but aren't a transcription of it.

`knowledge-package/examples/shell.example_y_reciprocal.md`:

```text
+++
id = "shell.example_y_reciprocal"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_y_axis_single_curve"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.12"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 3/x` and below by the *x*-axis over `[1, 4]` around the *y*-axis.

## Solution

The shell radius is `r(x) = x`, height `h(x) = 3/x`; the integrand simplifies
to a constant.

\[
V = \int_1^4 2\pi x \left(\frac{3}{x}\right) dx = 2\pi \int_1^4 3\,dx
  = 2\pi \cdot 3 \cdot (4 - 1) = 18\pi
\]

## Hints

- Write the shell volume formula and notice that `x` times `f(x)` simplifies
  to a constant.
- Integrating a constant over `[1, 4]` is just the constant times the
  interval length.
```

`knowledge-package/examples/shell.example_y_between_curves.md`:

```text
+++
id = "shell.example_y_between_curves"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_y_axis_between_curves"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.16"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 2x` and below by `g(x) = x^2` around the *y*-axis.

## Solution

The curves intersect at `x = 0` and `x = 2`. The shell height is
`h(x) = 2x - x^2`.

\[
V = \int_0^2 2\pi x(2x - x^2)\,dx = 2\pi\left[\frac{2x^3}{3} - \frac{x^4}{4}\right]_0^2
  = 2\pi\left(\frac{16}{3} - 4\right) = \frac{8\pi}{3}
\]

## Hints

- Find the *x*-limits of integration by setting the two functions equal.
- The shell height is the difference between the upper and lower curves.
```

`knowledge-package/examples/shell.example_shifted_vertical_axis.md`:

```text
+++
id = "shell.example_shifted_vertical_axis"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_shifted_vertical_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.15"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = x` and below by the *x*-axis over `[1, 2]` around the vertical line
`x = -1`.

## Solution

The shell radius is `r(x) = x - (-1) = x + 1`, height `h(x) = x`.

\[
V = \int_1^2 2\pi (x + 1)x\,dx = 2\pi \int_1^2 (x^2 + x)\,dx
  = 2\pi\left[\frac{x^3}{3} + \frac{x^2}{2}\right]_1^2 = \frac{23\pi}{3}
\]

## Hints

- The radius of a shell at position `x` is its distance to the axis of
  rotation `x = -1`, which is `x + 1`.
- Set up the integral with the shifted radius before evaluating.
```

This is a `direct` provenance ref — it uses the exact parameters (`f(x) = x`, `[1, 2]`, axis `x = -1`) from OpenStax's own Example 2.15, and the resulting `23π/3` matches the source exactly.

`knowledge-package/examples/shell.example_x_axis.md`:

```text
+++
id = "shell.example_x_axis"
concept_id = "shell.method_horizontal_axis"
objective_ids = ["shell.compute_volume_x_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.14"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded on the
right by `x = 2*sqrt(y)` and on the left by the *y*-axis, for `y` in `[0, 4]`,
around the *x*-axis.

## Solution

The shell radius is `r(y) = y`, height `h(y) = 2*sqrt(y)`.

\[
V = \int_0^4 2\pi y \left(2\sqrt{y}\right) dy = 4\pi \int_0^4 y^{3/2}\,dy
  = 4\pi \left[\frac{2}{5}y^{5/2}\right]_0^4 = \frac{256\pi}{5}
\]

## Hints

- When revolving around the *x*-axis with shells, the variable of
  integration is `y`.
- The shell radius is `r(y) = y` and the height is the horizontal extent
  `x = g(y)`.
```

This is also `direct` — it reproduces OpenStax's own Example 2.14 exactly (`c = 2, d = 4`), yielding the same `256π/5`.

`knowledge-package/examples/shell.example_setup_integrand.md`:

```text
+++
id = "shell.example_setup_integrand"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.15"
+++

## Problem

Consider the region bounded by `f(x) = x^3`, the *x*-axis, between `x = 0`
and `x = 1`, revolved around the line `x = -2`. Express the simplified
integrand (including `2*pi`) for finding the volume using the shell method —
do not evaluate the integral.

## Solution

The radius is `r(x) = x - (-2) = x + 2`, the height is `h(x) = x^3`.

\[
2\pi \cdot r(x) \cdot h(x) = 2\pi(x + 2)x^3 = 2\pi\left(x^4 + 2x^3\right)
\]

## Hints

- Identify the shell radius from the axis `x = -2` before multiplying by the
  height.
- Multiply out `(x + 2) \cdot x^3` to reach the simplified integrand.
```

This is a diagnostic, setup-only instance — no numeric evaluation, isolating integrand formulation from antidifferentiation, matching the original family's stated educational purpose.

`pf-method-select-integral-count` is deliberately **not** migrated to an `Example` file. It stays excluded, per spec §19: the underlying disk/washer-vs-shell integral count for the region it describes contradicts this package's own `shell.method_vertical_axis` concept and the corresponding claim in the pre-migration `provenance.json`, and that contradiction was never resolved against the actual source text (unavailable in this repository). No `.md` file for it exists anywhere under `knowledge-package/` after this task.

- [ ] **Step 5: Remove the pre-v1 files**

```bash
git rm knowledge-package/package.json knowledge-package/provenance.json
git rm knowledge-package/concepts/shell-method-vertical-axis.json
git rm knowledge-package/concepts/shell-method-horizontal-axis.json
git rm knowledge-package/concepts/method-selection-volume-of-revolution.json
git rm knowledge-package/objectives/*.json
git rm -r knowledge-package/problem-families/
```

- [ ] **Step 6: Sanity-check the migrated package loads before committing**

Run:

```bash
cd src-tauri && cargo run --locked --bin axiom -- --check-knowledge-package ../knowledge-package
```

This binary flag does not exist yet and is **not** part of this plan — Task 16 builds the actual verification harness with real acceptance criteria. For this step, instead write and run a throwaway local test (not committed) exercising `load_knowledge_package(Path::new("../knowledge-package"))` and printing the resulting counts, to catch typos (a mistyped id, a forgotten heading, a bad TOML table) before committing 16 new files. Delete the throwaway test before the commit in Step 7 — Task 16 is where the permanent version of this check is added.

- [ ] **Step 7: Commit**

```bash
git add knowledge-package/
git commit -m "feat(knowledge-package): migrate Calc II reference content to Knowledge Package v1"
```

---

### Task 16: Migration validation

**Files:**
- Create: `src-tauri/src/knowledge/tests/migration.rs`
- Modify: `src-tauri/src/knowledge/tests/mod.rs`

**Interfaces:**
- Consumes: `load_knowledge_package` (Task 12); the migrated `knowledge-package/` (Task 15).

This is the permanent version of Task 15 Step 6's throwaway check — a committed regression test proving the real migrated package structurally validates against the real loader, plus explicit content assertions so the migration can't silently drop an entity while still technically loading.

- [ ] **Step 1: Write the test**

Create `src-tauri/src/knowledge/tests/migration.rs`:

```rust
use std::path::Path;

use crate::knowledge::load_knowledge_package;

fn migrated_package_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package")
}

#[test]
fn migrated_calc_ii_package_loads_and_is_structurally_complete() {
    let package = load_knowledge_package(&migrated_package_root())
        .expect("migrated knowledge-package/ must load and pass every spec §12/§13 invariant: \
                 all references resolve, the prerequisite graph is acyclic, related_ids is \
                 authored on at most one side, and every entity has at least one provenance ref");

    assert_eq!(package.id.as_str(), "org.axiom.reference.calculus.cylindrical_shells");
    assert_eq!(package.schema_version, 1);
    assert_eq!(package.sources.len(), 1, "provenance.json's 11 entries must have collapsed to exactly 1 Source");

    assert_eq!(package.concepts.len(), 3);
    assert_eq!(package.objectives.len(), 6);
    assert_eq!(
        package.examples.len(),
        6,
        "6 of the 7 prior problem families become Examples; pf-method-select-integral-count \
         does not migrate"
    );

    let example_ids: Vec<&str> = package.examples.iter().map(|e| e.id.as_str()).collect();
    assert!(!example_ids.iter().any(|id| id.starts_with("pf-")));
    assert!(!example_ids.contains(&"shell.example_method_select_integral_count"));

    for concept in &package.concepts {
        assert!(
            !concept.provenance_refs.is_empty(),
            "{} has no provenance_refs",
            concept.id
        );
    }
    for objective in &package.objectives {
        assert!(
            !objective.provenance_refs.is_empty(),
            "{} has no provenance_refs",
            objective.id
        );
    }
    for example in &package.examples {
        assert!(
            !example.provenance_refs.is_empty(),
            "{} has no provenance_refs",
            example.id
        );
        assert!(!example.problem.is_empty());
        assert!(!example.solution.is_empty());
    }
}

#[test]
fn no_deprecated_json_or_problem_families_artifacts_remain() {
    let root = migrated_package_root();
    assert!(!root.join("package.json").exists());
    assert!(!root.join("provenance.json").exists());
    assert!(!root.join("problem-families").exists());
    for entry in std::fs::read_dir(root.join("concepts")).unwrap() {
        let path = entry.unwrap().path();
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("md"));
    }
    for entry in std::fs::read_dir(root.join("objectives")).unwrap() {
        let path = entry.unwrap().path();
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("md"));
    }
}
```

- [ ] **Step 2: Register the module and run**

In `src-tauri/src/knowledge/tests/mod.rs`, add `#[cfg(test)] mod migration;`.

Run: `cd src-tauri && cargo test --locked knowledge::tests::migration -- --nocapture`
Expected: both tests PASS. If `migrated_calc_ii_package_loads_and_is_structurally_complete` fails, the `.expect()` message names exactly which spec invariant to check against `knowledge-package/`'s actual files — fix the migration (Task 15), not this test.

- [ ] **Step 3: Run the full workspace test suite one final time**

Run: `cd src-tauri && cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: every test in the crate passes (modules::* from sub-project 1, unchanged; knowledge::* from this plan), clippy clean, formatting clean.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/knowledge/tests/migration.rs src-tauri/src/knowledge/tests/mod.rs
git commit -m "test(knowledge): add permanent migration regression test"
```

**What this proves, and what it deliberately doesn't**: this test proves the migrated package is *structurally* valid — every reference resolves, the graph invariants hold, nothing deprecated leaked through. It does not re-verify mathematical correctness or source fidelity; that was already established by hand across the original package's two review passes (documented in `.ai/tasks/_archive/049-knowledge-package-shell-fixes.md`), and spec §20 is explicit that structural validation must never be represented as a substitute for that. If a future change touches the migrated content's math, it needs the same manual scrutiny the original review gave it, not just a green `cargo test`.

---

### Task 17: Documentation cleanup

**Files:**
- Modify: `ARCHITECTURE.md`
- Modify: `knowledge-package/synthesis-report.md`

**Interfaces:** None — this task touches no code.

Only documentation made stale by adopting Knowledge Package v1. No unrelated architecture rewrites.

- [ ] **Step 1: Update `ARCHITECTURE.md`**

In `ARCHITECTURE.md`, find the line (added during task 049):

```text
knowledge-package/        # Stage 8 reference content; ad hoc until Knowledge Package v1
```

Replace it with:

```text
knowledge-package/        # Stage 8 reference content, Knowledge Package v1 format —
                          # see docs/superpowers/specs/2026-08-30-knowledge-package-v1-spec.md
```

Preserve the surrounding line's indentation/alignment style (match the column the comment starts at for neighboring entries, the same convention task 049's review checked).

- [ ] **Step 2: Rewrite `knowledge-package/synthesis-report.md`**

This file stays — spec §7 and the brainstorm both keep it as non-schema authoring/rationale documentation, explicitly never loaded by the runtime parser. Rewrite its content to match the new shape:

- Update the "Concepts selected" / "Objectives" / "Problem families" sections' ID references to the new grammar-compliant IDs (`shell.method_vertical_axis`, etc.), matching Task 15's files exactly.
- Add a note under "Structural inferences" recording that the 11 flat provenance entries collapsed into 1 `Source` plus locator-based citations, and that each `Concept`'s `Derived` vs `Direct` provenance refs now make the earlier prose-only "structural inference" claims machine-queryable.
- Move the `method_selection.volume_of_revolution` → disk/washer external-dependency gap (Task 15, Step 2) into "Unresolved gaps" explicitly, since it's no longer representable as a field in the package itself.
- Update "Rejected candidates" / "Problem families" sections to record that `pf-method-select-integral-count` was excluded from the v1 migration rather than converted, with a pointer to `.ai/tasks/_archive/049-knowledge-package-shell-fixes.md` for why.
- Leave "Human review priorities" as-is unless a specific item is now resolved by this migration (none are).

No code review gate applies to this step — it's prose — but per this repository's own review discipline (`.ai/review-checklist.md`), whoever reviews Task 17 should still confirm every ID mentioned in the rewritten report actually exists in `knowledge-package/` after Task 15, the same way any other cross-reference in this repository gets checked.

- [ ] **Step 3: Commit**

```bash
git add ARCHITECTURE.md knowledge-package/synthesis-report.md
git commit -m "docs: update ARCHITECTURE.md and synthesis-report.md for Knowledge Package v1"
```

This is the plan's final task. `ARCHITECTURE.md` no longer calls the package "ad hoc," `ROADMAP.md` needs no change (it already lists "Knowledge Package v1 schema" as roadmap item 1 without asserting completion status inline — updating its own checklist-of-sorts, if this repository tracks one elsewhere, is a process step outside this plan's file list).
