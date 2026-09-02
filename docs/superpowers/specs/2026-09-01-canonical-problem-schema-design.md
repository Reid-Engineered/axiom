# Canonical Problem Schema — design

Sub-project 1 of 6 in the "Practice engine" initiative Marcus scoped on 2026-09-01, itself
the remaining scope of `ROADMAP.md`'s Stage 8 (everything after the module/capability
runtime, which is done — tasks 045–048). The six sub-projects, in the order the roadmap
already prescribes and each dependent on the last: **(1) canonical Problem schema** (this
doc), (2) `math.verify` with Symbolica CAS, (3) deterministic seeded problem generation,
(4) the Practice Core Utility (`practice.generate`/`practice.evaluate`/`practice.hint`),
(5) Study Session UI integration, (6) an offline acceptance test.

Both of Marcus's scoping choices point the same direction: real deterministic generation
(not hand-authored fixed problems) and full Symbolica CAS integration (not a scoped-down
deterministic-only verifier) — build the real thing, efficiently, not a stopgap.

Source material: `ROADMAP.md` Stage 8, `CORE.md` (capability runtime this schema's
`generator`/verification fields will eventually be resolved through), the Knowledge Package
v1 spec (`docs/superpowers/specs/2026-08-30-knowledge-package-v1-spec.md`) — whose file
format, frontmatter grammar, and reference-validation machinery this schema reuses directly
— and seven pre-migration `knowledge-package/problem-families/*.json` prototype files
(recovered from git history at commit `17d8aa36c7c9be2dd51d697cbfa61dc5a0035b5e^`, since the
Knowledge Package v1 migration removed them from the working tree but its own spec §19
explicitly flags them as "design evidence for roadmap items 2 (Canonical Problem)... — not
designed here"). This design is built directly from that evidence, not from a blank page.

---

## 1. Scope

Define the data shape of a canonical Problem — both the authored template
(`ProblemFamily`) and the generated, learner-facing realization of one (`ProblemInstance`)
— plus the file format and validation rules for `ProblemFamily` as Knowledge Package
content. This sub-project does **not** build: the actual generator functions (sub-project
3), `math.verify` itself (sub-project 2, though this schema's `response_type` field is
designed to name the verification mode that capability will need), or anything UI-facing
(sub-project 5).

## 2. Decisions carried in from brainstorming

- **Two types, two lifecycles.** `ProblemFamily` is authored Knowledge Package content
  (same `+++` TOML-frontmatter + Markdown convention as `Concept`/`Objective`/`Example`,
  same parser, same reference-validation pass). `ProblemInstance` is the runtime-only
  output of generation — not authored, not a file format, not persisted by this
  sub-project (whether/how instances get saved as session history is the Practice Core
  Utility's question, sub-project 4).
- **Constraints are parsed, structured expressions — but scoped to parameter arithmetic.**
  The prototypes' free-text constraints split into two real categories: parameter-bound
  relationships (`"b <= coeff"`) and domain-validity claims (`"f(x) >= 0 for all x in
  [a,b]"`). Only the first category is this schema's concern, represented as a real parsed
  expression tree evaluable in Rust — authored as a familiar compact string
  (`constraints = ["b <= coeff"]`), parsed at load time, not stored as opaque text. Domain-
  validity claims are a generator-author's responsibility, proven by property-based tests
  on the generator's Rust code (the roadmap's own later "Practice's own heavy testing bar"
  item) — building generic domain-validity proving into the schema layer would be a
  fundamentally bigger undertaking (interval/sign analysis) than a schema should own.
- **Most prototype "constraints" turned out to be redundant with parameter bounds.**
  `"coeff >= 2 and coeff <= 6"` just restates `parameters.coeff`'s own `min`/`max`. The
  `constraints` array is only needed for relationships a single parameter's own bounds
  can't express.
- **Long-form content lives in the Markdown body, not TOML frontmatter** — following
  `Example`'s own precedent (`## Problem`/`## Solution`/`## Hints` headings) rather than
  cramming LaTeX-heavy prompt/solution/hint text into single-line TOML strings.

## 3. `ProblemFamily`

```rust
struct ProblemFamily {
    id: ProblemFamilyId,
    concept_id: ConceptId,
    objective_ids: Vec<ObjectiveId>,           // non-empty
    difficulty: DifficultyRange,                // { min: u8, max: u8 }, min <= max
    generator: GeneratorRef,                    // { id: String, version: u32 }
    parameters: BTreeMap<String, ParameterSpec>,
    constraints: Vec<ConstraintExpr>,           // parsed from authored strings
    response_type: ResponseType,                // tagged union, extensible
    canonical_solution: CanonicalSolution,      // shape tied to response_type
    hints: Vec<Hint>,                           // ordered by level, may be empty
    provenance_refs: Vec<ProvenanceRef>,        // reuses the existing type exactly
    status: ProblemFamilyStatus,                // verified | needs-review
}

enum ParameterType { Integer, Float }

#[serde(untagged)]
enum Bound {
    Literal(f64),
    Reference { parameter: String, #[serde(default)] offset: f64 },
}

struct ParameterSpec {
    #[serde(rename = "type")]
    kind: ParameterType,
    value: Option<Bound>,   // fixed constant; mutually exclusive with min/max
    min: Option<Bound>,
    max: Option<Bound>,
    description: Option<String>,
}

enum ResponseType {
    SymbolicExpression,
    Numeric,
}

enum CanonicalSolution {
    Symbolic { expression: String },   // resolved by response_type == SymbolicExpression
    Numeric { value: f64 },            // resolved by response_type == Numeric
}

struct Hint {
    level: u32,   // unique, positive, ascending
    // kind (e.g. "prompt"/"principle"/"setup"/"evaluation" from the prototypes) is
    // free-form authoring metadata, not schema-enforced — the body text is what matters
}

enum ProblemFamilyStatus { Verified, NeedsReview }
```

### Constraint expressions

```rust
enum Term {
    Param(String),
    Literal(f64),
    BinaryOp { op: ArithOp, left: Box<Term>, right: Box<Term> },
}
enum ArithOp { Add, Sub, Mul, Div }
enum CompareOp { Eq, Ne, Ge, Le, Gt, Lt }

enum ConstraintExpr {
    Comparison { left: Term, op: CompareOp, right: Term },
    All(Vec<ConstraintExpr>),   // conjunction — matches the prototypes' "and" usage
}
```

Authored as compact strings (`constraints = ["b <= coeff", "a + b < 10"]`), parsed into this
tree by a small recursive-descent parser at load time. A parse error, or a `Term::Param`
naming something not declared in `parameters`, is a structural validation failure — same
tier and same failure path as a TOML syntax error.

## 4. `ProblemInstance` (runtime, not a file format)

```rust
struct ProblemInstance {
    family_id: ProblemFamilyId,
    seed: u64,
    resolved_parameters: BTreeMap<String, f64>,
    prompt: String,                        // template rendered with resolved values
    canonical_solution: ResolvedSolution,   // expression/value with params substituted
    hints: Vec<String>,                     // rendered
}

enum ResolvedSolution {
    Symbolic(String),
    Numeric(f64),
}
```

Produced by `practice.generate(family, seed)` — sub-project 4's job, not built here. This
sub-project only fixes the shape sub-project 4 must produce.

## 5. File format

`ProblemFamily` files live under a new `problems/` directory in the Knowledge Package,
sibling to `concepts/`, `objectives/`, `examples/`. Same `+++`-delimited TOML frontmatter +
Markdown body convention as every other entity (KP v1 spec §6, reused verbatim — UTF-8
required, `+++` must open the file, unknown frontmatter keys rejected, LF/CRLF both
accepted).

```toml
+++
id = "problem.shell_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_y_axis_single_curve"]
difficulty = { min = 1, max = 2 }
generator = { id = "gen-shell-y-poly", version = 1 }
response_type = "symbolic-expression"
status = "verified"

[parameters.coeff]
type = "integer"
min = 2
max = 6
description = "Linear coefficient for quadratic curve f(x) = c*x - x^2"

[parameters.a]
type = "integer"
value = 0

[parameters.b]
type = "integer"
min = 1
max = { parameter = "coeff" }

[canonical_solution]
expression = "2*pi*(coeff*b^3/3 - b^4/4)"

[[hints]]
level = 1

[[hints]]
level = 2

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"
+++

## Prompt

Define R as the region bounded above by the graph of f(x) = {coeff}x - x^2 and below by
the x-axis over the interval [{a}, {b}]. Find the volume of the solid of revolution formed
by revolving R around the y-axis.

## Solution

V = \int_{a}^{b} 2\pi x f(x)\,dx = 2\pi \int_{0}^{b} (coeff\,x^2 - x^3)\,dx
  = 2\pi \left[\frac{coeff\,x^3}{3} - \frac{x^4}{4}\right]_0^b

## Hints

1. Identify the shell radius and shell height as functions of x for rotation around the
   y-axis.
2. For a region bounded by y = f(x) revolved around the y-axis, the shell radius is
   r(x) = x and the height is h(x) = {coeff}x - x^2.
```

The `## Hints` section's numbered list entries correspond 1:1, in order, to the
frontmatter's `[[hints]]` array (which carries only `level`); the hint's actual text lives
in the body, matching how `## Problem`/`## Solution`/`## Hints` already work for `Example`.

## 6. Validation

Layered on top of the existing pass — `ProblemFamily` loads in the *same* pass as
`Concept`/`Objective`/`Example`, sharing the in-memory concept/objective maps already being
built, not a separate re-parse:

- `concept_id` and every `objective_ids` entry must resolve to an existing `Concept` /
  `Objective` in the package (reuses KP v1 spec §12's reference-resolution machinery).
- Every parameter cross-reference (`{ parameter = "coeff" }`) must resolve to a parameter
  declared in the same family, and parameter references must not cycle (small-graph check,
  same idea as the existing prerequisite-DAG rule, much smaller graphs here — typically
  2–5 parameters).
- Constraint expressions must parse successfully and reference only declared parameters.
- `response_type` and `canonical_solution`'s variant must match.
- `difficulty.min <= difficulty.max`.
- Hint `level`s are unique, positive integers.
- `generator.id` is validated against the same identifier grammar as every other id in this
  package (KP v1 spec §2) — but **not** checked against an actually-registered Rust
  function. That binding happens in sub-project 3; this schema only validates the string's
  shape, since the generator registry doesn't exist yet.
- `provenance_refs`: reuses the existing rule exactly — at least one entry required.

## 7. Testing

Same bar as Stage 8 sub-project 1 (module/capability runtime): provable entirely through
`cargo test`, no UI required. A canonical worked example — literally migrating
`pf-shell-y-poly` (the prototype read directly in this brainstorm) into the new format —
plus a conformance-case table covering each rejection class in §6: dangling `concept_id`/
`objective_ids`, dangling or circular parameter reference, constraint parse error,
constraint referencing an undeclared parameter, `response_type`/`canonical_solution`
mismatch, malformed `difficulty` range, duplicate hint `level`.

## 8. Follow-ups (out of scope here, tracked for later)

- Sub-project 2: `math.verify` capability with Symbolica CAS — this schema's
  `response_type`/`canonical_solution` fields are the contract that capability will consume,
  but building it is out of scope here.
- Sub-project 3: the actual `gen-shell-y-poly`-style generator functions, including the
  property-based tests that prove domain-validity constraints this schema deliberately
  doesn't formalize.
- Sub-projects 4–6: Practice Core Utility, Study Session UI integration, offline acceptance
  test.
- Migrating the six already-`Example`-migrated problem families (plus deciding the fate of
  the seventh, excluded `pf-method-select-integral-count`) into the new `problems/` format
  is real content-authoring work for whichever task implements sub-project 3, not part of
  defining the schema itself.
