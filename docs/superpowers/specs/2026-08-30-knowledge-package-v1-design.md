# Knowledge Package v1 — architectural brainstorm

Stage 8 sub-project 2 (Knowledge Package v1 schema), the first item on `ROADMAP.md`'s
"Remaining Stage 8 scope" list — sitting immediately after sub-project 1 (`.ai/tasks/
045–048`, the module & capability runtime, done and merged). Everything from roadmap item 2
onward (Canonical Problem schema, the `math.verify` capability, a tiny reference Calc II
package, deterministic generation, the Practice Core Utility, Practice's testing bar, Study
Session UI integration, the offline acceptance test) is explicitly out of scope here and
gets its own brainstorm-and-spec pass once this contract is locked, mirroring exactly how
`docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md` §1 scoped itself
against this document.

This is the finalized output of an architectural brainstorm (`superpowers:brainstorming`),
not yet a frozen specification. Field shapes, on-disk grammar, and validation invariants
captured here are the recommended architecture; §14 names the next pass that freezes them
normatively.

---

## 1. Context

Sub-project 1 is done: `module.toml` parsing/validation, the `ModuleRegistry`/capability
resolution runtime, and a conformance suite proving "bundled doesn't mean privileged" — all
Rust, in-process, first-party built against exactly the contract a third party would use.
`CORE.md` is that contract's permanent home now, no longer aspirational for its own §§1–5.

Both `CORE.md` and the module-capability design document are explicit that everything past
sub-project 1 — Knowledge Package, Canonical Problem, Practice, `math.verify`, reference
content, UI integration — is a *consumer* of that contract, not part of it, and designing
them without their own pass "would mean designing against a guess."

The complication this brainstorm exists to resolve: `knowledge-package/` (a Calc II
"cylindrical shells" reference package) already exists in the repository, reviewed and
merged (tasks 049 review, 049 fix). Its mathematics is sound — every formula in it was
independently re-derived and verified across two review passes — but its *shape* (file
layout, ID grammar, entity boundaries) was invented ad hoc, before this schema existed to
constrain it. It is evidence of what a real package needs, not a precedent to preserve.

## 2. Problem Statement

**Knowledge Package v1 must:**

- Give a package and every entity inside it stable, typed identity and versioning.
- Model the minimum educational domain — concepts, learning objectives, static examples,
  relationships, source provenance — sufficient to represent the existing Calc II content
  faithfully.
- Be a deterministic, offline, text-based, git-diffable on-disk format a Rust loader can
  parse mechanically, mirroring how `module.toml` → `ModuleManifest` already works.
- Support structural validation (duplicate IDs, unresolved references, cycles) independent
  of any mathematical truth check.
- Draw a boundary sharp enough that Canonical Problem, Practice, `math.verify`, and Tutor
  can each be designed later without reopening this schema.

**Knowledge Package v1 explicitly does not solve:**

- Problem generation, parameterization, or seeding (Canonical Problem + deterministic
  generation, roadmap items 2 & 5).
- Mathematical/symbolic verification (`math.verify`, item 3).
- Practice orchestration: difficulty selection, session sequencing, spaced repetition,
  adaptive selection, attempt state, scoring policy, hint timing/delivery, mastery state
  (item 6).
- A Docling-based ingestion pipeline (deferred — the schema must be populatable by one
  later without redesign, not dependent on it now).
- Marketplace distribution, signing, or multi-package composition semantics beyond
  identifying one package.

## 3. Repository Evidence

- `CORE.md` §3's own `module.toml` example includes `[[requires]] id = "knowledge.query"`
  as an illustrative capability requirement — Knowledge is anticipated to eventually be
  reachable *as a capability* some provider serves, not only as files Practice reads
  directly off disk. This doesn't require v1 to design that capability now, but it means
  package/entity identity should be shaped so a future capability input can reference it
  cleanly.
- The module-capability design document §1: "Everything from Stage 8 §8.2 onward
  (Knowledge Package, Problem schema, Practice Utility, verification capability...) is
  explicitly out of scope here and gets its own brainstorm-and-spec pass once this contract
  is locked and stable." Confirms this document's placement and that no Knowledge decisions
  were smuggled into sub-project 1.
- `ARCHITECTURE.md:77`: `knowledge-package/ # Stage 8 reference content; ad hoc until
  Knowledge Package v1` — the repository already documents its own current package as
  provisional.
- `src/types/concept.ts` (Stage 0–6, mock-data-only, UI-facing) already has a `Concept`
  interface blending Knowledge-shaped fields (`name`, `chapter`, `displayFormula`,
  `explanation`, `prerequisiteConceptIds`, `relatedConceptIds`) with Practice/Mastery-shaped
  fields (`masteryState`, `wasMasteryState`, `dueForReviewInDays`, `onExam`,
  `recentDiagnostics`, `learnerHeuristic`, `notesCount`, `lastActivityAt`). It also carries
  `leadsToConceptIds` and `blocksConceptIds`, which read as the *inverse* of
  `prerequisiteConceptIds` rather than independently authored facts — direct evidence
  against authoring both a relationship and its reverse.
- `src/types/material.ts`'s `MaterialResult.kind: 'section' | 'workedExample' |
  'exerciseRange'` — `exerciseRange` is explicitly a count ("14 exercises · 3 attempted"),
  never full problem content. `Material` (book-level: `title`, `edition`, `totalPages`,
  `totalChapters`) is exactly the shape source-identity provenance needs.
- No `Objective` type exists anywhere in the frontend — genuinely open ground, no legacy
  shape to reconcile against.
- `src/types/module.ts`'s `Module.supportedConceptNames?: string[]` is plain display
  strings, not a real reference to any Concept entity — a UI convenience, not evidence of a
  required cross-package link.
- `src-tauri/src/modules/identifier.rs`'s `validate_identifier` (dot-segmented, lowercase
  ASCII + digit + underscore, ≥2 segments) is the one identifier grammar this codebase has
  actually implemented, tested, and conformance-checked (task 048). The existing
  `knowledge-package/`'s IDs violate it in three mutually inconsistent ways: concept IDs are
  pure hyphen-slugs, objective IDs mix a dotted prefix with hyphenated suffixes,
  `package.json`'s `id` is dot-segmented but contains a hyphen.
- `knowledge-package/` as it stands (post-049 fix) is already laid out as a normalized
  entity graph on disk — `concepts/`, `objectives/`, `problem-families/` as separate
  one-file-per-ID directories, cross-referenced by ID string, `package.json` enumerating all
  three ID lists — even though `synthesis-report.md` narrates it in concept-centric prose.
  The `problem-families/*.json` files carry `generator`, `parameters` (including a
  hand-rolled `{parameter, offset}` cross-parameter-dependency mechanism invented during the
  049 fix), `validator: {capability: "math.verify", version, mode}`, and templated `hints` —
  real, working evidence of what a *future* Canonical Problem / Practice / `math.verify`
  boundary needs to absorb, not evidence that Knowledge should keep it.
- `provenance.json` currently holds 11 flat entries, only one of which (the book itself) is
  actually a distinct work — the other 10 are locations *within* that one book, each
  redundantly repeating the same authors/title/license. This is the clearest concrete
  evidence in the whole package that its shape needs correcting, not formalizing (§9).

## 4. Design Constraints

- **Offline, no network, no runtime LLM calls** — every field must be locally resolvable
  from the package's own files.
- **Deterministic build/load** — identical package files always parse to the same in-memory
  representation; no wall-clock timestamps, no unordered-collection-dependent output.
- **Rust-consumable** — the on-disk shape must be mechanically parseable by a loader in the
  same style as `src-tauri/src/modules/manifest.rs` (raw form → typed, validated structs;
  raw form never propagates past that boundary).
- **Text-based and diffable** — one Knowledge Package entity per file for `Concept`/
  `Objective`/`Example`, matching both this repository's stated aversion to two agents
  fighting over one large shared file (`CLAUDE.md`) and the existing package's own working
  precedent.
- **No second ID grammar invented where reusing the proven one is possible** — but a
  Knowledge entity ID is a different *domain* than a Module/Capability ID, so the grammar is
  shared without the types being conflated.
- **No executable content** — nothing in a Knowledge Package runs code, matching `CORE.md`
  §2's "plain, structurally-clonable data" discipline at the module boundary.
- **Small enough that the existing tiny Calc II package can exercise nearly all of it** — a
  proposed entity or field earns its place only if something already true of that package or
  a named near-term consumer requires it.
- **Reviewable in isolation** — a reviewer can approve this schema's contract without a full
  implementation existing, the same posture sub-project 1 took.

## 5. Domain Boundary

The organizing principle is responsibility, not "fixed vs. computed" — static data can
still fail the test if it's machinery for producing an exercise rather than a fact about the
subject:

```text
Knowledge          = declarative subject knowledge
Canonical Problem   = declarative assessment/generation specification
Practice             = learner-facing orchestration and policy
math.verify           = mathematical correctness/equivalence computation
Tutor                 = pedagogical interaction behavior
```

The test: **does this describe the subject itself, or does it describe machinery for
producing, presenting, or assessing an exercise about the subject?**

```text
                 KNOWLEDGE
          "What is there to learn?"
                    │
        concepts / objectives /
      explanations / static examples /
      prerequisites / provenance
                    │
                    ▼
             CANONICAL PROBLEM
       "What constitutes a problem?"
                    │
      parameters / constraints /
       prompts / answer specs /
    supported difficulty range /
       generated hint specs
                    │
          ┌─────────┴─────────┐
          ▼                   ▼
       PRACTICE           math.verify
 "What do we give        "Is this answer
 this learner now?"        equivalent?"
```

Tutor consumes Knowledge like any other module would, through whatever future
`knowledge.query`-shaped capability reads a package; it is not part of this schema.

**Difficulty.** A supported difficulty range, or anything about how difficulty affects
generation, is a property of the problem family — Canonical Problem. Practice owns only the
*decision*: requested/selected difficulty for this learner, right now, and whatever adaptive
policy picks it.

**Hints.** A hint that depends on generated values (`"Use r = {radius}..."`) is instantiated
from the problem the same way the prompt is — Canonical Problem owns hint *definitions*.
Practice owns hint *selection, reveal timing, and state*. A fixed worked `Example` in
Knowledge may still carry its own optional, non-templated explanatory hints, authored
directly as part of that one example — nothing to instantiate.

**Editorial review state.** Whether a given transcription was checked is editorial/
authoring metadata, not a fact about the subject. It does not appear in the runtime schema
at all (§7).

**Canonical solutions.** A parameterized derivation does not survive into Knowledge
directly:

```text
parameterized canonicalSolution.structure
        ↓
does not directly survive Knowledge v1
        ↓
optionally materialize one concrete instance
        ↓
static Example.problem + Example.solution
```

A general family's derivation (parameter `coeff`, `b`) doesn't become a Knowledge entity —
but Knowledge may contain one fully-instantiated worked example (`coeff = 4, b = 3`, fixed
prompt text, fixed solution steps), exactly the way the source textbook's own worked
examples are each one fixed instance, not a family. The generic parameterized version
remains design evidence for Canonical Problem only.

**Objectives** describe "what a learner is expected to understand or be able to do" — not
"what proves you understand it." Objectives are declarative subject knowledge; whether a
given performance constitutes proof of mastery is a later system's call.

**Conclusion**: no `problem-families/` entity survives into Knowledge Package v1. It's
preserved as design evidence for roadmap items 2 (Canonical Problem), 3 (`math.verify`), 5
(deterministic generation), and 6 (Practice).

## 6. Candidate Domain Model

**Five primary entities** — `Package`, `Concept`, `Objective`, `Example`, `Source` — plus
supporting value/reference types: `KnowledgePackageId`, `ConceptId`, `ObjectiveId`,
`ExampleId`, `SourceId`, `ProvenanceRef`, `ProvenanceKind`, `SourceLocator`.

```rust
struct Package {
    id: KnowledgePackageId,
    schema_version: u32,        // this document defines v1 = 1
    version: semver::Version,   // content revision, independent of schema_version
    title: String,
    description: String,
    // No hand-maintained concept/objective/example ID lists — discovered from disk.
}

struct Concept {
    id: ConceptId,
    name: String,
    topic: Option<String>,             // display label, e.g. "2.3 Volumes of Revolution:
                                        // Cylindrical Shells" — mirrors src/types/concept.ts's
                                        // `chapter`; not a separate Topic entity
    description: String,               // entire Markdown body (§7)
    prerequisite_ids: Vec<ConceptId>,  // directed; authored on the gated concept only
    related_ids: Vec<ConceptId>,       // symmetric meaning; authoring rule below
    provenance_refs: Vec<ProvenanceRef>,  // required, ≥1 (§8)
}

struct Objective {
    id: ObjectiveId,
    concept_id: ConceptId,             // authored FK
    description: String,               // "what a learner is expected to understand/do";
                                        // entire Markdown body
    provenance_refs: Vec<ProvenanceRef>,  // required, ≥1
}

struct Example {
    id: ExampleId,
    concept_id: ConceptId,
    objective_ids: Vec<ObjectiveId>,   // zero or more — an example may illustrate a
                                        // concept without being tied to a formal objective
    problem: String,                   // ## Problem — fixed, fully-written, no templating
    solution: String,                  // ## Solution — fixed worked derivation
    hints: Vec<String>,                // ## Hints — optional, plain, non-templated
    provenance_refs: Vec<ProvenanceRef>,  // required, ≥1
}

struct Source {
    id: SourceId,
    title: String,
    authors: Vec<String>,              // structurally may be empty
    edition: Option<String>,
    license: Option<String>,           // stable identifier, e.g. "CC-BY-NC-SA-4.0"
}

enum ProvenanceKind { Direct, Derived }

struct ProvenanceRef {
    source_id: SourceId,
    locator: Option<SourceLocator>,
    kind: ProvenanceKind,
}

struct SourceLocator {
    section: Option<String>,   // e.g. "2.3" — free-form
    pages: Option<String>,     // e.g. "137-152" — free-form
    label: Option<String>,     // the element exactly as the source names it:
                                // "Example 2.13", "Rule 2.6", "Definition 4.1"
}
```

```text
KnowledgePackage
│
├── Concept
│    ├── prerequisite → Concept
│    └── related ↔ Concept
│
├── Objective
│    └── concept → Concept
│
├── Example
│    ├── concept → Concept
│    └── objectives → Objective*   (zero or more)
│
└── Source / ProvenanceRef
     └── referenced by authored knowledge
```

No learner state, problem-generation machinery, verifier configuration, runtime behavior,
or generic graph abstraction appears anywhere in this model.

**One rule does most of the structural work: every relationship is authored from exactly
one side; the other direction is a derived view, never separately authored.** Authored
directions, exhaustively:

```text
Objective → Concept
Example   → Concept
Example   → Objective*
Concept   → prerequisite Concept
Concept   → related Concept
```

`Concept.objective_ids`, `Concept.example_ids`, `Objective.example_ids`, `leads_to_ids`, and
`blocks_ids` are all deliberately absent — a loader derives every reverse index by scanning
`concept_id`/`objective_ids` back-references. This directly forecloses two real drift risks:
the `Concept.objectiveIds`/`Objective.conceptId` dual-authoring already present (harmlessly,
today) in the existing package, and the `leadsToConceptIds`/`blocksConceptIds` redundancy
already visible in `src/types/concept.ts`.

**`related_ids` authoring rule.** `related` is semantically symmetric but authored once.
Three options were compared: requiring both directions authored (doubles every edit, exactly
the dual-authoring shape the rest of the model avoids); one direction with a canonical
ordering rule (minimal authoring, but forces an author to compute which file owns a pair
before writing to it); either direction permitted, normalized to a symmetric view in memory,
duplicate reverse declaration rejected. The third wins on authoring burden, determinism, and
validation cost together. Rules:

```text
A.related_ids contains B  →  B.related_ids must NOT also contain A  (validation error)
self-reference in related_ids or prerequisite_ids  →  invalid
duplicate IDs within one related_ids/prerequisite_ids array  →  invalid
loader exposes the relation symmetrically in memory regardless of authored side
a future package-writing tool preserves only the originally-authored side
```

`prerequisite_ids` stays directed and is never mirrored. The prerequisite graph **must form
a DAG** — a cycle describes no coherent learning order. This rule applies only to
`prerequisite_ids`; `related_ids` has no acyclic constraint (it's symmetric by construction,
so a "cycle" isn't even a meaningful concept for it). Two concepts genuinely best learned
together, with no valid ordering between them, belong in `related_ids`, not a mutual
`prerequisite_ids` pair.

**ID grammar — lexical reuse only, no semantic loading.** `KnowledgePackageId`, `ConceptId`,
`ObjectiveId`, `ExampleId`, `SourceId` each reuse `identifier.rs`'s dot-segmented, lowercase
+ digit + underscore grammar as distinct wrapper types — not `ModuleId` itself, and not one
shared untyped string. A dot segment carries no implied hierarchy, taxonomy, parentage, or
package ownership: `shell.method_vertical_axis` is a stable opaque identifier, not a claim
that `shell` is a parent concept. Concept hierarchy, if ever needed, would be an explicit
`parent_id`-style field, never inferred from ID structure.

**Reference scope — package-local only.** Every `ConceptId`/`ObjectiveId`/`ExampleId`/
`SourceId` reference inside a Knowledge Package resolves within that same package. No
cross-package entity references in v1 — the roadmap's explicit deferral of multi-package
composition semantics supports this, and nothing in repository evidence contradicts it.

**Package ID vs. entity ID.** Entity IDs are never prefixed by package ID:

```text
package:  org.axiom.calculus_shells
concept:  shell.method_vertical_axis
```

not `org.axiom.calculus_shells.shell.method_vertical_axis`. Package identity, entity
identity, and package ownership are three separate facts; the third is never inferred from
the second.

**ID uniqueness — per entity kind, not package-global.** `ConceptId("shell.basic")` and
`ObjectiveId("shell.basic")` may coexist — the Rust wrapper types already prevent them from
being confused programmatically (every reference site in the domain model is kind-typed),
and directory-per-kind (§7) disambiguates them visually. Package-global lexical uniqueness
across kinds is a constraint with no demonstrated consumer and is not imposed.

## 7. Package Layout

**File granularity is a recommendation, not an inherited constraint.** `module.toml` shows
this repository is comfortable with small, independently-parseable files, but that alone
isn't evidence for ID-named entity *collections*. The actual evidence is: the existing
`knowledge-package/` already does this and worked cleanly through two review passes; git
diffs stay entity-scoped; a validation error localizes to one file; multi-agent edit
contention drops; directory listing gives deterministic, sorted discovery for free. It wins
those criteria for `Concept`/`Objective`/`Example`.

**`Source` gets a different answer.** The one-file-per-entity reasoning is weakest exactly
where cardinality and edit frequency are lowest: a package typically cites a handful of
works, added rarely once authoring is underway, unlike concepts/objectives/examples which
are added constantly. A single `sources.toml` with TOML's native `[[source]]`
array-of-tables keeps this simple without meaningfully reintroducing contention risk.

Recommended layout:

```text
knowledge-package/
├── package.toml
├── sources.toml
├── concepts/
├── objectives/
└── examples/
```

**Serialization format.** Compared against real content, not in the abstract — the concrete
offender is `Example.solution`, a multi-step worked derivation with several LaTeX
expressions, where `\int_0^3 2\pi x f(x)\,dx` becomes `\\int_0^3 2\\pi x f(x)\\,dx` as a JSON
string: mechanically valid, materially worse to read, review, and diff. Four layouts were
compared:

| | Pure JSON | TOML frontmatter + Markdown body, one file | JSON + companion `.md` | Entity directory (3+ files) |
|---|---|---|---|---|
| Prose/LaTeX ergonomics | Poor — escaped throughout | Native | Native | Native |
| Diff quality | Noisy | Clean | Clean | Clean |
| New parsing code | None | Small, bounded — one frontmatter delimiter, a fixed, closed set of headers | None for the JSON half; a filename-matching convention | Same, times three files |
| File-integrity risk | None | None — one file | Real — two files can drift apart | Worse — three files |
| Files per entity | 1 | 1 | 2 | 3+ |

JSON + companion Markdown is dominated by TOML+Markdown-in-one-file — it pays a real
pairing-integrity cost for no ergonomic gain the single-file form doesn't already have. The
entity-directory option establishes the lower bound of how far file-splitting could go and
is heavier than a 3-concept package justifies.

**Recommended: TOML frontmatter + Markdown body, one file per entity**, `.md` extension for
`Concept`/`Objective`/`Example`, `.toml` for `package.toml`/`sources.toml`. `+++` delimits
frontmatter (distinct from `---`, so nothing assumes YAML). The parser splits on the first
two literal `+++` lines; content between is TOML; content after the second, with leading
blank lines trimmed, is the body. UTF-8 is required; a BOM is rejected outright rather than
left to produce a confusing downstream parse failure. The parser accepts either LF or CRLF
line endings; LF is this repository's authoring/commit convention, enforced by tooling, not
by the parser rejecting CRLF on sight.

**Body grammar per entity, deliberately narrow — no generic Markdown AST anywhere in
Knowledge v1:**

- `Concept` / `Objective`: the entire remaining body, with no required heading, is
  `description`. Neither entity has more than one long-form field, so no section convention
  is needed.
- `Example` recognizes exactly `## Problem`, `## Solution`, `## Hints`, in that fixed order.
  `Problem` and `Solution` are **required**; `Hints` is **optional**, defaulting to empty.
  Duplicate headings are invalid. Any `##` heading outside this closed set is invalid, not
  silently ignored. Each top-level Markdown list item under `## Hints` becomes one `String`
  in `hints`; non-list-item content under that heading is invalid.

```text
+++
id = "shell.example_basic"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.objective_setup"]
+++

## Problem

Find the volume of the solid formed by revolving the region under
`f(x) = 2x - x^2` on `[0, 2]` around the *y*-axis.

## Solution

The shell radius is `r(x) = x` and the height is `h(x) = 2x - x^2`.

\[
V = \int_0^2 2\pi x (2x - x^2)\,dx = \frac{8\pi}{3}
\]

## Hints

- Identify the radius and height as functions of `x` before integrating.
```

**Serialization and domain model stay separate.** The loader's job is: on-disk
representation → parse → typed `Concept`/`Objective`/`Example`/`Source`. Nothing
downstream — Tutor, a future Study UI, a future `knowledge.query` capability — ever knows or
needs to know whether a given `String` field originated as a JSON value or a Markdown
section.

**Editorial workflow stays outside the schema entirely.** Three options were compared:
separate authored-source and compiled-runtime forms (a real guarantee, but a compile
pipeline nothing else in this repository has, for a need no consumer has demonstrated);
a unified form carrying an optional `review_status` field (weakens the §5 boundary that
editorial state isn't subject knowledge, just to avoid a split); a unified canonical package
with **no** review/draft field in the schema at all, editorial state carried entirely by
this repository's own `.ai/tasks/` + review process. The third was chosen: `verified`,
`needs-review`, `draft`, and any equivalent do not appear as schema fields. A Knowledge
Package contains only canonical content; unresolved content is not "in the package with a
flag" — it simply isn't in the package yet, tracked the same way every other piece of work
in this repository already is. No source/runtime compile stage in v1.

## 8. Provenance Model

Kept to exactly five questions.

**1. What source/work did this knowledge come from?** `Source { id, title, authors,
edition, license }`. `authors` is `Vec<String>`, structurally allowed to be empty — the
current OpenStax source requiring attribution doesn't prove every future source has a
conventional author list (institutional standards, uncertain authorship, collaboratively
authored material), and a separate `attribution: Option<String>` field wasn't justified
alongside it. `license` is intended as a stable identifier (SPDX-style where one exists,
e.g. `"CC-BY-NC-SA-4.0"`, not required to be SPDX where no clean mapping exists) — prose
describing terms belongs elsewhere, not in this field.

**2. Where within that source did it come from?**

```rust
struct SourceLocator {
    section: Option<String>,   // e.g. "2.3"
    pages: Option<String>,     // e.g. "137-152" — free-form; structured {from, to} deferred
                                // until a consumer needs range arithmetic or sorting
    label: Option<String>,     // the element exactly as the source names it
}
```

`label` replaces an earlier, looser `note` field precisely because a "note" invites
arbitrary commentary ("near bottom of page," an ingestion-tool block ID); `label` means only
"the source's own name for this element" — `"Example 2.13"`, `"Rule 2.6"`, `"Definition
4.1"`, `"Figure 3"`. **`SourceLocator` fields are human/source-facing identifiers; they must
never contain ingestion-tool-local IDs.**

**3. Can multiple source locations support one entity?** Yes — already true and already
exercised: `provenance_refs: Vec<ProvenanceRef>`. The current package already cites, e.g.,
`shell-method-vertical-axis` against Rule 2.6 and Examples 2.12/2.13/2.15/2.16
simultaneously, all resolving correctly. This formalizes an already-working pattern (a flat
array of bare source-id strings) into `{source_id, locator, kind}` triples.

**4. Can synthesized/derived Knowledge still point back to evidence?** Yes, via:

```rust
enum ProvenanceKind { Direct, Derived }
```

```text
Direct:  the entity substantially represents content explicitly present at the
         cited source location.
Derived: the entity contains synthesis, reorganization, generalization, inference,
         or newly-authored pedagogical structure based on the cited source location.
```

`kind` lives on `ProvenanceRef`, not the entity — one `Concept` may carry both a `Direct` ref
(a formula stated verbatim in Rule 2.6) and a `Derived` ref (the concept boundary itself,
synthesized across a whole section, exactly as `synthesis-report.md` already documents for
separating `shell-method-vertical-axis` from `shell-method-horizontal-axis`). This is
justified by a use case already exercised for real: the original source-fidelity review of
this package had to reconstruct this exact distinction by hand from prose; `kind` makes it a
queryable fact instead of an essay. No confidence scores, no finer-grained kinds — nothing
beyond this two-variant distinction is demonstrated.

**5. Can future Docling ingestion populate the model without Docling IDs becoming part of
the permanent schema?** Yes, by construction — `SourceLocator` is human/textbook-shaped
(section label, page range, named element), never a technical block/span/bounding-box ID,
exactly like the current `provenance.json` already writes it. Docling would produce a
normalized source/evidence layer, in its own ID scheme; a future extraction/transformation
step maps that onto these human-shaped fields when populating a Knowledge Package. Docling's
own identifiers are working data for the ingestion tool and never need to survive that
mapping. This document does not design that transformation step.

**Provenance granularity is entity-level only.** Field-level (`problem_provenance`,
`solution_provenance`, `hint_provenance`) and content-block-level provenance were both
considered and rejected — checked specifically against the current package for a concrete
failure case where entity-level provenance would *mislead*, not merely be less precise, and
found none. Even a mixed case (core math directly derived from Rule 2.6, diagnostic hints
entirely Axiom-authored) is still honestly served by a `Derived` ref at the whole-entity
level. Field-level provenance is an explicit v1 non-goal, revisited only if a real case
demonstrates entity-level provenance actively misleading a consumer.

**Provenance is required.** Every `Concept`, `Objective`, and `Example` must contain at
least one `ProvenanceRef`. Two pieces of evidence support this: every entity in the current
reference package already has one — no orphans were found across two full review passes —
and the package's own stated premise is source-grounded content
(`package.json.description`: "derived authoritatively from OpenStax"). This does not block
future Axiom-original content: a package may declare a `Source` representing Axiom's own
authorship (e.g. `id = "org.axiom_original"`, no external license required) and cite it the
same way everything else is cited — honest provenance ("this is deliberately original"),
not a workaround, and a stronger guarantee than allowing silent, unattributed claims by
default.

**Nothing else.** No field-level provenance, confidence scores, Docling IDs, extraction
model names, OCR metadata, timestamps, bounding boxes, or transformation lineage. None of it
is demonstrated by any named v1 consumer.

## 9. Existing Package Reconciliation

The single biggest structural finding: **`provenance.json`'s 11 entries are not 11
`Source`s.** Only `src-openstax-calc2-book` is a real `Source`. The other 10 (`sec2-2`,
`sec2-3`, `rule2-6`, `rule-xaxis`, `ex2-12`…`ex2-17`) are all locations *within* that one
book, each currently modeled as its own independent source, redundantly repeating the same
authors/title/license ten times over:

```text
11 flat "source" entries
        ↓
1 Source
        +
10 SourceLocator-based citations
```

This is the strongest concrete example in this reconciliation of the brainstorm correcting
an ad hoc prototype rather than formalizing it: the old package was not simply incomplete
here — it modeled source *locations* as if they were independent *works*.

The second biggest: **no `problem-families/*.json` content survives as-is.** Six of seven
become small, freshly-authored static `Example`s built from the same underlying,
independently-verified math, reduced to one fixed instance rather than a parameterized
family; the seventh cannot migrate at all under this model.

| Current path/field | Current purpose | v1 decision | New representation/home | Migration required | Reason |
|---|---|---|---|---|---|
| **`package.json`** |||||
| `schemaVersion` | Schema version | Keep | `Package.schema_version` | No | Direct match |
| `id` | Package identity | Keep, rename | `Package.id` (`KnowledgePackageId`) | Yes — `cylindrical-shells` → `cylindrical_shells` | Hyphen violates identifier grammar |
| `version` | Content revision | Keep | `Package.version` | No | Direct match |
| `title`, `description` | Display metadata | Keep | `Package.title` / `.description` | No | Direct match |
| `conceptIds`, `objectiveIds`, `problemFamilyIds` | Manifest inventory (added task 049) | Remove | Directory scan at load time | Yes — delete fields | No reverse-authored inventories (§6) |
| **`provenance.json`** |||||
| `src-openstax-calc2-book` entry | The book itself | Keep, restructure | The package's one `Source` | Yes — rename id, drop `book`/`sourceType` | Only genuine `Source` in the file |
| `src-openstax-calc2-sec2-2`, `-sec2-3` entries | Section-level citations | Restructure | `ProvenanceRef{locator: {section}, kind: Derived}` against the one `Source` | Yes — no longer independent entries | These are locations, not sources |
| `src-openstax-calc2-rule2-6`, `-rule-xaxis` entries | Named-rule citations | Restructure | `ProvenanceRef{locator: {section, label}, kind: Direct}` | Yes | Same |
| `src-openstax-calc2-ex2-12`…`ex2-17` entries (6) | Named-example citations | Restructure | `ProvenanceRef{locator: {section, label: "Example 2.1N"}, kind: Direct}` | Yes | Same |
| Every entry's `authors`, `book`, `license` | Repeated 11× | Remove (10×), keep (1×) | Lives once, on the one `Source` | Yes | Pure redundancy under the old flat model |
| Every entry's `notes` | Descriptive prose + worked arithmetic | Mostly remove; partly informal | Dropped where an `Example.solution` now carries the same content; otherwise informal, in `synthesis-report.md`, never the runtime schema | Yes | No field in the new model holds prose summaries |
| **`concepts/*.json`** (3 files) |||||
| `id` | Identity | Keep, rename | `Concept.id` | Yes — e.g. `shell-method-vertical-axis` → `shell.method_vertical_axis` | Grammar violation |
| `name`, `description` | Display/content | Keep | `Concept.name` / `.description` (Markdown body) | Format only | Direct match |
| `prerequisiteIds` | Directed edges | Keep | `Concept.prerequisite_ids` | Yes — referenced IDs renamed | Already the right authored direction |
| `relatedConceptIds` | Symmetric edges | Keep, rename; clean up one entry | `Concept.related_ids` | Yes — `method-selection-volume-of-revolution.json` currently duplicates its own `prerequisiteIds` here; drop the duplicate | Resolves a filed follow-up from the 049 review directly |
| `objectiveIds` | Reverse index | Remove | Derived from `Objective.concept_id` | Yes — delete field, 3 files | No reverse-authored fields |
| `provenanceRefs` | Bare source-id array | Restructure | `Vec<ProvenanceRef>` | Yes, all 3 | Book-vs-locator split |
| **`objectives/*.json`** (6 files) |||||
| `id` | Identity | Keep, rename | `Objective.id` | Yes — e.g. `shell.setup-radius-height-y-axis` → `shell.setup_radius_height_y_axis` | Hyphenated segment violates grammar |
| `conceptId` | Authored FK | Keep | `Objective.concept_id` | Yes — referenced IDs renamed | Already the correct direction |
| `description` | Content | Keep | `Objective.description` | Format only | Direct match |
| `evidence` | Observable-ability bullets | Remove | None in schema; may move informally to `synthesis-report.md` | Yes — delete field, 6 files | No named v1 consumer |
| `provenanceRefs` | Bare source-id array | Restructure | `Vec<ProvenanceRef>` | Yes, all 6 | Same as concepts |
| **`problem-families/*.json`** (7 files) |||||
| `id` | Family identity | Keep, semantically rename | New `Example.id`, e.g. `pf-shell-y-poly` → `shell.example_y_poly` | Yes | `pf-` names a family; there is no family anymore |
| `conceptId`, `objectiveIds` | Linkage | Keep | `Example.concept_id` / `.objective_ids` | Yes — IDs renamed | Same shape, `objective_ids` now explicitly zero-or-more |
| `difficulty: {min, max}` | Generation difficulty range | Move out | Roadmap items 2 (supported range) & 6 (requested/selected) | Preserved as evidence only | Machinery for producing an exercise, not subject knowledge |
| `generator`, `parameters`, `constraints` | Deterministic generation, incl. task 049's `{parameter, offset}` mechanism | Move out | Roadmap items 2 & 5 | Preserved as high-value evidence — a working, tested prior-art example | Same |
| `promptTemplate` | Templated prompt | Move out; replaced | Item 2 owns templating; Knowledge gets a freshly authored, fully-instantiated `Example.problem` | Yes — new authoring, not a field rename | A static example has one fixed prompt |
| `responseType` | Answer-format declaration | Move out | Roadmap item 2 | N/A | Runtime/verification concern |
| `canonicalSolution.structure` | Worked derivation, parameterized | Does not survive directly | One concrete instantiation freshly authored as `Example.solution`; generic form preserved as evidence | Yes — new authoring | Per §5's migration story |
| `canonicalSolution.expression` | Machine-checkable answer key | Move out | Roadmap items 2/3 | N/A | `math.verify`'s concern |
| `validator` | Verification capability declaration | Move out | Roadmap item 3 | N/A | Never a Knowledge concern |
| `hints[].template` | Templated hints | Move out; replaced | Item 2/6 own templated hints; Knowledge gets fresh, de-templated `Example.hints` | Yes — new authoring | Depends on generated values |
| `provenanceRefs` | Bare source-id array | Restructure | `Example.provenance_refs` | Yes, all 7 | Same as concepts/objectives |
| `status` | Editorial review flag | Remove entirely | Editorial state lives outside the package (`.ai/tasks/`) | N/A | Not declarative subject knowledge |
| **`pf-method-select-integral-count.json` specifically** | Currently `status: needs-review` | Does not migrate | Not present in Knowledge v1 | A task, not a field migration | It does not migrate into Knowledge v1. Preserved as design/source-review evidence until the unresolved source question is settled. If it later proves useful and source-supported, a fixed static `Example` may be authored from it; otherwise it may simply disappear from canonical Knowledge. The schema is not obligated to preserve every prototype artifact. |
| **`synthesis-report.md`** | Human rationale narrative | Keep, reclassified | Non-schema authoring/rationale documentation, never loaded by the runtime parser | Content rewrite to match the new shape; role unchanged | Never machine-loaded schema data |

**Consolidated identifier-grammar violations**: every ID in the current package needs
renaming — `package.json.id`, all 3 concept IDs, all 6 objective IDs, all 7 problem-family
IDs, and 10 of 11 provenance entries. The grammar-reuse decision (§6) has real, repository-
wide migration cost, not a cosmetic one.

**Consolidated dual-authored-field cleanup**: `Concept.objectiveIds` and
`method-selection-volume-of-revolution`'s duplicate `related_ids`/`prerequisite_ids` entries
are the two concrete instances of exactly the drift pattern §6's single-authored-direction
rule exists to prevent.

**Housekeeping, not a schema decision**: `ARCHITECTURE.md:77`'s "ad hoc until Knowledge
Package v1" line needs updating once this is adopted — noted here as a consequence, not a
reconciliation row, since it's documentation *about* the package, not part of it.

## 10. Alternatives Considered

| Fork | Alternatives compared | Recommended | Why |
|---|---|---|---|
| A. Domain model shape | Concept-centric nesting · Normalized entity graph · Document-centric/semantic-block | **Normalized entity graph** | Matches what the existing package already does usefully; supports independent lookup; doesn't collapse Knowledge into a Docling-shaped document |
| B. Relationship representation | Generic edge entity · Multiple authored inverse fields · Small explicit vocabulary + derived reverses | **`prerequisite_ids` / `related_ids`, reverses derived** | Two semantically distinct types suffice; authored inverses duplicate what's already derivable |
| C. Problem-family content | Preserve as Knowledge · Partially preserve generation metadata · Static `Example` only | **Static `Example` only** | Not a quality judgment — the existing work proved those fields belong to items 2/3/5/6 |
| D. Serialization/layout | Pure JSON · TOML frontmatter + Markdown body, one file · JSON + companion Markdown · Entity directories | **TOML frontmatter + Markdown body, one file** | Readable LaTeX/prose, clean diffs, no paired-file drift, no unnecessary file multiplication |
| E. Source/runtime workflow | Separate authored/compiled forms · Unified + `review_status` · Unified, editorial state outside schema | **Unified, no `review_status`** | No consumer needs a compiled artifact; review state isn't subject knowledge; this repository's task/review workflow already does the job |
| F. Provenance representation | Flat source-per-citation · `Source` + `SourceLocator` + `ProvenanceRef` · Field/block-level | **`Source` + `SourceLocator` + `ProvenanceRef`, entity granularity, `Direct`/`Derived`** | The current package's 11-entries-for-1-book redundancy is direct proof of the flat model's failure |
| G. Identifier strategy | Preserve ad hoc slugs · Invent a Knowledge-specific grammar · Reuse `identifier.rs`'s grammar, distinct types | **Shared lexical grammar, distinct domain types** | Reuses proven, tested validation; dot segments carry no hierarchy/taxonomy meaning |

None of the rejected alternatives are wrong in principle, only unjustified for v1:

- A separate source/runtime compilation stage becomes worth its cost once a real
  Docling-assisted authoring pipeline exists and drafts genuinely need to be structurally
  unreachable from runtime.
- Field-level provenance becomes worth its cost if entity-level provenance is ever shown to
  actively mislead a consumer, not merely be less precise.
- Richer relationship types become worth their cost once a concrete consumer names a need
  for them.
- Structured `pages: {from, to}` becomes worth its cost once something needs to sort or
  range-check page numbers.

The governing principle throughout: **defer capability until a demonstrated consumer
requires it**, not until it sounds educationally complete.

## 11. Recommended v1 Direction

A normalized entity graph of five primary entities (`Package`, `Concept`, `Objective`,
`Example`, `Source`), each with exactly one authored identity and forward-reference
direction; every reverse view is derived by the loader, never separately authored. The full
type shapes are in §6; the on-disk layout, frontmatter grammar, and body grammar are in §7;
the provenance model is in §8.

Illustrating with real content — `concepts/shell.method_vertical_axis.md`:

```text
+++
id = "shell.method_vertical_axis"
name = "The Method of Cylindrical Shells (Vertical Axis of Revolution)"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = ["shell.method_horizontal_axis", "method_selection.volume_of_revolution"]

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

and `examples/shell.example_y_poly.md` — one fixed instantiation of the former
`pf-shell-y-poly` family (`coeff = 4, b = 3`):

```text
+++
id = "shell.example_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.compute_volume_y_axis_single_curve"]

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

This is specific enough that a formal spec pass can go directly to freezing field types, the
exact TOML/Markdown grammar, and the loader's validation rules — no open questions remain in
the core model itself; what remains open (§12) is deliberately narrow and mostly about the
edges of this shape, not its center.

## 12. Open Questions

The architectural center is settled: normalized entity graph; `Package`/`Concept`/
`Objective`/`Example`/`Source`; TOML frontmatter + Markdown body; one entity per file;
`prerequisite`/`related` relationships; entity-level provenance; `Direct`/`Derived`; no
problem families; no learner state; no verification machinery; no editorial state. The
remaining questions are narrow.

### Blocking

1. **Package/Source file layout.** `package.toml` (single, frontmatter-only) + a single
   `sources.toml` (`[[source]]` array-of-tables) + per-entity directories for `Concept`/
   `Objective`/`Example`. `Source`'s low cardinality and low edit-frequency, unlike
   `Concept`/`Objective`/`Example`, don't justify one-file-per-entity for it specifically.
2. **File extension and frontmatter grammar.** `.md` for entity files, `.toml` for
   `package.toml`/`sources.toml`; `+++` delimiter; UTF-8 required; BOM rejected; parser
   accepts LF or CRLF, LF is the authoring convention enforced by tooling, not the parser.
3. **Body grammar per entity type.** `Concept`/`Objective`: whole body = `description`.
   `Example`: `## Problem`/`## Solution` required, `## Hints` optional, fixed order,
   duplicate or unknown headings invalid, each top-level list item under `Hints` is one
   `String`, non-list content under `Hints` invalid.
4. **`related_ids` validation invariants.** No dual-authorship, no self-reference, no
   duplicates, symmetric in memory, only the authored side is ever (re-)serialized.
5. **Prerequisite cycle semantics.** `prerequisite_ids` must form a DAG; the rule applies
   only to `prerequisite_ids`, never to `related_ids`.
6. **Reference scope.** Package-local only; no cross-package entity references in v1.
7. **ID uniqueness scope.** Per entity kind, not package-global.
8. **Package ID vs. entity ID relationship.** Fully independent; no prefixing; package
   ownership never inferred from entity ID segments.
9. **Provenance requiredness.** Required — every `Concept`/`Objective`/`Example` must carry
   ≥1 `ProvenanceRef`; original Axiom content satisfies this via an Axiom-authored `Source`.
10. **`Direct`/`Derived` semantics.** Definitions as given in §8; both may coexist on one
    entity; no confidence scores or finer-grained kinds.

All ten have a recommended answer above with supporting evidence — none were left
artificially undecided. All must be resolved, exactly as recommended above, before the
formal specification can be written unambiguously.

### Non-blocking

- **Structured page ranges** — `pages: Option<String>` stays free-form until range
  arithmetic or sorting is needed.
- **Source licensing policy** — `license` stays a stable identifier; compatibility/
  redistribution/attribution-rendering policy is later work.
- **Topic structure** — `Concept.topic` stays a flat label; a future hierarchy is unforced.
- **Field-level provenance** — stays a non-goal unless entity-level provenance is shown to
  mislead.
- **Source/runtime compilation** — deferred until a real authoring pipeline justifies it.
- **Docling metadata** — never enters the schema; an ingestion IR's own concern if one
  exists later.
- **Richer relationship vocabulary** — no demonstrated need beyond `prerequisite`/`related`.
- **Parent/child concept hierarchy** — no demonstrated need.
- **Markdown rendering subset** — the schema only needs enough grammar to parse entity
  boundaries (§7); it does not need to define a math/Markdown rendering contract, and this
  repository doesn't have a canonical one to inherit yet.

## 13. Explicit Non-Goals for v1

- Problem generation, parameterization, or deterministic seeding (roadmap items 2, 5).
- Mathematical/symbolic verification (`math.verify`, item 3).
- Practice orchestration — difficulty selection, session sequencing, spaced repetition,
  adaptive selection, attempt state, scoring policy, hint timing/delivery, mastery state
  (item 6).
- Tutor prompt or interaction design — Tutor consumes Knowledge, it is not part of the
  schema.
- A Docling ingestion pipeline — the schema must be populatable by one later, without being
  dependent on one now.
- Editorial/review workflow state serialized into the package — handled entirely by this
  repository's existing `.ai/tasks/` process; unresolved content is simply absent.
- A separate authored-source vs. compiled-runtime package split.
- Field- or content-block-level provenance.
- Cross-package references, multi-package composition, marketplace distribution, or
  signing — v1 references are package-local only.
- A generic relationship/graph-edge system — only `prerequisite` and `related`.
- A curriculum-standards or topic-hierarchy ontology — a flat display label only.
- Confidence scores, extraction-model lineage, OCR metadata, or any other
  ingestion-provenance detail beyond `Source`/`SourceLocator`/`ProvenanceKind`.

## 14. Proposed Next Step

```text
brainstorm (this document)
        ↓
formal Knowledge Package v1 specification
        ↓
implementation plan
        ↓
implementation + Calc II package reconciliation
```

The next pass is the **formal Knowledge Package v1 specification**, the same weight
`CORE.md` and the module-capability design document got for sub-project 1. It freezes what
this document deliberately left as recommendations and open questions: exact domain field
types (Rust structs, not pseudotypes), the exact TOML+Markdown grammar, the exact file/
directory layout, exact identifier/reference rules, exact validation invariants, exact
provenance semantics, and a set of canonical worked examples drawn from §11's illustrations
and the full §9 reconciliation table — normative enough that migrating the existing Calc II
package against the agreed contract becomes a mechanical exercise, not another design pass.

No implementation plan is written here, and `knowledge-package/` is not migrated here — both
are later steps, gated on the formal specification existing first.
