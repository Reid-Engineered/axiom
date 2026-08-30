# Knowledge Package v1 — formal specification

Freezes the architecture accepted in
`docs/superpowers/specs/2026-08-30-knowledge-package-v1-design.md` (the brainstorm) into an
exact, normative contract. Two independent implementations conforming to this document MUST
produce the same accept/reject verdict for any given package, and MUST produce equivalent
typed values for any package both accept.

This document does not reopen architecture. Where the brainstorm left a contract detail
open, this document resolves it, with reasoning, in place. No accepted architectural
decision changed during this pass.

Normative keywords `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY` are used as commonly
understood (RFC 2119 sense) throughout.

---

## 1. Scope and terminology

**Five primary entities**: `KnowledgePackage`, `Concept`, `Objective`, `Example`, `Source`.

**Supporting types**: `KnowledgePackageId`, `ConceptId`, `ObjectiveId`, `ExampleId`,
`SourceId`, `ProvenanceRef`, `ProvenanceKind`, `SourceLocator`.

**Domain boundary** (unchanged from the brainstorm §5; restated here because it governs what
this specification is permitted to contain):

```text
Knowledge          = declarative subject knowledge
Canonical Problem   = declarative assessment/generation specification
Practice             = learner-facing orchestration and policy
math.verify           = mathematical correctness/equivalence computation
Tutor                 = pedagogical interaction behavior
```

A conforming Knowledge Package MUST NOT contain: problem generators, parameter ranges,
generator constraints, verifier/provider declarations, learner state, mastery state,
adaptive policy, review/editorial status, Tutor prompt templates, or runtime code. A loader
that encounters a field shaped like any of the above is not implementing this
specification.

## 2. Identifier specification

Knowledge identifier grammar is lexically identical to the grammar implemented and tested by
`src-tauri/src/modules/identifier.rs`'s `validate_identifier`, transcribed here exactly from
that implementation (read directly, not restated from memory):

> A value is a valid identifier if and only if:
> 1. Splitting the value on `.` (U+002E) produces two or more segments.
> 2. Every segment is non-empty.
> 3. Every segment's first character is an ASCII lowercase letter (`a`–`z`).
> 4. Every subsequent character in that segment is an ASCII lowercase letter, an ASCII
>    digit (`0`–`9`), or `_` (U+005F).

Formally, each segment MUST match `^[a-z][a-z0-9_]*$`, and the full value MUST match
`^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$`.

Consequences, stated explicitly because they are easy to get wrong:

- **Uppercase is rejected everywhere** in the identifier, not just the first character.
- **Hyphens are rejected everywhere.** `shell-method` is not a valid segment;
  `shell.method_vertical_axis` is.
- **Leading dot, trailing dot, and consecutive dots are rejected** — each produces an empty
  segment, and rule 2 rejects empty segments.
- **A single segment (no dot) is rejected** by rule 1, regardless of its own validity.
- **No length limit is enforced.** The current Rust implementation imposes none, and this
  specification imposes none either. A future revision MAY add one; v1 does not.

Confirmed against `identifier.rs`'s own test fixtures (task 048,
`identifiers_enforce_the_locked_grammar`): `practice.generate`, `org.axiom.practice`, `a.b`,
`a0.b_1` are valid; `Practice`, `practice`, `practice..generate`, `.practice`, `practice.`,
`practice.Generate`, `practice.generate-more` are invalid.

**Knowledge identifiers use distinct semantic types** — `KnowledgePackageId`, `ConceptId`,
`ObjectiveId`, `ExampleId`, `SourceId` — each validated against the grammar above, but never
interchangeable with each other or with `ModuleId`/`CapabilityId`. An implementation in this
repository SHOULD reuse `identifier.rs`'s `validate_identifier` function directly rather than
reimplementing the grammar; this is an implementation recommendation, not part of the
interoperability contract — any implementation, in any language, that accepts and rejects
values exactly per the grammar above conforms.

**Segments carry no semantic weight beyond satisfying the grammar.** Dot-separated
identifier segments have no implied hierarchy, taxonomy, parentage, package ownership, or
namespace semantics. `shell.method_vertical_axis` is an opaque stable identifier; the fact
that it contains the substring `shell` implies nothing structural.

**Entity IDs are package-local and MUST NOT be prefixed by package ID.**

```text
KnowledgePackageId:  org.axiom.calculus_shells
ConceptId:            shell.method_vertical_axis
```

not `org.axiom.calculus_shells.shell.method_vertical_axis`.

**Uniqueness is per entity kind, not package-global.**

```text
ConceptId    MUST be unique among a package's Concepts
ObjectiveId  MUST be unique among a package's Objectives
ExampleId    MUST be unique among a package's Examples
SourceId     MUST be unique among a package's Sources
```

Identical lexical values MAY exist across different entity kinds — `ConceptId("shell.basic")`
and `ObjectiveId("shell.basic")` coexisting in one package is legal. Every reference site in
this schema is statically typed by entity kind (§12), so no ambiguity results.

## 3. Package layout

```text
<package-root>/
├── package.toml
├── sources.toml
├── concepts/
│   └── <ConceptId>.md
├── objectives/
│   └── <ObjectiveId>.md
└── examples/
    └── <ExampleId>.md
```

- `package.toml` MUST exist.
- `sources.toml` MUST exist. Provenance is required on every `Concept`/`Objective`/`Example`
  (§11), so any package containing at least one entity necessarily needs at least one
  `Source`; a package with zero entities of every kind is degenerate but not itself
  forbidden by this section, and MAY have an empty `[[sources]]` collection.
- `concepts/`, `objectives/`, `examples/` MAY each be absent when the package has zero
  entities of that kind (git does not track empty directories; requiring their literal
  presence would be impractical). When present, each directory contains zero or more `.md`
  files.
- Nested subdirectories inside `concepts/`, `objectives/`, or `examples/` are NOT permitted
  and MUST cause validation failure if present.
- Any file inside `concepts/`, `objectives/`, or `examples/` that is not a `.md` file, or
  whose name does not match `<EntityId>.md` for some entity ID valid per §2, MUST cause
  validation failure. These directories are treated strictly, deliberately: silent
  acceptance of a stray or misnamed file would hide authoring mistakes.
- **Filename MUST exactly equal the entity's `id` field plus `.md`.** A file
  `concepts/shell.method_vertical_axis.md` whose frontmatter declares
  `id = "shell.other_name"` MUST be rejected as a filename/ID mismatch.
- Unknown files at the package root — anything other than `package.toml`, `sources.toml`,
  `concepts/`, `objectives/`, `examples/` — MAY exist and MUST be ignored by a Knowledge
  loader (§15). This is what allows non-schema authoring documentation
  (`synthesis-report.md`, a `README.md`) to coexist with the canonical package.
- **Discovery order is not semantically significant.** Raw filesystem enumeration order
  (which varies across operating systems and filesystems) MUST NOT affect the loaded,
  validated `KnowledgePackage` value. Where an implementation exposes deterministic
  iteration over a directory's entities, it SHOULD sort by entity ID (§16 covers ordering
  significance generally).
- `package.toml` MUST NOT contain hand-authored inventories of concept/objective/example
  IDs. The set of entities is exactly what discovery (this section) finds on disk.

## 4. `package.toml`

```rust
struct KnowledgePackage {
    id: KnowledgePackageId,
    schema_version: u32,
    version: semver::Version,
    title: String,
    description: String,
}
```

```toml
id = "org.axiom.calculus_shells"
schema_version = 1
version = "1.0.0"
title = "Cylindrical Shells"
description = "A tiny Calculus II reference knowledge package."
```

- `id`, `schema_version`, `version`, `title`, `description` are all **required**. There are
  no optional fields in `package.toml`.
- Unknown top-level keys MUST be rejected.
- `id` MUST satisfy §2's grammar.
- `version` MUST parse as a semantic version (`semver::Version` in Rust terms — the same
  crate and parsing rule `module.toml`'s own `version`/`minimum_axiom_version` fields
  already use). A malformed value is a structural (parse-layer) failure, not a semantic
  one.
- `schema_version` is **the version of the Knowledge Package schema this document
  defines** — this document is schema_version 1. It is entirely distinct from `version`,
  which is **this package's own content revision**, independent of and unaffected by
  `schema_version`. §14 defines compatibility behavior for `schema_version`.
- `title` and `description` are plain display strings with no further constraint beyond
  being non-empty (an empty `title`/`description` MUST be rejected — a blank display name
  or description is an authoring mistake, not a legitimate degenerate case).

## 5. `sources.toml`

```rust
struct Source {
    id: SourceId,
    title: String,
    authors: Vec<String>,
    edition: Option<String>,
    license: Option<String>,
}
```

```toml
[[sources]]
id = "src.openstax_calc2"
title = "Calculus Volume 2"
authors = ["Gilbert Strang", "Edwin \"Jed\" Herman"]
edition = "2016"
license = "CC-BY-NC-SA-4.0"
```

- `sources.toml`'s only permitted top-level construct is the `[[sources]]` array of tables.
  Any other top-level key MUST be rejected.
- Within one `[[sources]]` entry: `id` and `title` are **required**; `authors`, `edition`,
  `license` are **optional**. Unknown keys within a `[[sources]]` table MUST be rejected.
- `authors` MAY be an empty array, and the key MAY be omitted entirely (equivalent to an
  empty array) — a source with institutional, uncertain, or collaborative authorship is not
  required to invent named authors.
- `edition` MAY be absent.
- `license` MAY be absent. When present, it SHOULD contain a stable license identifier where
  one exists (SPDX-style, e.g. `"CC-BY-NC-SA-4.0"`) rather than free prose describing terms.
  This specification does not require SPDX conformance where a source's actual license has
  no clean SPDX mapping, and defines no license compatibility, redistribution, or
  attribution-rendering policy — those are later, separate concerns.
- `id` MUST satisfy §2's grammar and MUST be unique among the package's `Source` entries;
  a duplicate `SourceId` MUST cause validation failure.
- **The order of `[[sources]]` entries is not semantically significant.** Any permutation of
  a valid `sources.toml`'s entries is an equivalent package. (This is distinct from the
  order of names *within* one `Source.authors` list — see §16.)

## 6. TOML frontmatter grammar

Applies to every `Concept`, `Objective`, and `Example` file.

- The file MUST be UTF-8. A byte-order mark (BOM) MUST be rejected outright.
- **The opening delimiter, `+++`, MUST be the first line of the file** — no leading blank
  lines or whitespace before it. This gives every conforming file one canonical shape rather
  than several visually-equivalent ones.
- The closing delimiter is the next line, after the opening one, that is exactly `+++` with
  no surrounding characters. Content strictly between the two delimiter lines MUST be valid
  TOML; a TOML syntax error there MUST cause the whole file to fail loading (this mirrors
  `manifest.rs`'s `ManifestError::TomlSyntax` pattern — a syntax failure at the raw-parse
  layer, distinct from a semantic validation failure).
- If the file does not open with `+++`, or opens with `+++` but no closing `+++` line is
  ever found, loading MUST fail.
- Everything after the closing delimiter's line is the entity's Markdown body. Leading blank
  lines immediately following the closing delimiter MAY be trimmed by the parser and carry
  no semantic weight; trailing blank lines at end of file MAY likewise be trimmed. Blank
  lines *within* the body (paragraph breaks) MUST be preserved exactly as authored — they
  are meaningful Markdown structure, not incidental whitespace.
- The parser MUST accept both LF and CRLF line endings; a file is not rejected for using
  CRLF. This repository's own tooling SHOULD emit LF for committed files, as a convention,
  not a parser-enforced rule.
- **Unknown frontmatter keys MUST be rejected.** Typo detection and deterministic authoring
  matter more in v1 than forward-compatible loose parsing.

## 7. `Concept` specification

```rust
struct Concept {
    id: ConceptId,
    name: String,
    topic: Option<String>,
    description: String,
    prerequisite_ids: Vec<ConceptId>,
    related_ids: Vec<ConceptId>,
    provenance_refs: Vec<ProvenanceRef>,
}
```

```toml
+++
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

A method for calculating the volume of a solid of revolution...
```

- `id`, `name` are **required**. `topic` is **optional**. `prerequisite_ids`, `related_ids`
  MAY be omitted, each defaulting to an empty list. `provenance_refs` MUST be present with
  at least one entry (§11) — an omitted or empty `provenance_refs` is a validation failure,
  distinct from the parse-layer default of an empty list for the relationship fields.
- The **entire remaining Markdown body, with no required heading**, maps to
  `Concept.description`. `description` MUST NOT be empty after the leading/trailing
  trimming described in §6 — a `Concept` with no explanatory content is not meaningfully
  knowledge.
- `prerequisite_ids` and `related_ids` are validated per §10.
- `provenance_refs` is validated per §11.
- This specification does not define Markdown rendering semantics for `description`'s
  content — only that it is UTF-8 Markdown text, preserved verbatim.

## 8. `Objective` specification

```rust
struct Objective {
    id: ObjectiveId,
    concept_id: ConceptId,
    description: String,
    provenance_refs: Vec<ProvenanceRef>,
}
```

```toml
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

- `id`, `concept_id` are **required**. `provenance_refs` MUST be present with at least one
  entry.
- **There is no `evidence` field, no `example_ids` field, no reverse concept list, and no
  mastery-criteria field.** An implementation that accepts any of these is not implementing
  this specification.
- `concept_id` MUST resolve to an existing `Concept` in the same package (§12).
- The entire remaining Markdown body, with no required heading, maps to
  `Objective.description`, subject to the same non-empty rule as `Concept.description`.

## 9. `Example` specification

```rust
struct Example {
    id: ExampleId,
    concept_id: ConceptId,
    objective_ids: Vec<ObjectiveId>,
    problem: String,
    solution: String,
    hints: Vec<String>,
    provenance_refs: Vec<ProvenanceRef>,
}
```

- `id`, `concept_id` are **required**. `objective_ids` MAY be omitted, defaulting to an
  empty list — an `Example` MAY illustrate a `Concept` without being tied to any formal
  `Objective`. `provenance_refs` MUST be present with at least one entry.
- `concept_id` MUST resolve to an existing `Concept` in the same package.
- Every ID in `objective_ids` MUST resolve to an existing `Objective` in the same package.

**Resolved contract detail — cross-concept objectives are not permitted.** Every
`ObjectiveId` in `objective_ids` MUST reference an `Objective` whose own `concept_id` equals
this `Example`'s `concept_id`. `Example.concept_id` is singular by design (§7 of the
brainstorm) — allowing `objective_ids` to scatter across unrelated concepts would make an
`Example`'s single declared "home" concept misleading, since it could silently demonstrate
objectives belonging elsewhere with no way to express that at the concept level. Every
`Example` in the current reference package already satisfies this constraint without
exception (confirmed by direct inspection of all seven problem-family prototypes during the
original review). If a genuine need for a multi-concept `Example` emerges later, that is an
argument for evolving this field (e.g. a `Vec<ConceptId>`), not for relaxing this constraint
while `concept_id` stays singular.

### Body grammar

```markdown
## Problem

...

## Solution

...

## Hints

- ...
- ...
```

The parser recognizes **exactly** three level-2 headings, `## Problem`, `## Solution`,
`## Hints`, and nothing else. This is a narrow, closed grammar — not a general Markdown
parser — and the following rules are exhaustive:

- Any content before the first recognized heading MUST be ignorable whitespace only
  (blank lines). Non-whitespace content before `## Problem` MUST be rejected.
- `## Problem` MUST appear. Its content MUST NOT be empty after trimming.
- `## Solution` MUST appear, immediately after `## Problem`'s content, before any further
  recognized heading. Its content MUST NOT be empty after trimming.
- `## Hints` is **optional**. If present, it MUST appear after `## Solution` — no other
  position is valid. If absent, `hints` is an empty `Vec`.
- Headings MUST appear in the fixed order Problem, then Solution, then (optionally) Hints.
  Any other order MUST be rejected.
- A duplicate occurrence of any recognized heading MUST be rejected.
- **Any level-2 (`##`) heading other than the three recognized ones MUST be rejected.**
  Deeper headings (`###` and below) and all other Markdown constructs (bold, italic, links,
  blockquotes, code spans/blocks, nested lists) are **not** parsed or validated at all —
  they are opaque content, copied verbatim into whichever field's substring they fall
  within. The parser's only job is locating `##`-level section boundaries.
- **`## Hints` content grammar**: if the `## Hints` heading is present, it MUST contain at
  least one hint (an empty `## Hints` section is rejected — omit the heading entirely for
  zero hints). Each hint is exactly one line beginning with `- ` (a single hyphen, a single
  space) at zero indentation; the hint's text is everything after that prefix on the same
  line, trimmed. `*` and `+` bullet markers are not recognized — only `-`. Blank lines
  between hint lines are permitted and ignorable. **Multi-line hints are not supported in
  v1**: any line under `## Hints` that is not blank and does not match the `- <text>`
  pattern (an indented continuation line, a numbered list item, a differently-bulleted item,
  or bare prose) MUST be rejected.

## 10. Relationship semantics

### `prerequisite_ids`

Directed. If `A.prerequisite_ids` contains `B`, then `B` is a prerequisite of `A` — `B` must
be understood before `A`.

- Every ID MUST resolve to an existing `Concept` in the same package.
- Self-reference (`A` appearing in its own `prerequisite_ids`) MUST be rejected.
- Duplicate IDs within one `prerequisite_ids` list MUST be rejected.
- **The entire package's prerequisite graph MUST be acyclic** — a directed cycle among
  `prerequisite_ids` edges anywhere in the package is a package-level validation failure,
  checked after every individual `Concept`'s own field-level validation succeeds (cycle
  detection requires the whole graph assembled).
- No reverse edge is ever serialized or implied. A consumer MAY derive a "dependents of"
  view by scanning the graph, but that view is never authored.

### `related_ids`

Semantically symmetric; authored once, on exactly one endpoint, by either endpoint's author.

- Every ID MUST resolve to an existing `Concept` in the same package.
- Self-reference MUST be rejected.
- Duplicate IDs within one `related_ids` list MUST be rejected.
- **If `A.related_ids` contains `B`, then `B.related_ids` MUST NOT also contain `A`.** This
  is a validation failure — a "reverse duplicate declaration" — not a case to silently
  deduplicate. The pair is declared exactly once, by whichever concept's author chose to
  author it.
- A loader's in-memory model exposes the relation symmetrically regardless of which side
  authored it (querying "related to `B`" returns `A` even though only `A`'s file declares
  it). A future package-writing tool MUST preserve only the originally-authored side when
  serializing a package back out — it MUST NOT mechanically write the derived reverse edge
  into the other file.
- `related_ids` carries **no acyclic constraint**. A "cycle" (`A` related `B` related `C`
  related `A`) has no special meaning and is not an error — `related` has no ordering
  semantics, so "cycle" is not even a coherent concept for it the way it is for
  `prerequisite_ids`.

No generic `Relationship` entity exists in this schema. These are the only two relationship
fields v1 defines.

## 11. Provenance specification

```rust
enum ProvenanceKind {
    Direct,
    Derived,
}

struct ProvenanceRef {
    source_id: SourceId,
    locator: Option<SourceLocator>,
    kind: ProvenanceKind,
}

struct SourceLocator {
    section: Option<String>,
    pages: Option<String>,
    label: Option<String>,
}
```

Serialized values are lowercase, case-sensitive: `kind = "direct"` or `kind = "derived"`.
Any other value (including `"Direct"`, `"DIRECT"`, or anything not in this set) MUST be
rejected.

**Normative semantics** (not mechanically provable, stated for consistent authoring and
review):

```text
Direct:  the entity substantially represents content explicitly present at the
         cited source location.
Derived: the entity contains synthesis, reorganization, generalization, inference,
         or newly-authored pedagogical structure based on the cited source location.
```

- `source_id` is **required** and MUST resolve to an existing `Source` in the same package.
- `locator` is **optional** — a citation MAY reference a `Source` as a whole (no specific
  section/page/label) by omitting `locator` entirely.
- **If a `locator` table is present at all, at least one of `section`, `pages`, `label`
  MUST be non-empty.** An entirely empty locator table conveys no information; omit
  `locator` instead of supplying an empty one.
- `kind` is **required**.
- An entity MAY carry both `Direct` and `Derived` `ProvenanceRef`s simultaneously.
- **Exact duplicates MUST be rejected.** Two `ProvenanceRef`s on the same entity with
  identical `(source_id, locator, kind)` are a duplicate — rejected. Two refs sharing a
  `source_id` but differing in `locator` and/or `kind` are the normal multi-citation case
  and are permitted.
- **Every `Concept`, `Objective`, and `Example` MUST have at least one `ProvenanceRef`.**
  This is required, not optional, based on two pieces of evidence: every entity in the
  current reference package already has one (confirmed across two full review passes, no
  orphans found), and the package's own stated premise is source-grounded content.
  Axiom-original content satisfies this requirement the same way anything else does: by
  declaring a `Source` that represents Axiom's own authorship (e.g.
  `id = "org.axiom_original"`, no external license required) and citing it. This is honest
  provenance — "this is deliberately original" — not a workaround.

**`SourceLocator` field semantics:**

```text
section  — a human/source-facing section identifier, e.g. "2.3"
pages    — a human/source-facing page or page-range string, e.g. "137-152"
label    — the element exactly as the source names it, e.g. "Example 2.13", "Rule 2.6"
```

`section` and `pages` remain free-form `String`s in v1; structured page ranges
(`{from, to}`) are deferred until a consumer needs range arithmetic or sorting.

**These fields MUST NOT contain ingestion-tool-local identifiers** (a Docling block ID, a
span offset, an OCR bounding box) as part of the authoring contract. This is a **normative
semantic requirement, not a mechanically enforceable validation rule** — no automated check
can prove a given string isn't secretly a tool-internal ID rather than a genuine
human-facing label; enforcement is an authoring and review discipline, the same discipline
this repository's own task/review process already applies to everything else.

**Provenance is entity-level only in v1.** There is no field-level or content-block-level
provenance (no `problem_provenance`, `solution_provenance`, separate hint provenance). This
was checked specifically against the current package for a concrete case where entity-level
provenance would mislead a consumer, and none was found; it is an explicit non-goal (§13),
revisited only if such a case is demonstrated.

## 12. Reference validation

Every reference edge in this schema, collected in one place:

```text
Objective.concept_id           → Concept
Example.concept_id             → Concept
Example.objective_ids          → Objective*   (each MUST share Example's concept_id, §9)
Concept.prerequisite_ids       → Concept*     (MUST form a DAG, §10)
Concept.related_ids            → Concept*     (symmetric-once rule, §10)
ProvenanceRef.source_id        → Source
```

All references are **package-local** (§2) — no reference resolves outside the declaring
package, and this specification defines no cross-package reference syntax at all.

**Every reference MUST resolve.** An unresolved reference of any kind — a `concept_id`
naming no `Concept`, an `objective_id` naming no `Objective`, a `source_id` naming no
`Source`, a `prerequisite_ids`/`related_ids` entry naming no `Concept` — MUST fail package
validation.

**Knowledge Package validation is atomic at the package boundary.** No partial package
loading exists in this specification: a single malformed or unresolved entity anywhere in
the package makes the **whole package** invalid, not just that entity. This is a deliberate
departure from `ModuleRegistry`'s "one broken module does not block the rest" behavior
(`CORE.md` §3), and the difference is intentional, not an oversight: modules are independent
units composed at a coarse grain, where one Tutor module failing to register has no bearing
on whether Practice works. A Knowledge Package's entities are not independent in that way —
they form one coherent cross-referenced graph (objectives reference concepts, examples
reference both, provenance references sources, prerequisites form a single package-wide
DAG). Silently loading "everything except the three broken entities" risks serving a package
that looks complete but is missing pieces other entities depend on, with no clear signal
anything is wrong — a worse failure mode than refusing to load at all. Given this
specification's packages are calibrated to be small (§14 of the brainstorm), there is no
analogous pressure to keep "the rest of the bundle" working the way Core's registry has.

Which specific error(s) a given invalid package surfaces, and in what order, is **not**
part of this contract (see §13) — implementations SHOULD collect and report every structural
error found in one validation pass rather than failing on the first, for authoring
ergonomics, but this is a quality-of-implementation SHOULD, not a MUST. The only normative
requirement is the final accept/reject verdict and, on acceptance, the resulting typed
`KnowledgePackage` value.

## 13. Structural validation order

The following is a **recommended implementation sequence** (SHOULD), not a normative
requirement — nothing about observable behavior depends on checking these in exactly this
order, only on all of them holding before a package is accepted:

```text
1. locate required package files/directories (package.toml, sources.toml)
2. parse package.toml; validate schema_version (§14) and package id (§2)
3. parse sources.toml; validate Source id uniqueness (§5) and grammar
4. discover concepts/, objectives/, examples/ entity files deterministically (§3)
5. parse every entity file's frontmatter and body (§6, §7, §8, §9)
6. validate filename ↔ entity-id agreement for every entity file (§3)
7. validate per-kind ID uniqueness across all discovered entities (§2)
8. resolve every foreign reference (§12)
9. validate related_ids symmetric-authorship invariant (§10)
10. validate the prerequisite_ids graph is acyclic (§10)
11. validate provenance minimums and duplicate rules (§11)
12. produce the typed, validated KnowledgePackage value
```

Exact error ordering/reporting granularity across implementations is explicitly not part of
the interoperability contract.

## 14. Schema version compatibility

- `schema_version = 1` is the only value accepted by a v1 loader.
- `schema_version = 0`, `schema_version = 2`, or any value other than `1`: a well-typed
  integer that is simply unsupported — MUST be rejected with a semantic "unsupported schema
  version" failure.
- `schema_version` missing: `package.toml`'s `schema_version` field is required (§4); its
  absence is a structural (missing required field) failure at the parse layer, not a
  separate semantic check.
- `schema_version` present but not an integer (e.g. a string or a float): a TOML type
  mismatch — a structural/syntax failure, distinct from the semantic
  "unsupported-but-well-typed" case above.
- `Package.version` (the content revision) has **no effect on schema compatibility**. A
  loader accepts or rejects a package based on `schema_version` alone.
- No migration behavior is designed by this document. A future schema revision's
  compatibility story is out of scope for v1.

## 15. Unknown fields/files policy

| Artifact | Unknown fields/files |
|---|---|
| `package.toml` top-level keys | Rejected |
| `sources.toml` top-level constructs (anything but `[[sources]]`) | Rejected |
| `sources.toml`, unknown keys within one `[[sources]]` entry | Rejected |
| `Concept`/`Objective`/`Example` frontmatter, unknown keys | Rejected |
| Package root, files/directories other than the five reserved names | Ignored — MAY exist, MUST NOT be rejected |
| `concepts/`/`objectives/`/`examples/`, non-`.md` files or subdirectories | Rejected |
| `concepts/`/`objectives/`/`examples/`, `.md` files not matching an entity's own `id` | Rejected (filename/ID mismatch, §3) |

The package-root exception exists specifically so non-schema authoring documentation
(`synthesis-report.md`, a `README.md`) can coexist with the canonical package without
weakening entity discovery — everywhere entity discovery actually happens (the three entity
directories), the policy is strict.

## 16. Determinism

A package's meaning MUST NOT depend on filesystem enumeration order, hash-map iteration
order, wall-clock time, environment variables, network availability, or any runtime LLM
call. A conforming loader MUST be able to validate and load a package completely offline.

Array ordering significance, decided per field rather than assumed uniformly:

| Field | Order significance |
|---|---|
| `Source.authors` | **Significant** — presentation/citation byline order; MUST be preserved as authored |
| `Example.hints` | **Significant** — pedagogically ordered (progressive disclosure); MUST be preserved as authored |
| `Concept.prerequisite_ids` | Not significant |
| `Concept.related_ids` | Not significant |
| `Example.objective_ids` | Not significant |
| every entity's `provenance_refs` | Not significant |
| `sources.toml`'s `[[sources]]` entries | Not significant |
| raw directory/filesystem enumeration | Not significant, and MUST NOT leak into any of the above |

Where order is "not significant," an implementation MAY sort for presentation, hashing, or
test-comparison purposes (a lexicographic-by-ID sort is RECOMMENDED where a canonical order
is needed), but MUST NOT treat the authored order in those fields as meaningful, and MUST
NOT reorder the two "significant" fields above under any circumstance.

## 17. Canonical package example

A complete, minimal, conforming package. Every formula below was checked by direct
calculation, not merely transcribed.

**`package.toml`**

```toml
id = "org.axiom.calculus_shells"
schema_version = 1
version = "1.0.0"
title = "Cylindrical Shells (Reference Example)"
description = "A minimal conforming Knowledge Package v1 example, drawn from OpenStax Calculus Volume 2 §2.3."
```

**`sources.toml`**

```toml
[[sources]]
id = "src.openstax_calc2"
title = "Calculus Volume 2"
authors = ["Gilbert Strang", "Edwin \"Jed\" Herman"]
edition = "2016"
license = "CC-BY-NC-SA-4.0"
```

**`concepts/shell.method_vertical_axis.md`**

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

**`concepts/shell.method_horizontal_axis.md`**

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

**`concepts/shell.method_selection.md`**

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

**`objectives/shell.setup_radius_height.md`**

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

**`examples/shell.example_y_poly.md`**

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

This package satisfies every requirement in §§2–16: 1 `KnowledgePackage`, 1 `Source`, 3
`Concept`s exercising both `prerequisite_ids` (`shell.method_selection` depends on both
axis-method concepts) and `related_ids` (the two axis-method concepts, authored once, on
`shell.method_vertical_axis` only), 1 `Objective`, 1 `Example` exercising `concept_id`,
`objective_ids` (same-concept), `Problem`/`Solution`/`Hints`, and provenance including both
`Direct` and `Derived` refs (on `shell.method_vertical_axis`).

## 18. Invalid examples / conformance cases

Representative rejection cases, each naming the specific rule it exercises. These become
the natural seed for a future conformance suite; they are not implemented here.

| Case | Snippet | Outcome | Rule |
|---|---|---|---|
| Invalid identifier | `id = "Shell-Method"` | Rejected | §2 — uppercase and hyphen both forbidden |
| Single-segment identifier | `id = "shell"` | Rejected | §2 — minimum two segments |
| Filename/ID mismatch | file `shell.a.md` declares `id = "shell.b"` | Rejected | §3 |
| Duplicate entity ID | two files under `concepts/` both declare `id = "shell.x"` | Rejected | §2 |
| Unresolved concept reference | `Objective.concept_id = "shell.nonexistent"` | Rejected | §12 |
| Unresolved objective reference | `Example.objective_ids = ["shell.nonexistent"]` | Rejected | §12 |
| Unresolved Source | `ProvenanceRef.source_id = "src.nonexistent"` | Rejected | §12 |
| Self prerequisite | `Concept("shell.a").prerequisite_ids = ["shell.a"]` | Rejected | §10 |
| Prerequisite cycle | `A→B`, `B→C`, `C→A` in `prerequisite_ids` | Rejected | §10 |
| Self related | `Concept("shell.a").related_ids = ["shell.a"]` | Rejected | §10 |
| Reverse-double-authored related edge | `A.related_ids = ["B"]` and `B.related_ids = ["A"]` | Rejected | §10 |
| Duplicate ID inside a list | `prerequisite_ids = ["shell.a", "shell.a"]` | Rejected | §10 |
| Missing provenance | `Concept` with `provenance_refs = []` (or omitted) | Rejected | §11 |
| Unknown ProvenanceKind | `kind = "inferred"` | Rejected | §11 |
| Cross-concept objective | `Example.concept_id = "A"`, one `objective_ids` entry belongs to concept `B` | Rejected | §9 |
| Missing Example Problem | body has `## Solution` with no preceding `## Problem` | Rejected | §9 |
| Missing Example Solution | body has only `## Problem` | Rejected | §9 |
| Duplicate Example heading | two `## Solution` sections | Rejected | §9 |
| Unknown Example `##` heading | `## Notes` present | Rejected | §9 |
| Non-list Hints content | `## Hints` followed by a plain paragraph, no `- ` items | Rejected | §9 |
| Malformed TOML frontmatter | unterminated string inside `+++ ... +++` | Rejected | §6 |
| Unsupported schema_version | `schema_version = 2` | Rejected | §14 |

## 19. Existing Calc II reconciliation contract

The full field-by-field reconciliation lives in the brainstorm document's §9; this section
states only the categorical, normative migration outcome. Old filenames and old field names
are not normative — nothing here requires a future migration to preserve them.

```text
package.json metadata                → KnowledgePackage (package.toml)
3 existing concepts/*.json           → Concept (renamed IDs, restructured provenance)
6 existing objectives/*.json         → Objective (renamed IDs, evidence field dropped)
11 provenance.json entries           → 1 Source + 10 SourceLocator-based ProvenanceRefs
6 of 7 problem-families/*.json       → newly-authored, fixed-instance Example entities
1 of 7 (pf-method-select-integral-count) → excluded; see below
all generation/verification fields   → excluded from Knowledge v1 entirely
    (generator, parameters, constraints, promptTemplate, canonicalSolution.expression,
     validator, templated hints, difficulty range)
                                        → design evidence for roadmap items 2 (Canonical
                                          Problem), 3 (math.verify), 5 (deterministic
                                          generation), 6 (Practice) — not designed here
```

The headline finding this specification preserves without dilution: **11 flat "source"
entries become 1 `Source` plus 10 `SourceLocator`-based citations.** The old package did not
model this incompletely — it modeled source *locations* as if they were independent
*works*, redundantly repeating the same authors/title/license ten times over.

`pf-method-select-integral-count.json` does not migrate into Knowledge v1. It is preserved
as design/source-review evidence until the unresolved source question documented in the
brainstorm is settled. If it later proves useful and source-supported, a fixed static
`Example` may be authored from it; otherwise it may simply disappear from canonical
Knowledge. This specification is not obligated to preserve every prototype artifact.

Migrating `knowledge-package/` against this contract is explicitly not part of this pass
(§21 restates this).

## 20. Explicit validation boundary

This specification's structural validation covers: schema shape, identifier grammar,
reference resolution, relationship consistency (including the prerequisite DAG invariant),
provenance presence and duplicate rules, and entity body grammar.

It does **not** prove, and MUST NOT be represented as proving:

- Mathematical truth or formula correctness.
- Pedagogical quality.
- Source fidelity (whether a `Direct` ref genuinely matches its cited location, or a
  `Derived` claim is a reasonable synthesis).
- License compliance or redistribution correctness.
- Mastery validity or any claim about a learner's understanding.

This distinction matters concretely: the existing Calc II package's mathematics was checked
by independent, manual derivation across two review passes — that is a separate review
discipline this specification's structural validation does not replace, automate, or imply.
A package that passes every rule in this document may still be mathematically wrong; passing
this specification is a necessary, not sufficient, condition for a trustworthy package.

## 21. Sequencing

```text
brainstorm ✓
        ↓
formal specification ✓ (this document)
        ↓
implementation plan
        ↓
implementation + Calc II package reconciliation
```

No implementation plan is written here. The loader is not implemented here.
`knowledge-package/` is not migrated here. All three are later, separately-gated steps.
