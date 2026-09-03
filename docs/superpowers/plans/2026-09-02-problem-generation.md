# Problem Generation Engine Implementation Plan

> **For agentic workers:** this plan is handed to Codex to implement directly against `.ai/tasks/`, `AGENTS.md`, and `CLAUDE.md`'s normal workflow — not executed via `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Steps use checkbox (`- [ ]`) syntax for tracking regardless of executor.

**Goal:** Add `generate_problem_instance(family: &ProblemFamily, seed: u64) -> Result<ProblemInstance, GenerationError>` under `src-tauri/src/generation/` — given an authored `ProblemFamily` and a seed, deterministically sample parameters, substitute them into the prompt/hints/canonical solution, and produce a concrete `ProblemInstance`.

**Architecture:** New sibling module to `knowledge/`, `modules/`, `capabilities/`. A hand-rolled deterministic PRNG (SplitMix64) drives parameter sampling; dependency-aware resolution handles `Bound::Reference` (one parameter's range depending on another's sampled value); a reject-and-resample loop (capped) enforces declared `ConstraintExpr`s, whose actual evaluation is new — added to `knowledge/constraint.rs` itself, which today only parses constraints, never evaluates them. Two independent string-substitution mechanisms handle the two template conventions the schema already uses (curly-brace for prompt/hints, identifier-aware for the canonical-solution expression). Dispatch on `GeneratorId` is a plain `match`, not a registry — there is exactly one real generator today, and every family currently expressible by this schema needs only the one generic pipeline.

**Tech Stack:** Rust, no new dependencies (production or dev) — hand-rolled PRNG and a hand-rolled seed-loop property test, per the spec's explicit decision not to add `rand` or `proptest`.

**Spec:** `docs/superpowers/specs/2026-09-02-problem-generation-design.md`

## Global Constraints

- New code lives in `src-tauri/src/generation/`, plus additions to the existing
  `src-tauri/src/knowledge/constraint.rs` (new `Term::evaluate`/`ConstraintExpr::holds`
  methods) and one new top-level `pub mod generation;` line in `src-tauri/src/lib.rs`. No
  Tauri command, no frontend change, no new Cargo dependency.
- Do **not** add `rand`, `rand_chacha`, `proptest`, or any other crate. The PRNG (§ below)
  and the property test (Task 7) are both hand-rolled, per the spec's §2 decision.
- `generate_problem_instance` trusts, rather than re-checks, that `family.parameters` is
  already free of `Bound::Reference` cycles — that invariant is established by
  `src-tauri/src/knowledge/problem_family.rs`'s `validate_parameter_references`
  (task 054) at package-load time, before a `ProblemFamily` value can exist at all.
- `MAX_RESAMPLE_ATTEMPTS = 1000` (spec §4) — use this exact constant name and value.
- The PRNG is SplitMix64, specified byte-for-byte in the spec (§6) and reproduced exactly in
  Task 1 below — do not substitute a different algorithm.
- Follow `knowledge/`'s established conventions: `pub(crate)` on internal helpers, a single
  flat error enum with a hand-written `Display` (no `thiserror`), inline `#[cfg(test)] mod
  tests` in the file the code under test lives in, a `tests/` subdirectory for the one
  cross-file test (the property test, Task 7).
- The real canonical fixture (`src-tauri/src/knowledge/tests/fixtures/canonical/`, loaded via
  `load_knowledge_package`) is the source of truth for `gen.shell_y_poly`'s actual
  `ProblemFamily` shape — tests that need a realistic family load this fixture rather than
  hand-typing a duplicate of it. Synthetic families (for edge cases the real fixture doesn't
  cover, like an unsatisfiable constraint) are constructed directly as `ProblemFamily` struct
  literals — every field is `pub`, so no Markdown/TOML round-trip is needed for those.

---

### Task 1: `DeterministicRng`

**Files:**
- Create: `src-tauri/src/generation/rng.rs`

**Interfaces:**
- Produces: `DeterministicRng::new(seed: u64) -> Self`, `.next_u64(&mut self) -> u64`,
  `.sample_integer(&mut self, min: i64, max: i64) -> i64`, `.sample_float(&mut self, min: f64,
  max: f64) -> f64`. Task 4 consumes all four.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/generation/rng.rs`:

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

    pub(crate) fn sample_integer(&mut self, min: i64, max: i64) -> i64 {
        let range = (max - min + 1) as u64;
        min + (self.next_u64() % range) as i64
    }

    pub(crate) fn sample_float(&mut self, min: f64, max: f64) -> f64 {
        min + (self.next_u64() as f64 / u64::MAX as f64) * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_the_same_sequence() {
        let mut a = DeterministicRng::new(42);
        let mut b = DeterministicRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = DeterministicRng::new(1);
        let mut b = DeterministicRng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn sample_integer_stays_within_bounds_and_reaches_both_endpoints() {
        let mut rng = DeterministicRng::new(7);
        let mut saw_min = false;
        let mut saw_max = false;
        for _ in 0..10_000 {
            let value = rng.sample_integer(3, 5);
            assert!((3..=5).contains(&value));
            saw_min |= value == 3;
            saw_max |= value == 5;
        }
        assert!(saw_min, "never sampled the minimum in 10,000 draws");
        assert!(saw_max, "never sampled the maximum in 10,000 draws");
    }

    #[test]
    fn sample_float_stays_within_bounds() {
        let mut rng = DeterministicRng::new(99);
        for _ in 0..10_000 {
            let value = rng.sample_float(-2.5, 4.5);
            assert!((-2.5..=4.5).contains(&value));
        }
    }
}
```

- [ ] **Step 2: Wire the module and run the tests**

In `src-tauri/src/lib.rs`, add (alphabetical, after `pub mod db;`):

```rust
pub mod generation;
```

Create `src-tauri/src/generation/mod.rs`:

```rust
mod rng;
```

Run: `cd src-tauri && cargo test --locked generation::rng -- --nocapture`

Expected: all 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/generation
git commit -m "feat(generation): add DeterministicRng (SplitMix64)"
```

---

### Task 2: Constraint evaluation

**Files:**
- Modify: `src-tauri/src/knowledge/constraint.rs`

**Interfaces:**
- Produces: `Term::evaluate(&self, params: &BTreeMap<String, f64>) -> f64`,
  `ConstraintExpr::holds(&self, params: &BTreeMap<String, f64>) -> bool`. Task 4 consumes
  `ConstraintExpr::holds`.
- Precondition (documented, not re-checked): every `Term::Param` name in `self` exists as a
  key in `params` — true by construction once Task 4 builds `params` from every one of a
  family's declared parameters, since `ConstraintExpr`s only ever reference that family's own
  parameters (enforced by a different, already-existing check elsewhere in `knowledge/`).

- [ ] **Step 1: Write the failing tests**

In `src-tauri/src/knowledge/constraint.rs`, add near the top (after the existing `use`
statements):

```rust
use std::collections::BTreeMap;
```

Add after the `ConstraintExpr` enum definition:

```rust
impl Term {
    pub(crate) fn evaluate(&self, params: &BTreeMap<String, f64>) -> f64 {
        match self {
            Term::Param(name) => params[name],
            Term::Literal(value) => *value,
            Term::BinaryOp { op, left, right } => {
                let left = left.evaluate(params);
                let right = right.evaluate(params);
                match op {
                    ArithOp::Add => left + right,
                    ArithOp::Sub => left - right,
                    ArithOp::Mul => left * right,
                    ArithOp::Div => left / right,
                }
            }
        }
    }
}

impl ConstraintExpr {
    pub(crate) fn holds(&self, params: &BTreeMap<String, f64>) -> bool {
        match self {
            ConstraintExpr::Comparison { left, op, right } => {
                let left = left.evaluate(params);
                let right = right.evaluate(params);
                match op {
                    CompareOp::Eq => left == right,
                    CompareOp::Ne => left != right,
                    CompareOp::Ge => left >= right,
                    CompareOp::Le => left <= right,
                    CompareOp::Gt => left > right,
                    CompareOp::Lt => left < right,
                }
            }
            ConstraintExpr::All(parts) => parts.iter().all(|part| part.holds(params)),
        }
    }
}
```

Add these tests to the existing `#[cfg(test)] mod tests` block in the same file:

```rust
    fn params(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), *value))
            .collect()
    }

    #[test]
    fn evaluates_literals_and_parameters() {
        let values = params(&[("b", 3.0)]);
        assert_eq!(Term::Literal(2.0).evaluate(&values), 2.0);
        assert_eq!(Term::Param("b".to_owned()).evaluate(&values), 3.0);
    }

    #[test]
    fn evaluates_every_arithmetic_operator() {
        let values = params(&[]);
        fn binary(op: ArithOp, left: f64, right: f64) -> Term {
            Term::BinaryOp {
                op,
                left: Box::new(Term::Literal(left)),
                right: Box::new(Term::Literal(right)),
            }
        }
        assert_eq!(binary(ArithOp::Add, 2.0, 3.0).evaluate(&values), 5.0);
        assert_eq!(binary(ArithOp::Sub, 5.0, 3.0).evaluate(&values), 2.0);
        assert_eq!(binary(ArithOp::Mul, 4.0, 3.0).evaluate(&values), 12.0);
        assert_eq!(binary(ArithOp::Div, 9.0, 3.0).evaluate(&values), 3.0);
    }

    #[test]
    fn holds_evaluates_every_comparison_operator() {
        let values = params(&[]);
        fn comparison(op: CompareOp, left: f64, right: f64) -> ConstraintExpr {
            ConstraintExpr::Comparison {
                left: Term::Literal(left),
                op,
                right: Term::Literal(right),
            }
        }
        assert!(comparison(CompareOp::Eq, 1.0, 1.0).holds(&values));
        assert!(comparison(CompareOp::Ne, 1.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Ge, 2.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Le, 2.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Gt, 3.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Lt, 2.0, 3.0).holds(&values));
        assert!(!comparison(CompareOp::Gt, 2.0, 3.0).holds(&values));
    }

    #[test]
    fn holds_requires_every_conjunct_to_hold() {
        let values = params(&[("b", 4.0)]);
        let expression = parse_constraint("family", "b >= 1 and b <= 10").unwrap();
        assert!(expression.holds(&values));

        let expression = parse_constraint("family", "b >= 1 and b <= 3").unwrap();
        assert!(!expression.holds(&values));
    }
```

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test --locked knowledge::constraint -- --nocapture`

Expected: all existing `constraint.rs` tests still pass, plus the 4 new ones.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/knowledge/constraint.rs
git commit -m "feat(knowledge): add ConstraintExpr::holds and Term::evaluate"
```

---

### Task 3: `GenerationError`

**Files:**
- Create: `src-tauri/src/generation/error.rs`
- Modify: `src-tauri/src/generation/mod.rs`

**Interfaces:**
- Consumes: `GeneratorId`, `ProblemFamilyId` (`crate::knowledge`, already exported).
- Produces: `GenerationError` — Task 4 constructs `UnderspecifiedParameter` and
  `ConstraintsUnsatisfiable`; Task 6 constructs `UnknownGenerator`.

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/generation/error.rs`:

```rust
use std::error::Error;
use std::fmt;

use crate::knowledge::{GeneratorId, ProblemFamilyId};

#[derive(Debug, PartialEq)]
pub enum GenerationError {
    UnknownGenerator {
        id: GeneratorId,
    },
    UnderspecifiedParameter {
        family_id: ProblemFamilyId,
        parameter: String,
    },
    ConstraintsUnsatisfiable {
        family_id: ProblemFamilyId,
        attempts: u32,
    },
}

impl fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownGenerator { id } => {
                write!(formatter, "no generator is registered for {id}")
            }
            Self::UnderspecifiedParameter {
                family_id,
                parameter,
            } => write!(
                formatter,
                "{family_id}: parameter {parameter:?} has neither a fixed value nor both a \
                 min and max bound, so it cannot be sampled"
            ),
            Self::ConstraintsUnsatisfiable {
                family_id,
                attempts,
            } => write!(
                formatter,
                "{family_id}: no combination of sampled parameters satisfied every declared \
                 constraint after {attempts} attempts"
            ),
        }
    }
}

impl Error for GenerationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_generator_displays_the_generator_id() {
        let error = GenerationError::UnknownGenerator {
            id: GeneratorId::new("gen.nonexistent").unwrap(),
        };
        assert_eq!(
            error.to_string(),
            "no generator is registered for gen.nonexistent"
        );
    }

    #[test]
    fn underspecified_parameter_displays_family_and_parameter() {
        let error = GenerationError::UnderspecifiedParameter {
            family_id: ProblemFamilyId::new("problem.test").unwrap(),
            parameter: "x".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "problem.test: parameter \"x\" has neither a fixed value nor both a min and max \
             bound, so it cannot be sampled"
        );
    }

    #[test]
    fn constraints_unsatisfiable_displays_family_and_attempt_count() {
        let error = GenerationError::ConstraintsUnsatisfiable {
            family_id: ProblemFamilyId::new("problem.test").unwrap(),
            attempts: 1000,
        };
        assert_eq!(
            error.to_string(),
            "problem.test: no combination of sampled parameters satisfied every declared \
             constraint after 1000 attempts"
        );
    }
}
```

- [ ] **Step 2: Wire it and run the tests**

In `src-tauri/src/generation/mod.rs`:

```rust
mod error;
mod rng;

pub use error::GenerationError;
```

Run: `cd src-tauri && cargo test --locked generation::error -- --nocapture`

Expected: all 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/generation/error.rs src-tauri/src/generation/mod.rs
git commit -m "feat(generation): add GenerationError"
```

---

### Task 4: Parameter resolution and constraint-checked resampling

**Files:**
- Create: `src-tauri/src/generation/sampling.rs`
- Modify: `src-tauri/src/generation/mod.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`

**Interfaces:**
- Consumes: `DeterministicRng` (Task 1), `GenerationError` (Task 3), `ConstraintExpr::holds`
  (Task 2), `ProblemFamily`/`ParameterSpec`/`ParameterType`/`Bound` (`crate::knowledge`,
  already exported).
- Produces: `pub(crate) fn resolve_parameters(family: &ProblemFamily, rng: &mut
  DeterministicRng) -> Result<BTreeMap<String, f64>, GenerationError>`. Task 6 consumes this
  exact signature. Also produces `crate::knowledge::parse_constraint` as a newly-exported
  path (see Step 0) — this task's tests are the first caller from outside `knowledge/`.

- [ ] **Step 0: Export `parse_constraint` from `knowledge/`**

`src-tauri/src/knowledge/constraint.rs`'s `parse_constraint` is `pub(crate)` (crate-wide
visible) but the `constraint` module itself is declared as plain `mod constraint;` in
`src-tauri/src/knowledge/mod.rs` — private to `knowledge/`, so `crate::knowledge::
constraint::parse_constraint` will not compile from a sibling module like `generation/`
(the item's own visibility doesn't help if the module path segment isn't reachable). Fix by
re-exporting the function itself, not the module. In `src-tauri/src/knowledge/mod.rs`, change:

```rust
pub use constraint::{ArithOp, CompareOp, ConstraintExpr, Term};
```

to:

```rust
pub(crate) use constraint::parse_constraint;
pub use constraint::{ArithOp, CompareOp, ConstraintExpr, Term};
```

This task's tests below call it as `crate::knowledge::parse_constraint`.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/generation/sampling.rs`:

```rust
use std::collections::BTreeMap;

use crate::knowledge::{Bound, ParameterSpec, ParameterType, ProblemFamily};

use super::error::GenerationError;
use super::rng::DeterministicRng;

const MAX_RESAMPLE_ATTEMPTS: u32 = 1000;

pub(crate) fn resolve_parameters(
    family: &ProblemFamily,
    rng: &mut DeterministicRng,
) -> Result<BTreeMap<String, f64>, GenerationError> {
    for _ in 0..MAX_RESAMPLE_ATTEMPTS {
        let mut resolved = BTreeMap::new();
        for name in family.parameters.keys() {
            resolve_parameter(name, &family.parameters, &mut resolved, rng, family)?;
        }
        if family.constraints.iter().all(|c| c.holds(&resolved)) {
            return Ok(resolved);
        }
    }
    Err(GenerationError::ConstraintsUnsatisfiable {
        family_id: family.id.clone(),
        attempts: MAX_RESAMPLE_ATTEMPTS,
    })
}

fn resolve_parameter(
    name: &str,
    parameters: &BTreeMap<String, ParameterSpec>,
    resolved: &mut BTreeMap<String, f64>,
    rng: &mut DeterministicRng,
    family: &ProblemFamily,
) -> Result<f64, GenerationError> {
    if let Some(value) = resolved.get(name) {
        return Ok(*value);
    }
    let spec = &parameters[name];
    let value = if let Some(bound) = &spec.value {
        resolve_bound(bound, parameters, resolved, rng, family)?
    } else {
        match (&spec.min, &spec.max) {
            (Some(min), Some(max)) => {
                let min = resolve_bound(min, parameters, resolved, rng, family)?;
                let max = resolve_bound(max, parameters, resolved, rng, family)?;
                match spec.kind {
                    ParameterType::Integer => {
                        rng.sample_integer(min.round() as i64, max.round() as i64) as f64
                    }
                    ParameterType::Float => rng.sample_float(min, max),
                }
            }
            _ => {
                return Err(GenerationError::UnderspecifiedParameter {
                    family_id: family.id.clone(),
                    parameter: name.to_owned(),
                })
            }
        }
    };
    resolved.insert(name.to_owned(), value);
    Ok(value)
}

fn resolve_bound(
    bound: &Bound,
    parameters: &BTreeMap<String, ParameterSpec>,
    resolved: &mut BTreeMap<String, f64>,
    rng: &mut DeterministicRng,
    family: &ProblemFamily,
) -> Result<f64, GenerationError> {
    match bound {
        Bound::Literal(value) => Ok(*value),
        Bound::Reference { parameter, offset } => {
            Ok(resolve_parameter(parameter, parameters, resolved, rng, family)? + offset)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{
        CanonicalSolution, ConceptId, ConstraintExpr, DifficultyRange, GeneratorId,
        GeneratorRef, ObjectiveId, ProblemFamilyId, ProblemFamilyStatus, ProvenanceKind,
        ProvenanceRef, ResponseType, SourceId,
    };

    fn minimal_family(
        parameters: BTreeMap<String, ParameterSpec>,
        constraints: Vec<ConstraintExpr>,
    ) -> ProblemFamily {
        ProblemFamily {
            id: ProblemFamilyId::new("problem.test_fixture").unwrap(),
            concept_id: ConceptId::new("concept.test").unwrap(),
            objective_ids: vec![ObjectiveId::new("objective.test").unwrap()],
            difficulty: DifficultyRange { min: 1, max: 1 },
            generator: GeneratorRef {
                id: GeneratorId::new("gen.test").unwrap(),
                version: 1,
            },
            parameters,
            constraints,
            prompt: "Prompt.".to_owned(),
            solution_structure: "Solution.".to_owned(),
            response_type: ResponseType::Numeric,
            canonical_solution: CanonicalSolution::Numeric { value: 1.0 },
            hints: Vec::new(),
            provenance_refs: vec![ProvenanceRef {
                source_id: SourceId::new("src.test").unwrap(),
                locator: None,
                kind: ProvenanceKind::Direct,
            }],
            status: ProblemFamilyStatus::Verified,
        }
    }

    fn fixed(value: f64) -> ParameterSpec {
        ParameterSpec {
            kind: ParameterType::Integer,
            value: Some(Bound::Literal(value)),
            min: None,
            max: None,
            description: None,
        }
    }

    fn bounded(kind: ParameterType, min: Bound, max: Bound) -> ParameterSpec {
        ParameterSpec {
            kind,
            value: None,
            min: Some(min),
            max: Some(max),
            description: None,
        }
    }

    #[test]
    fn fixed_value_parameter_resolves_without_consuming_rng_state() {
        let mut parameters = BTreeMap::new();
        parameters.insert("a".to_owned(), fixed(7.0));
        let family = minimal_family(parameters, Vec::new());

        let mut rng_before = DeterministicRng::new(1);
        let baseline = rng_before.next_u64();

        let mut rng = DeterministicRng::new(1);
        let resolved = resolve_parameters(&family, &mut rng).unwrap();
        assert_eq!(resolved["a"], 7.0);
        assert_eq!(rng.next_u64(), baseline, "fixed value must not draw from the RNG");
    }

    #[test]
    fn reference_chain_resolves_in_dependency_order() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "coeff".to_owned(),
            bounded(ParameterType::Integer, Bound::Literal(2.0), Bound::Literal(2.0)),
        );
        parameters.insert(
            "b".to_owned(),
            bounded(
                ParameterType::Integer,
                Bound::Literal(0.0),
                Bound::Reference {
                    parameter: "coeff".to_owned(),
                    offset: 0.0,
                },
            ),
        );
        parameters.insert(
            "c".to_owned(),
            bounded(
                ParameterType::Integer,
                Bound::Reference {
                    parameter: "b".to_owned(),
                    offset: 1.0,
                },
                Bound::Reference {
                    parameter: "b".to_owned(),
                    offset: 1.0,
                },
            ),
        );
        let family = minimal_family(parameters, Vec::new());

        let mut rng = DeterministicRng::new(1);
        let resolved = resolve_parameters(&family, &mut rng).unwrap();
        assert_eq!(resolved["coeff"], 2.0);
        assert_eq!(resolved["c"], resolved["b"] + 1.0);
    }

    #[test]
    fn underspecified_parameter_is_rejected() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "x".to_owned(),
            ParameterSpec {
                kind: ParameterType::Integer,
                value: None,
                min: None,
                max: None,
                description: None,
            },
        );
        let family = minimal_family(parameters, Vec::new());

        let mut rng = DeterministicRng::new(1);
        assert!(matches!(
            resolve_parameters(&family, &mut rng),
            Err(GenerationError::UnderspecifiedParameter { parameter, .. }) if parameter == "x"
        ));
    }

    #[test]
    fn unsatisfiable_constraint_fails_after_exactly_max_attempts() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "x".to_owned(),
            bounded(ParameterType::Integer, Bound::Literal(1.0), Bound::Literal(1.0)),
        );
        let constraint = crate::knowledge::parse_constraint("test", "x > 1").unwrap();
        let family = minimal_family(parameters, vec![constraint]);

        let mut rng = DeterministicRng::new(1);
        assert!(matches!(
            resolve_parameters(&family, &mut rng),
            Err(GenerationError::ConstraintsUnsatisfiable { attempts, .. }) if attempts == MAX_RESAMPLE_ATTEMPTS
        ));
    }

    #[test]
    fn satisfiable_narrow_constraint_eventually_succeeds() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "x".to_owned(),
            bounded(ParameterType::Integer, Bound::Literal(1.0), Bound::Literal(2.0)),
        );
        let constraint = crate::knowledge::parse_constraint("test", "x == 1").unwrap();
        let family = minimal_family(parameters, vec![constraint]);

        let mut rng = DeterministicRng::new(1);
        let resolved = resolve_parameters(&family, &mut rng).unwrap();
        assert_eq!(resolved["x"], 1.0);
    }
}
```

- [ ] **Step 2: Wire the module and run the tests**

In `src-tauri/src/generation/mod.rs`:

```rust
mod error;
mod rng;
mod sampling;

pub use error::GenerationError;
```

Run: `cd src-tauri && cargo test --locked generation::sampling -- --nocapture`

Expected: all 5 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/generation/sampling.rs src-tauri/src/generation/mod.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(generation): parameter resolution with constraint-checked resampling"
```

---

### Task 5: Template substitution

**Files:**
- Create: `src-tauri/src/generation/template.rs`
- Modify: `src-tauri/src/generation/mod.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks (pure string functions).
- Produces: `pub(crate) fn substitute_braces(template: &str, resolved_parameters:
  &BTreeMap<String, f64>) -> String`, `pub(crate) fn substitute_identifiers(template: &str,
  resolved_parameters: &BTreeMap<String, f64>) -> String`. Task 6 consumes both exact
  signatures.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/generation/template.rs`:

```rust
use std::collections::BTreeMap;

pub(crate) fn substitute_braces(template: &str, resolved_parameters: &BTreeMap<String, f64>) -> String {
    let mut result = template.to_owned();
    for (name, value) in resolved_parameters {
        result = result.replace(&format!("{{{name}}}"), &format_number(*value));
    }
    result
}

pub(crate) fn substitute_identifiers(
    template: &str,
    resolved_parameters: &BTreeMap<String, f64>,
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() || c == '_' {
            let mut identifier = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    identifier.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            match resolved_parameters.get(&identifier) {
                Some(value) => result.push_str(&format_number(*value)),
                None => result.push_str(&identifier),
            }
        } else {
            result.push(c);
            chars.next();
        }
    }
    result
}

fn format_number(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), *value))
            .collect()
    }

    #[test]
    fn substitute_braces_replaces_every_placeholder() {
        let result = substitute_braces(
            "f(x) = {coeff}x - x^2 over [{a}, {b}]",
            &params(&[("coeff", 4.0), ("a", 0.0), ("b", 3.0)]),
        );
        assert_eq!(result, "f(x) = 4x - x^2 over [0, 3]");
    }

    #[test]
    fn substitute_braces_leaves_text_with_no_matching_placeholder_untouched() {
        let result = substitute_braces("no placeholders here", &params(&[("coeff", 4.0)]));
        assert_eq!(result, "no placeholders here");
    }

    #[test]
    fn substitute_identifiers_replaces_exact_parameter_names_only() {
        let result = substitute_identifiers(
            "2*pi*(coeff*b^3/3 - b^4/4)",
            &params(&[("coeff", 4.0), ("b", 3.0)]),
        );
        assert_eq!(result, "2*pi*(4*3^3/3 - 3^4/4)");
    }

    #[test]
    fn substitute_identifiers_leaves_non_parameter_identifiers_untouched() {
        let result = substitute_identifiers("sin(pi/2)", &params(&[("b", 3.0)]));
        assert_eq!(result, "sin(pi/2)");
    }

    #[test]
    fn substitute_identifiers_does_not_partially_match_a_longer_name() {
        let result = substitute_identifiers("coefficient + coeff", &params(&[("coeff", 2.0)]));
        assert_eq!(result, "coefficient + 2");
    }

    #[test]
    fn format_number_omits_trailing_zero_for_whole_numbers() {
        assert_eq!(format_number(4.0), "4");
        assert_eq!(format_number(-2.0), "-2");
        assert_eq!(format_number(2.5), "2.5");
    }
}
```

- [ ] **Step 2: Wire the module and run the tests**

In `src-tauri/src/generation/mod.rs`:

```rust
mod error;
mod rng;
mod sampling;
mod template;

pub use error::GenerationError;
```

Run: `cd src-tauri && cargo test --locked generation::template -- --nocapture`

Expected: all 6 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/generation/template.rs src-tauri/src/generation/mod.rs
git commit -m "feat(generation): template substitution for prompts and expressions"
```

---

### Task 6: `generate_problem_instance`

**Files:**
- Modify: `src-tauri/src/generation/mod.rs`

**Interfaces:**
- Consumes: `resolve_parameters` (Task 4), `substitute_braces`/`substitute_identifiers`
  (Task 5), `GenerationError` (Task 3); `ProblemFamily`, `ProblemInstance`, `CanonicalSolution`,
  `ResolvedSolution` (`crate::knowledge`, already exported).
- Produces: `pub fn generate_problem_instance(family: &ProblemFamily, seed: u64) ->
  Result<ProblemInstance, GenerationError>` — the module's one public entry point.

- [ ] **Step 1: Write the failing test**

Replace the contents of `src-tauri/src/generation/mod.rs` with:

```rust
mod error;
mod rng;
mod sampling;
mod template;

pub use error::GenerationError;

use crate::knowledge::{CanonicalSolution, ProblemFamily, ProblemInstance, ResolvedSolution};

use rng::DeterministicRng;

pub fn generate_problem_instance(
    family: &ProblemFamily,
    seed: u64,
) -> Result<ProblemInstance, GenerationError> {
    match family.generator.id.as_str() {
        "gen.shell_y_poly" => generate_generic(family, seed),
        _ => Err(GenerationError::UnknownGenerator {
            id: family.generator.id.clone(),
        }),
    }
}

fn generate_generic(
    family: &ProblemFamily,
    seed: u64,
) -> Result<ProblemInstance, GenerationError> {
    let mut rng = DeterministicRng::new(seed);
    let resolved_parameters = sampling::resolve_parameters(family, &mut rng)?;

    let prompt = template::substitute_braces(&family.prompt, &resolved_parameters);
    let hints = family
        .hints
        .iter()
        .map(|hint| template::substitute_braces(&hint.text, &resolved_parameters))
        .collect();

    let canonical_solution = match &family.canonical_solution {
        CanonicalSolution::Numeric { value } => ResolvedSolution::Numeric(*value),
        CanonicalSolution::Symbolic { expression } => ResolvedSolution::Symbolic(
            template::substitute_identifiers(expression, &resolved_parameters),
        ),
    };

    Ok(ProblemInstance {
        family_id: family.id.clone(),
        seed,
        resolved_parameters,
        prompt,
        canonical_solution,
        hints,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::knowledge::load_knowledge_package;

    fn shell_y_poly_family() -> ProblemFamily {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/knowledge/tests/fixtures/canonical");
        let package = load_knowledge_package(&fixture_root).unwrap();
        package
            .problem_families
            .into_iter()
            .find(|family| family.id.as_str() == "problem.shell_y_poly")
            .expect("fixture must contain problem.shell_y_poly")
    }

    #[test]
    fn unknown_generator_id_is_rejected() {
        use crate::knowledge::GeneratorRef;
        let mut family = shell_y_poly_family();
        family.generator = GeneratorRef {
            id: crate::knowledge::GeneratorId::new("gen.nonexistent").unwrap(),
            version: 1,
        };
        assert!(matches!(
            generate_problem_instance(&family, 1),
            Err(GenerationError::UnknownGenerator { .. })
        ));
    }

    #[test]
    fn same_seed_produces_byte_identical_instances() {
        let family = shell_y_poly_family();
        let first = generate_problem_instance(&family, 42).unwrap();
        let second = generate_problem_instance(&family, 42).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn seed_actually_affects_the_sampled_parameters() {
        // coeff has only 5 possible values (2..=6) and b even fewer, so any single pair of
        // seeds has a real (if small) chance of coincidentally landing on the same
        // combination — this checks across 100 seeds that generation isn't silently
        // ignoring the seed, without depending on any one pairwise comparison.
        let family = shell_y_poly_family();
        let distinct_outcomes: std::collections::BTreeSet<Vec<(String, i64)>> = (0..100u64)
            .map(|seed| {
                generate_problem_instance(&family, seed)
                    .unwrap()
                    .resolved_parameters
                    .into_iter()
                    .map(|(name, value)| (name, value as i64))
                    .collect()
            })
            .collect();
        assert!(
            distinct_outcomes.len() > 1,
            "100 different seeds all produced the same resolved parameters"
        );
    }

    #[test]
    fn produced_instance_has_no_unsubstituted_placeholders_or_bare_parameter_names() {
        let family = shell_y_poly_family();
        let instance = generate_problem_instance(&family, 7).unwrap();

        assert!(!instance.prompt.contains('{'));
        for hint in &instance.hints {
            assert!(!hint.contains('{'));
        }

        let ResolvedSolution::Symbolic(expression) = &instance.canonical_solution else {
            panic!("problem.shell_y_poly is a SymbolicExpression family");
        };
        for name in ["coeff", "a", "b"] {
            assert!(
                !expression
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .any(|token| token == name),
                "expression {expression:?} still contains bare parameter {name:?}"
            );
        }
    }

    #[test]
    fn canonical_solution_expression_is_parseable_by_mathcore() {
        let family = shell_y_poly_family();
        let instance = generate_problem_instance(&family, 7).unwrap();
        let ResolvedSolution::Symbolic(expression) = &instance.canonical_solution else {
            panic!("problem.shell_y_poly is a SymbolicExpression family");
        };
        mathcore::MathCore::new()
            .calculate(expression)
            .unwrap_or_else(|error| panic!("{expression:?} did not parse: {error}"));
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test --locked generation:: -- --nocapture`

Expected: every test across `generation::rng`, `generation::sampling`, `generation::template`,
and `generation` (the 5 tests just added) passes.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/generation/mod.rs
git commit -m "feat(generation): generate_problem_instance entry point"
```

---

### Task 7: Domain-validity property test

**Files:**
- Create: `src-tauri/src/generation/tests/mod.rs`
- Modify: `src-tauri/src/generation/mod.rs`

**Interfaces:**
- Consumes: `generate_problem_instance` (Task 6).
- Produces: nothing further tasks depend on — this is the deliverable ROADMAP.md names for
  this sub-project.

- [ ] **Step 1: Write the property test**

Create `src-tauri/src/generation/tests/mod.rs`:

```rust
use std::path::Path;

use crate::generation::generate_problem_instance;
use crate::knowledge::load_knowledge_package;

fn shell_y_poly_family() -> crate::knowledge::ProblemFamily {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical");
    let package = load_knowledge_package(&fixture_root).unwrap();
    package
        .problem_families
        .into_iter()
        .find(|family| family.id.as_str() == "problem.shell_y_poly")
        .expect("fixture must contain problem.shell_y_poly")
}

/// `problem.shell_y_poly` describes the region bounded above by
/// `f(x) = coeff*x - x^2` and below by the x-axis over `[a, b]` — geometrically valid only if
/// `f(x) >= 0` across the whole sampled interval. The schema (task 054) deliberately does not
/// formalize this as a declarative constraint; this test proves the generic sampling engine's
/// actual declared bounds (`b`'s max referencing `coeff`) enforce it anyway, across many seeds.
#[test]
fn shell_y_poly_height_stays_non_negative_across_ten_thousand_seeds() {
    let family = shell_y_poly_family();

    for seed in 0..10_000u64 {
        let instance = generate_problem_instance(&family, seed)
            .unwrap_or_else(|error| panic!("seed {seed} failed to generate: {error}"));

        let coeff = instance.resolved_parameters["coeff"];
        let b = instance.resolved_parameters["b"];

        const SAMPLE_POINTS: u32 = 50;
        for i in 0..=SAMPLE_POINTS {
            let x = b * (i as f64 / SAMPLE_POINTS as f64);
            let height = coeff * x - x * x;
            assert!(
                height >= 0.0,
                "seed {seed}: h({x}) = {height} < 0 for coeff={coeff}, b={b}"
            );
        }
    }
}
```

- [ ] **Step 2: Wire the test module and run it**

In `src-tauri/src/generation/mod.rs`, add at the end:

```rust
#[cfg(test)]
mod tests;
```

Run: `cd src-tauri && cargo test --locked generation::tests -- --nocapture`

Expected: `shell_y_poly_height_stays_non_negative_across_ten_thousand_seeds` passes. This test
runs 10,000 generations × 51 sample points — if it's slow enough to be annoying in the full
suite, that's expected and acceptable (it's the one deliberately heavy test in this plan); do
not reduce the seed count to make it faster.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/generation/mod.rs src-tauri/src/generation/tests
git commit -m "test(generation): domain-validity property test for gen.shell_y_poly"
```

---

### Task 8: Full validation pass

**Files:** none (verification only).

**Interfaces:** none.

- [ ] **Step 1: Run the full test suite**

Run: `cd src-tauri && cargo test --locked --quiet`

Expected: every existing test still passes, plus every test added in Tasks 1–7 (4 + 4 + 3 +
5 + 6 + 5 + 1 = 28 new tests — count them and confirm the total matches before proceeding).

- [ ] **Step 2: Run clippy**

Run: `cd src-tauri && cargo clippy --all-targets --locked --quiet -- -D warnings`

Expected: no warnings. Fix any that appear (do not `#[allow]` them away without a specific
reason recorded in the task's worklog).

- [ ] **Step 3: Run fmt and the whitespace check**

Run: `cd src-tauri && cargo fmt --all --check`
Run: `cd /home/marcus/axiom && git diff --check`

Expected: both clean.

- [ ] **Step 4: Update the task file and commit**

Update `.ai/tasks/056-problem-generation.md`: move `status` to `review`, fill in "What was
built / tested / left out" with the actual validation command output (test count,
clippy/fmt results), and add a `## Review` heading placeholder for the next reviewer,
following the exact shape `.ai/tasks/_archive/055-math-verify.md` used.

```bash
git add .ai/tasks/056-problem-generation.md
git commit -m "docs(056): mark problem generation implementation complete, ready for review"
```
