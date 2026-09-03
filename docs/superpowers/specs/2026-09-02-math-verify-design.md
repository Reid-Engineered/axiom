# `math.verify` capability — design

## 1. Scope

Roadmap Stage 8, sub-project 2 of 6 in the "Practice engine" initiative (`ROADMAP.md`
"Remaining Stage 8 scope"), following directly on sub-project 1 (canonical Problem schema,
`.ai/tasks/_archive/054-canonical-problem-schema.md`, merged) and the module-capability
runtime (`.ai/tasks/_archive/045-048`, done).

Add a first-party `math.verify` capability, version 1: given a `ProblemInstance`'s
`response_type` and `canonical_solution` (`src-tauri/src/knowledge/types.rs`) plus a
student's response, decide whether the response is correct. This is the **first concrete
consumer** of the module-capability runtime built in sub-project 1
(`docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md`) — no real
`CapabilityProvider` exists yet, only the generic mechanism and test fixtures. This design
also sets the pattern later first-party capabilities will follow.

**Does not build:** partial credit, error classification, or any other diagnostic reasoning
beyond a correctness boolean (that's Practice's concern — see §2); the generator functions
that produce `ProblemInstance` values (sub-project 3); the Practice Core Utility that will
call this capability (sub-project 4); any UI.

## 2. Decisions carried in from brainstorming

- **CAS dependency: `mathcore` (crates.io, MIT), used narrowly.** Symbolica was ruled out —
  its free tiers require either an email-registered annual key or a 1-core/device cap, and
  any commercial use needs a paid license; unacceptable for a "no accounts, works offline"
  desktop app with an unresolved commercial future. Two open-source Rust CAS alternatives
  were checked and rejected: `Symmetrica` is archived/dead (4 stars, read-only since March
  2026); the fully-open `cas-rs` is actively maintained (380 commits) but explicitly
  self-described as "very early stage." `mathcore` (127 stars, MIT, plain-copyright license
  with no field-of-use restriction) was chosen deliberately, accepting that it has had no
  commits since its initial release and unaddressed issues/PRs — a known risk, mitigated by
  using only its most basic, best-exercised surface (see below), never its differentiation/
  integration/solve/matrix code, and by this design's test suite exercising it against every
  real equivalence case the reference Knowledge Package actually needs.
- **No free variables ever reach this capability.** `ProblemInstance.resolved_parameters` is
  always a fully-resolved `BTreeMap<String, f64>` and `ResolvedSolution` is always
  closed-form (`types.rs`, sub-project 1) — the generator (sub-project 3, not built yet) is
  responsible for substituting every parameter before producing an instance. This means both
  response types reduce to "evaluate to a real number, compare within tolerance" — no
  symbolic simplification, no domain sampling over free variables, is needed for v1.
  Formula-shaped answers ("express your answer in terms of `r`") are out of scope; see §8.
- **No symbolic-exactness enforcement in v1.** `response_type: SymbolicExpression` exists to
  let a problem require a symbolic-looking answer, but this capability does not try to
  detect and reject a numerically-equivalent decimal approximation (e.g. accepting `6.283`
  where `2*pi` was "intended"). Any response that evaluates within tolerance passes,
  regardless of surface form. If this turns out to matter pedagogically, a cheap follow-up
  heuristic (reject responses with no non-numeric token) is documented in §8, not built now.
- **One provider, not two.** `ROADMAP.md` describes "deterministic + Symbolica-CAS
  providers" (written before this brainstorm); this design implements that as one
  `CapabilityProvider` that branches internally on `response_type`, not two separately
  registered providers. There is no reason today for a workspace to swap one half out
  independently, and the module-capability runtime's `resolve()` picks providers per
  capability, not per response type — splitting now would be speculative.
- **Verdict only, no diagnostics.** Per the existing Practice/`math.verify` boundary
  (`docs/superpowers/specs/2026-08-30-knowledge-package-v1-design.md` §5: `math.verify`
  answers "is this answer equivalent?"; Practice answers "what do we give this learner
  now?"), the output is a correctness boolean plus an optional parse-failure reason — never
  partial credit, error classification, or hint selection.

## 3. Location and registration

New top-level module `src-tauri/src/capabilities/math_verify/`, sibling to `knowledge/` and
`modules/` — the first of what should become a `capabilities/` home for first-party
capability providers (a later `knowledge.query` capability would live alongside it, not
inside `knowledge/` itself). Follows the file-split convention `knowledge/` already
established: `types.rs` (request/response shapes), `error.rs` (`MathVerifyError`),
`provider.rs` (the `CapabilityProvider` impl), `mod.rs` (public exports), inline
`#[cfg(test)] mod tests` per file plus a `tests/` directory for the full-registry
round-trip test (see §7).

Ships one first-party manifest, `module.toml`, embedded via `include_str!` at compile time
— the pattern `docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md` §
"Manifest source" already established for first-party modules
(`modules::EmbeddedManifestSource`). The manifest declares exactly one capability:

```toml
manifest_version = 1
id = "core.math_verify"
name = "Core Math Verify"
version = "1.0.0"
minimum_axiom_version = "0.1.0"
offline = "full"

[[provides]]
id = "math.verify"
version = 1
```

Note the module id uses `_` not `-` — `src-tauri/src/modules/identifier.rs`'s grammar
(dot-separated segments, ASCII lowercase/digit/underscore only) rejects hyphens.

(Exact manifest field names/grammar follow `src-tauri/src/modules/manifest.rs` as it exists
today — this design does not change that grammar.)

## 4. Request/response contract

Transported as `serde_json::Value` through the existing generic
`CapabilityProvider::invoke(capability_id, version, input) -> Result<Value, InvocationError>`
(`src-tauri/src/modules/registry.rs`) — no change to that trait.

```rust
#[derive(Deserialize)]
#[serde(tag = "response_type", rename_all = "kebab-case")]
pub enum VerifyRequest {
    Numeric {
        canonical_solution: f64,
        student_response: f64,
    },
    SymbolicExpression {
        canonical_solution: String, // same expression grammar as CanonicalSolution::Symbolic
        student_response: String,
    },
}

#[derive(Serialize)]
pub struct VerifyResult {
    pub is_correct: bool,
    /// Set only when `student_response` could not be parsed/evaluated at all (SymbolicExpression
    /// only). A response that parses but is simply wrong leaves this `None`.
    pub error: Option<String>,
}
```

The tagged-enum request shape means a malformed pairing (e.g. `response_type: "numeric"`
with a string `student_response`) is a plain deserialization failure, mapped to
`InvocationError::InvalidInput` by the registry's existing `invoke()` — this capability's
provider code never has to defensively re-check that pairing itself.

## 5. Algorithm

**Numeric.** `|student_response - canonical_solution| <= tolerance`, where
`tolerance = ABSOLUTE_EPSILON.max(RELATIVE_EPSILON * canonical_solution.abs())`,
`ABSOLUTE_EPSILON = 1e-9`, `RELATIVE_EPSILON = 1e-9`. A non-finite `student_response`
(`NaN`/`Infinity`) is simply incorrect (`is_correct: false`), not an error — a learner can
type garbage. A non-finite `canonical_solution` is impossible by construction (sub-project 1
already rejects it at authoring time via `NonFiniteCanonicalSolution`) but is not
re-validated here; if it somehow occurred, the comparison would just always fail.

**SymbolicExpression.** Parse and evaluate `canonical_solution` and `student_response` with
`MathCore::new().calculate(&str) -> Result<f64, MathError>` (the same call for both — no free
variables, so no `evaluate_with_vars` needed), each producing an `f64` directly (`calculate`
internally parses and rejects non-real results). `differentiate`/`integrate`/`solve`/
`simplify`/matrix/precision APIs are never called from this capability. If
`student_response` fails to parse or evaluate, return
`VerifyResult { is_correct: false, error: Some(<mathcore's error message>) }`. If
`canonical_solution` fails to parse (should be impossible — sub-project 1 doesn't validate
that its `expression` string is `mathcore`-parseable, a gap noted in §8), treat it as an
internal `InvocationError::Failed`, since that indicates a broken Knowledge Package, not a
wrong student answer. Otherwise apply the same tolerance comparison as the numeric case.

## 6. Error handling

- Malformed request JSON (wrong shape, unknown `response_type` tag) →
  `InvocationError::InvalidInput` (registry's existing generic handling — see §4).
- Unparseable canonical expression (authoring bug, not student error) →
  `InvocationError::Failed`.
- Unparseable student expression, non-finite student number, or a parseable-but-wrong
  answer → `Ok(VerifyResult { is_correct: false, .. })`, never an error. Wrongness is not
  failure.
- New `MathVerifyError` enum (mirrors `KnowledgeError`'s shape/`Display` convention) exists
  only for the internal `InvocationError::Failed` case above; it is not exposed in the JSON
  contract.

## 7. Testing

- Numeric: exact match, within tolerance, just outside tolerance (both directions), zero,
  negative values, non-finite student input.
- SymbolicExpression: several equivalent forms of the same value (`2*pi`, `tau`,
  `2*3.14159265358979`), a wrong-but-parseable answer, an unparseable answer (confirms
  `error` is set and `is_correct` is `false`), a case exercising each `mathcore` function
  this capability relies on (`sin`, `cos`, `tan`, `exp`, `ln`, `sqrt`) at least once against
  a hand-computed expected value — this is the direct mitigation for `mathcore`'s
  maintenance risk (§2): every function path this code depends on gets a real, checked
  example, not just typical happy-path arithmetic.
- One full-stack test: embed the manifest, register `MathVerifyProvider` into a real
  `ModuleRegistry` (`src-tauri/src/modules/registry.rs`), resolve `math.verify` v1 through
  `ModuleRegistry::resolve`, and `invoke` it — proving the whole path this design adds to
  the module-capability runtime actually wires together, not just the provider in isolation.

## 8. Follow-ups (out of scope here, tracked for later)

- Formula-shaped symbolic answers with genuine free variables (domain-sampling
  equivalence checking) — not needed until a problem family actually requires one.
- Symbolic-exactness enforcement (rejecting a numerically-correct decimal approximation for
  a `SymbolicExpression` problem) if it turns out to matter pedagogically — a cheap
  heuristic (reject a response with no non-numeric-literal token) rather than real
  structural equivalence.
- Validating at Knowledge-authoring time (sub-project 1's `problem_family.rs`) that
  `CanonicalSolution::Symbolic`'s `expression` string is actually `mathcore`-parseable,
  instead of only discovering a broken authored expression at verification time.
- Sub-project 3 (generator functions), sub-project 4 (Practice Core Utility, the actual
  caller of this capability), sub-projects 5–6 (Study Session UI, offline acceptance test).
