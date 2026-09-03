# Problem generation engine — design

## 1. Scope

Roadmap Stage 8, sub-project 3 of 6 in the "Practice engine" initiative, per the numbering
task 054 and 055 both use: "the actual generator functions or domain-validity property
tests." Follows directly on the canonical Problem schema
(`.ai/tasks/_archive/054-canonical-problem-schema.md`) and `math.verify`
(`.ai/tasks/_archive/055-math-verify.md`), both merged.

Add a `generate_problem_instance(family: &ProblemFamily, seed: u64) -> Result<ProblemInstance,
GenerationError>` engine: given an authored `ProblemFamily` (`src-tauri/src/knowledge/
types.rs`) and a seed, deterministically produce a concrete `ProblemInstance` — sampled
parameters, a prompt with placeholders filled in, a canonical solution with parameters
substituted, and hint text with placeholders filled in.

**Does not build** (per brainstorming — scope decision, engine-only): authoring real
`ProblemFamily` content into `knowledge-package/`'s reference package (the engine is proven
against the existing test fixture, `tests/fixtures/canonical/problems/problem.shell_y_poly.md`,
which already exercises every schema feature this engine needs to handle); the Practice Core
Utility that will call this engine (sub-project 4); bespoke bound/constraint-defying
generator logic for hypothetical future families (see §8) — v1 ships one generic,
data-driven pipeline plus one dispatch entry for `gen.shell_y_poly`.

## 2. Decisions carried in from brainstorming

- **Engine-only scope.** No new authored content in `knowledge-package/` — this sub-project
  proves the mechanism against the existing fixture. Real reference-package authoring is a
  follow-up.
- **Hand-rolled deterministic PRNG, not `rand`.** Consistent with this project's pattern so
  far (no new production dependency where a small, auditable, well-known algorithm suffices —
  `constraint.rs`'s hand-written parser, `math.verify`'s narrow use of one existing crate
  rather than pulling in more). SplitMix64 (§6) is simple enough to fully specify and audit
  here, and this only needs uniform sampling over small integer/float ranges, not
  cryptographic quality.
- **Hand-rolled property-test loop, not `proptest`.** A plain `#[test]` running the generator
  across ~10,000 deterministic seeds and asserting a domain predicate, using the same PRNG.
  No new dependency (dev or production). Less polished failure reporting than `proptest`'s
  shrinking, but reproducible (a failing seed is directly re-runnable) and simple to read.
- **Generic sampling engine, not per-family bespoke generation code.** The one real example
  (`gen.shell_y_poly`) turns out not to need bespoke sampling logic at all: its domain-
  validity requirement (the height function stays non-negative over the sampled interval)
  is already fully captured by the existing schema — `b`'s `max` bound references `coeff`
  directly (`max = { parameter = "coeff" }`). What's genuinely bespoke per family is the
  **property test** (each family has its own mathematical claim to state), not the
  parameter-generation mechanism. `generate_problem_instance` therefore dispatches on
  `GeneratorId` via a plain `match`, not a capability-style registry — generators aren't
  third-party-swappable like `math.verify`, so the module-capability runtime doesn't apply
  here. Every dispatch arm today (there is exactly one) delegates to the same generic
  pipeline; a future family needing genuinely custom sampling gets its own match arm without
  requiring a redesign of the dispatch mechanism.
- **Constraint evaluation is a genuine gap this sub-project fills.** `constraint.rs`
  (`src-tauri/src/knowledge/constraint.rs`) currently only *parses* a `ConstraintExpr` into
  an AST for structural validation — nothing in the codebase evaluates one against concrete
  parameter values yet. This engine is the first real consumer that needs to, so the
  `evaluate`/`holds` methods are added to `constraint.rs` itself (next to the AST they
  operate on), not duplicated in the new `generation/` module — the reuse mistake task 054's
  review already flagged once (duplicated cycle-detection) is exactly what this avoids.

## 3. Location and entry point

New top-level module `src-tauri/src/generation/`, sibling to `knowledge/`, `modules/`, and
`capabilities/`. This is a runtime operation (produce a concrete instance from already-loaded
content) — a different responsibility from `knowledge/`'s "load and validate a package from
disk," the same reasoning that put `capabilities/` in its own top-level module rather than
inside `knowledge/` even though it also operates on `knowledge/`'s types.

File split, following the established convention (`types.rs`/`error.rs`/one file per real
responsibility, inline `#[cfg(test)] mod tests`, a `tests/` directory for the cross-cutting
property test):

- `mod.rs` — the one public entry point:
  ```rust
  pub fn generate_problem_instance(
      family: &ProblemFamily,
      seed: u64,
  ) -> Result<ProblemInstance, GenerationError>
  ```
- `error.rs` — `GenerationError` (§7).
- `rng.rs` — `DeterministicRng` (§6).
- `sampling.rs` — parameter resolution and constraint-checked resampling (§4).
- `template.rs` — the two substitution mechanisms (§5).
- `tests/mod.rs` — the domain-validity property test for `gen.shell_y_poly` (§8).

## 4. Parameter resolution and constraint checking

For each of `ProblemFamily.parameters: BTreeMap<String, ParameterSpec>`, resolve a concrete
`f64`, via a memoized recursive walk (a parameter already resolved this attempt is returned
from the memo, not re-sampled) — safe to assume cycle-free, since Knowledge validation
(`src-tauri/src/knowledge/problem_family.rs`'s `validate_parameter_references`, task 054)
already rejects reference cycles at package-load time; that is a precondition this module
trusts rather than re-checks (this module has no way to construct a `ProblemFamily` other
than through that validated load path).

- A parameter with a `value` bound resolves directly to that bound's value (no randomness).
- Otherwise, resolve `min` and `max` (each a `Bound::Literal(f64)` or a
  `Bound::Reference { parameter, offset }`, resolved recursively the same way) and sample
  uniformly within `[min, max]`, inclusive, respecting `ParameterType`:
  `Integer` rounds `min`/`max` to whole numbers first, then samples an integer in that
  inclusive range; `Float` samples continuously.
- If a parameter has neither `value` nor both `min` and `max` — schema-valid today (task
  054's validation only forbids `value` *and* bounds together, never requires either) but
  unsampleable — resolution fails with `GenerationError::UnderspecifiedParameter` (§7). This
  is a real gap in the existing schema; the actual fix belongs in Knowledge-authoring-time
  validation, out of scope here (§8).

Once every parameter resolves, evaluate every `ProblemFamily.constraints: Vec<ConstraintExpr>`
entry against the resolved values via the new `ConstraintExpr::holds` (§2). If any constraint
fails, discard this attempt's resolved values and retry from scratch with a fresh draw from
the (still-advancing) RNG, up to `MAX_RESAMPLE_ATTEMPTS = 1000`. Exceeding that returns
`GenerationError::ConstraintsUnsatisfiable` — a too-tight or self-contradictory constraint
set fails loudly at generation time instead of looping. (For `gen.shell_y_poly` specifically
there are no `[[constraints]]` entries in the fixture — the sole cross-parameter relationship
is already expressed as `b`'s `max` bound — so this path is exercised by a synthetic fixture
in testing, not by the one real example family.)

## 5. Template substitution

Two independent, purpose-built mechanisms — the fixture uses two different conventions and
neither can substitute for the other:

- **Prompt and hint text** (`{param}` curly-brace placeholders, e.g. `"f(x) = {coeff}x -
  x^2"`): literal substring replacement, one pass per resolved parameter
  (`template.replace(&format!("{{{name}}}"), &format_number(value))`). Unambiguous — curly
  braces unambiguously delimit the placeholder, no word-boundary concern.
- **`canonical_solution.expression`** (bare identifiers inside real math syntax, e.g.
  `"2*pi*(coeff*b^3/3 - b^4/4)"`): must remain a valid `mathcore`-parseable expression after
  substitution, so a curly-brace scheme can't be used here — the string needs to stay
  `"2*pi*(4*3^3/3 - 3^4/4)"`-shaped, not `"2*pi*({4}*{3}^3/3 - {3}^4/4)"`. This needs a small
  word-boundary-aware identifier scanner: find maximal runs of `[A-Za-z_][A-Za-z0-9_]*`, and
  replace a run with its formatted value only if it exactly equals a declared parameter name
  — anything else (a `mathcore` constant like `pi`, a function name like `sin`) is left
  untouched. Deliberately not reusing `constraint.rs`'s parser (no `^`/power operator in that
  grammar) or `mathcore`'s parser (would fully evaluate to a float, destroying the
  intentionally-unreduced symbolic form `ResolvedSolution::Symbolic` exists to preserve for
  display).

Both are pure string transforms with no failure mode of their own (an identifier that
doesn't match any parameter name is simply left alone) — neither needs a `GenerationError`
variant.

`format_number` formats an integer-typed value without a trailing `.0` (e.g. `4` not `4.0`)
and a float-typed value with enough precision to round-trip; the exact formatting only
matters for the substituted text a learner reads and the substituted expression `mathcore`
re-parses, both of which tolerate ordinary `f64` `Display` formatting for whole numbers
already, so no bespoke formatting logic beyond checking `value.fract() == 0.0`.

Hints substitute in the same order the family's `hints: Vec<Hint>` are already ordered
(ascending `level`, already enforced by task 054's validation) — `ProblemInstance.hints:
Vec<String>` is exactly that sequence with each `Hint.text` substituted.

`CanonicalSolution::Numeric { value }` carries a bare literal, not a template — the current
schema has no expression mechanism for the `Numeric` response type (only `Symbolic` carries
an `expression: String`). Generation for that case is a direct copy: `ResolvedSolution::
Numeric(value)`, no substitution possible or needed. This is a real limitation of the
already-locked schema (a `Numeric`-response family's canonical answer cannot depend on its
own sampled parameters) — not something to fix here (§8), and irrelevant to the one real
example family, which uses `SymbolicExpression`.

## 6. Determinism: `DeterministicRng`

SplitMix64 — fully specified so this is implemented once, correctly, not reinvented:

```rust
pub(crate) struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}
```

Two sampling helpers built on `next_u64`:

- Integer, inclusive `[min, max]`: `min + (rng.next_u64() % (max - min + 1)) as i64`. The
  modulo-bias this introduces is real but astronomically negligible for the range sizes this
  schema actually produces (single digits to low hundreds against a 64-bit generator) — not
  worth rejection-sampling complexity for this use case; noted here explicitly so it reads as
  a deliberate tradeoff, not an oversight.
- Float, continuous `[min, max]`: `min + (rng.next_u64() as f64 / u64::MAX as f64) * (max -
  min)`.

Determinism guarantee: the same `(family, seed)` pair always produces byte-identical
`ProblemInstance` output, forever — this is the property Practice (sub-project 4) and any
future regression/grading tooling will depend on. `next_u64` is called in a fixed, spec-
defined order per resolution/resample attempt (§4), so nothing about iteration order (e.g.
`BTreeMap`'s already-deterministic ascending-key order) introduces nondeterminism.

## 7. `GenerationError`

```rust
pub enum GenerationError {
    UnknownGenerator { id: GeneratorId },
    UnderspecifiedParameter { family_id: ProblemFamilyId, parameter: String },
    ConstraintsUnsatisfiable { family_id: ProblemFamilyId, attempts: u32 },
}
```

`UnknownGenerator` is a dispatch miss (a `ProblemFamily.generator.id` this build has no match
arm for — an authoring/deployment mismatch, not a runtime data problem). The other two are
documented in §4. All three are genuine failures (never silently degrade to a wrong-but-
plausible instance) — mirrors `KnowledgeError`/`MathVerifyError`'s existing convention:
hand-written `Display`, no `thiserror`.

## 8. Testing

- `rng.rs`: `DeterministicRng::new(seed).next_u64()` is repeatable (same seed, same
  sequence); different seeds diverge; the two sampling helpers stay within `[min, max]`
  across many draws and hit both endpoints over enough draws.
- `sampling.rs`: a fixed-value parameter resolves without consuming RNG state; a
  `Bound::Reference` resolves relative to its target; a multi-level reference chain (`c`
  depends on `b` depends on `coeff`) resolves in dependency order; a parameter with neither
  `value` nor both bounds returns `UnderspecifiedParameter`; a synthetic fixture with an
  unsatisfiable constraint set returns `ConstraintsUnsatisfiable` after exactly
  `MAX_RESAMPLE_ATTEMPTS` attempts; a synthetic fixture with a satisfiable-but-narrow
  constraint eventually succeeds.
- `template.rs`: curly-brace substitution against the real fixture's prompt/hint text;
  identifier substitution against the real fixture's `canonical_solution.expression`,
  including a case proving `pi` is left untouched while `coeff`/`b` are replaced; a
  parameter name that doesn't appear in the template is simply not present in the output
  (not an error).
- `constraint.rs` additions: `Term::evaluate`/`ConstraintExpr::holds` against hand-computed
  cases for every `ArithOp`/`CompareOp` variant.
- **The domain-validity property test** (`generation/tests/mod.rs`), the actual deliverable
  ROADMAP.md names for this sub-project: generate `gen.shell_y_poly` instances for seeds `0`
  through `9999`, and for each, sample the height function `h(x) = coeff*x - x^2` at, say, 50
  evenly-spaced points across `[0, b]` using that instance's own `resolved_parameters`,
  asserting `h(x) >= 0` at every point for every seed. This is the family-specific
  mathematical claim the schema deliberately doesn't formalize (task 054's own scope note);
  proving it holds across thousands of seeds is real evidence the generic sampling engine
  and this family's declared bounds actually cooperate to produce a geometrically valid
  problem every time, not just typically.
- One full-pipeline test: `generate_problem_instance` against the real fixture for a fixed
  seed, asserting the produced `ProblemInstance`'s `prompt` and `canonical_solution` contain
  no unsubstituted `{...}` placeholders and no bare `coeff`/`b`/`a` identifiers, and that
  `mathcore::MathCore::new().calculate(...)` (called directly — `mathcore` is already a
  project dependency since task 055; this is a test-only sanity check, not a new production
  dependency between `generation/` and `capabilities/`) succeeds against the substituted
  `canonical_solution` expression, confirming it's genuinely `mathcore`-parseable.

## 9. Follow-ups (out of scope here, tracked for later)

- Knowledge-authoring-time validation (task 054's module) should require every
  `ParameterSpec` have either a fixed `value` or both `min` and `max` — today's schema
  allows an unsampleable parameter to pass authoring-time validation, only failing here, at
  generation time, instead. A real fix belongs in `problem_family.rs`, not this module.
- `CanonicalSolution::Numeric` has no expression/template mechanism, so a `Numeric`-response
  family's canonical answer cannot depend on its own sampled parameters. Not a problem for
  the one real example (`SymbolicExpression`), but a real limitation the schema will need to
  address before a genuinely numeric-answer generative family can be authored.
- Real reference-package content authoring (`knowledge-package/problems/`) — deferred per
  scope decision, engine-only for this sub-project.
- Bespoke non-generic generator functions for a future family whose domain-validity can't be
  expressed as declarative bounds/constraints (the `match`-based dispatch in `mod.rs` is
  built to accept new arms without redesign).
- Sub-project 4 (Practice Core Utility, the actual caller of this engine), sub-projects 5–6
  (Study Session UI, offline acceptance test).
