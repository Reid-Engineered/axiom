# math.verify Capability Implementation Plan

> **For agentic workers:** this plan is handed to Codex to implement directly against `.ai/tasks/`, `AGENTS.md`, and `CLAUDE.md`'s normal workflow — not executed via `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Steps use checkbox (`- [ ]`) syntax for tracking regardless of executor.

**Goal:** Add a first-party `math.verify` v1 capability under `src-tauri/src/capabilities/math_verify/` that decides whether a student's response to a `ProblemInstance` matches its canonical solution — the first real `CapabilityProvider` built against the module-capability runtime.

**Architecture:** New sibling module to `knowledge/` and `modules/`, following the same raw-types/error/tests file-split convention. One `CapabilityProvider` impl branches on `response_type`: `Numeric` is plain float comparison (no dependency); `SymbolicExpression` parses and evaluates both the canonical and student expression strings with the new `mathcore` crate's `MathCore::calculate`, then applies the same tolerance comparison. The manifest ships embedded via `include_str!`, matching the pattern `modules::EmbeddedManifestSource` already established for first-party modules — but this plan does not wire it into `modules::EmbeddedManifestSource::default()` or any Tauri command/app-startup path; that's app-layer work, out of scope here (the same boundary sub-project 1 of the module-capability runtime and task 054 both respected).

**Tech Stack:** Rust, `serde`/`serde_json` (already dependencies), one new crate: `mathcore = "=0.3.1"` (MIT), `default-features = false` (the published crate has no `std` feature; disabling defaults skips `mathcore`'s `parallel`/`fft` features — `rayon`/`rustfft` — since only `MathCore::calculate`'s parse+evaluate path is used; no threading or FFT needed).

**Spec:** `docs/superpowers/specs/2026-09-02-math-verify-design.md`

## Global Constraints

- New code lives in `src-tauri/src/capabilities/math_verify/` only, plus one new top-level
  `pub mod capabilities;` line in `src-tauri/src/lib.rs` and one new dependency line in
  `src-tauri/Cargo.toml`. No Tauri command, no frontend change, no change to
  `src-tauri/src/modules/` or `src-tauri/src/knowledge/`.
- This capability's job is a correctness verdict only — never partial credit, error
  classification, or hint selection (spec §2, §6).
- No free variables ever reach this capability — both `canonical_solution` and
  `student_response` are always closed-form (spec §2). Do not add domain-sampling or
  `evaluate_with_vars` machinery.
- Tolerance: `ABSOLUTE_EPSILON = 1e-9`, `RELATIVE_EPSILON = 1e-9`,
  `tolerance = ABSOLUTE_EPSILON.max(RELATIVE_EPSILON * canonical.abs())` (spec §5). Use these
  exact constants in every task that needs tolerance.
- `mathcore` is used **only** through `MathCore::new()` and `.calculate(&str) -> Result<f64,
  mathcore::MathError>`. Never call its `differentiate`/`integrate`/`solve`/`simplify`/matrix/
  precision APIs from this capability (spec §2, §5) — that unused surface is exactly the part
  of `mathcore` this design deliberately avoids depending on.
- Follow `knowledge/`'s established conventions: `pub(crate)` on internal helpers, a single
  flat error enum with a hand-written `Display` (no `thiserror`), inline `#[cfg(test)] mod
  tests` in the file the code under test lives in, a `tests/` subdirectory for the one
  cross-file integration test.
- Module id grammar (`src-tauri/src/modules/identifier.rs`): dot-separated segments, each
  starting with an ASCII lowercase letter, remaining characters lowercase/digit/underscore,
  minimum 2 segments — **no hyphens**. Use `core.math_verify` (module id) and `math.verify`
  (capability id); both are valid under this grammar.

---

### Task 1: Add the `mathcore` dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Produces: the `mathcore` crate available to the rest of this plan, specifically
  `mathcore::MathCore::new()` and `mathcore::MathCore::calculate(&self, expression: &str) ->
  Result<f64, mathcore::MathError>`.

- [ ] **Step 1: Add the dependency**

In `src-tauri/Cargo.toml`, in the `[dependencies]` section (after the existing `async-trait`
line, to keep the file's existing top-to-bottom exact-pin style), add:

```toml
mathcore = { version = "=0.3.1", default-features = false }
```

- [ ] **Step 2: Verify it resolves and builds**

Run: `cd src-tauri && cargo check --locked`

Expected: succeeds (this only adds the dependency to the lockfile/build graph; nothing
references it yet, so there should be no warnings about unused imports since no code changed).

If `cargo check --locked` fails because `Cargo.lock` needs updating, run
`cargo check` once (without `--locked`) to let Cargo update the lock file, confirm the diff
touches only `mathcore` and its transitive dependencies, then re-run with `--locked` to
confirm it's now satisfied.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore(math-verify): add mathcore dependency"
```

---

### Task 2: `VerifyRequest` / `VerifyResult` types

**Files:**
- Create: `src-tauri/src/capabilities/mod.rs`
- Create: `src-tauri/src/capabilities/math_verify/mod.rs`
- Create: `src-tauri/src/capabilities/math_verify/types.rs`

**Interfaces:**
- Produces: `VerifyRequest` (an internally-tagged enum, `Deserialize` only — it is only ever
  the input side of the capability), `VerifyResult` (`Serialize` only — output side). Every
  later task in this plan consumes these exact names, variants, and field names.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/capabilities/math_verify/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "response_type", rename_all = "kebab-case")]
pub enum VerifyRequest {
    SymbolicExpression {
        canonical_solution: String,
        student_response: String,
    },
    Numeric {
        canonical_solution: f64,
        student_response: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VerifyResult {
    pub is_correct: bool,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_request_deserializes_from_json() {
        let value = serde_json::json!({
            "response_type": "numeric",
            "canonical_solution": 4.0,
            "student_response": 4.0000001,
        });

        let request: VerifyRequest = serde_json::from_value(value).unwrap();

        assert_eq!(
            request,
            VerifyRequest::Numeric {
                canonical_solution: 4.0,
                student_response: 4.0000001,
            }
        );
    }

    #[test]
    fn symbolic_expression_request_deserializes_from_json() {
        let value = serde_json::json!({
            "response_type": "symbolic-expression",
            "canonical_solution": "2*pi",
            "student_response": "tau",
        });

        let request: VerifyRequest = serde_json::from_value(value).unwrap();

        assert_eq!(
            request,
            VerifyRequest::SymbolicExpression {
                canonical_solution: "2*pi".to_owned(),
                student_response: "tau".to_owned(),
            }
        );
    }

    #[test]
    fn unknown_response_type_is_rejected() {
        let value = serde_json::json!({
            "response_type": "vector",
            "canonical_solution": 1.0,
            "student_response": 1.0,
        });

        assert!(serde_json::from_value::<VerifyRequest>(value).is_err());
    }

    #[test]
    fn verify_result_serializes_with_null_error_when_absent() {
        let result = VerifyResult {
            is_correct: true,
            error: None,
        };

        let value = serde_json::to_value(&result).unwrap();

        assert_eq!(
            value,
            serde_json::json!({ "is_correct": true, "error": null })
        );
    }
}
```

Create `src-tauri/src/capabilities/math_verify/mod.rs`:

```rust
mod types;

pub use types::{VerifyRequest, VerifyResult};
```

Create `src-tauri/src/capabilities/mod.rs`:

```rust
pub mod math_verify;
```

- [ ] **Step 2: Wire the new top-level module and run the tests**

In `src-tauri/src/lib.rs`, add (alphabetical, before `pub mod commands;`):

```rust
pub mod capabilities;
```

Run: `cd src-tauri && cargo test --locked capabilities::math_verify::types -- --nocapture`

Expected: 4 tests pass (`numeric_request_deserializes_from_json`,
`symbolic_expression_request_deserializes_from_json`, `unknown_response_type_is_rejected`,
`verify_result_serializes_with_null_error_when_absent`).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/capabilities
git commit -m "feat(math-verify): add VerifyRequest/VerifyResult types"
```

---

### Task 3: `MathVerifyError`

**Files:**
- Create: `src-tauri/src/capabilities/math_verify/error.rs`
- Modify: `src-tauri/src/capabilities/math_verify/mod.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `MathVerifyError` — Task 5 constructs
  `MathVerifyError::UnparseableCanonicalSolution { expression, message }` when a Knowledge
  Package's authored `canonical_solution` expression fails to parse/evaluate (an authoring
  bug, not a student error — spec §5, §6).

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/capabilities/math_verify/error.rs`:

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MathVerifyError {
    UnparseableCanonicalSolution { expression: String, message: String },
}

impl fmt::Display for MathVerifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnparseableCanonicalSolution { expression, message } => write!(
                formatter,
                "canonical solution expression {expression:?} could not be evaluated: {message}"
            ),
        }
    }
}

impl Error for MathVerifyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unparseable_canonical_solution_displays_expression_and_message() {
        let error = MathVerifyError::UnparseableCanonicalSolution {
            expression: "2*+pi".to_owned(),
            message: "parse error".to_owned(),
        };

        assert_eq!(
            error.to_string(),
            "canonical solution expression \"2*+pi\" could not be evaluated: parse error"
        );
    }
}
```

- [ ] **Step 2: Run the test**

Run: `cd src-tauri && cargo test --locked capabilities::math_verify::error -- --nocapture`

Expected: `unparseable_canonical_solution_displays_expression_and_message` passes (this file
has no failing-first step beyond compiling — the type and its `Display` are written together
since there's no simpler intermediate state worth testing separately).

- [ ] **Step 3: Export it and commit**

In `src-tauri/src/capabilities/math_verify/mod.rs`, add `mod error;` and export it:

```rust
mod error;
mod types;

pub use error::MathVerifyError;
pub use types::{VerifyRequest, VerifyResult};
```

```bash
git add src-tauri/src/capabilities/math_verify/error.rs src-tauri/src/capabilities/math_verify/mod.rs
git commit -m "feat(math-verify): add MathVerifyError"
```

---

### Task 4: `MathVerifyProvider` — numeric path

**Files:**
- Create: `src-tauri/src/capabilities/math_verify/provider.rs`
- Modify: `src-tauri/src/capabilities/math_verify/mod.rs`

**Interfaces:**
- Consumes: `VerifyRequest`, `VerifyResult` (Task 2); `CapabilityProvider`, `CapabilityId`,
  `InvocationError` from `crate::modules` (`src-tauri/src/modules/mod.rs` — already exported).
- Produces: `MathVerifyProvider` (a unit struct), `within_tolerance(student: f64, canonical:
  f64) -> bool` (`pub(crate)`, reused unchanged by Task 5), the capability identity constants
  `CAPABILITY_ID: &str = "math.verify"` and `CAPABILITY_VERSION: u32 = 1` that Task 6's
  manifest must match exactly.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/capabilities/math_verify/provider.rs`:

```rust
use serde_json::Value;

use crate::modules::{CapabilityId, CapabilityProvider, InvocationError};

use super::types::{VerifyRequest, VerifyResult};

pub(crate) const CAPABILITY_ID: &str = "math.verify";
pub(crate) const CAPABILITY_VERSION: u32 = 1;
const ABSOLUTE_EPSILON: f64 = 1e-9;
const RELATIVE_EPSILON: f64 = 1e-9;

pub struct MathVerifyProvider;

#[async_trait::async_trait]
impl CapabilityProvider for MathVerifyProvider {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        input: Value,
    ) -> Result<Value, InvocationError> {
        if capability_id.as_str() != CAPABILITY_ID || version != CAPABILITY_VERSION {
            return Err(InvocationError::UnknownCapability {
                capability_id: capability_id.clone(),
                version,
            });
        }

        let request: VerifyRequest =
            serde_json::from_value(input).map_err(|error| InvocationError::InvalidInput {
                capability_id: capability_id.clone(),
                message: error.to_string(),
            })?;

        let result = verify(request).map_err(|error| InvocationError::Failed {
            message: error.to_string(),
        })?;

        serde_json::to_value(result).map_err(|error| InvocationError::Failed {
            message: error.to_string(),
        })
    }
}

fn verify(request: VerifyRequest) -> Result<VerifyResult, super::error::MathVerifyError> {
    match request {
        VerifyRequest::Numeric {
            canonical_solution,
            student_response,
        } => Ok(VerifyResult {
            is_correct: within_tolerance(student_response, canonical_solution),
            error: None,
        }),
        VerifyRequest::SymbolicExpression { .. } => {
            unimplemented!("added in Task 5")
        }
    }
}

pub(crate) fn within_tolerance(student: f64, canonical: f64) -> bool {
    if !student.is_finite() || !canonical.is_finite() {
        return false;
    }
    let tolerance = ABSOLUTE_EPSILON.max(RELATIVE_EPSILON * canonical.abs());
    (student - canonical).abs() <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(canonical_solution: f64, student_response: f64) -> Value {
        serde_json::json!({
            "response_type": "numeric",
            "canonical_solution": canonical_solution,
            "student_response": student_response,
        })
    }

    fn invoke(input: Value) -> Result<Value, InvocationError> {
        let capability_id = CapabilityId::new(CAPABILITY_ID).unwrap();
        tauri::async_runtime::block_on(MathVerifyProvider.invoke(
            &capability_id,
            CAPABILITY_VERSION,
            input,
        ))
    }

    #[test]
    fn exact_match_is_correct() {
        let output = invoke(request(4.0, 4.0)).unwrap();
        assert_eq!(output["is_correct"], true);
        assert_eq!(output["error"], Value::Null);
    }

    #[test]
    fn within_tolerance_is_correct() {
        let output = invoke(request(1000.0, 1000.0 + 1e-10)).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn just_above_tolerance_is_incorrect() {
        let output = invoke(request(1.0, 1.0 + 2e-9)).unwrap();
        assert_eq!(output["is_correct"], false);
    }

    #[test]
    fn just_below_tolerance_is_incorrect() {
        let output = invoke(request(1.0, 1.0 - 2e-9)).unwrap();
        assert_eq!(output["is_correct"], false);
    }

    #[test]
    fn negative_values_compare_correctly() {
        let output = invoke(request(-4.0, -4.0)).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn zero_compares_correctly() {
        let output = invoke(request(0.0, 0.0)).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn non_finite_student_response_is_incorrect_not_an_error() {
        for student_response in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let result = verify(VerifyRequest::Numeric {
                canonical_solution: 4.0,
                student_response,
            })
            .unwrap();

            assert!(!result.is_correct);
            assert_eq!(result.error, None);
        }
    }

    #[test]
    fn unknown_capability_id_is_rejected() {
        let capability_id = CapabilityId::new("math.other").unwrap();
        let result = tauri::async_runtime::block_on(MathVerifyProvider.invoke(
            &capability_id,
            CAPABILITY_VERSION,
            request(1.0, 1.0),
        ));
        assert!(matches!(
            result,
            Err(InvocationError::UnknownCapability { .. })
        ));
    }

    #[test]
    fn unknown_capability_version_is_rejected() {
        let capability_id = CapabilityId::new(CAPABILITY_ID).unwrap();
        let result = tauri::async_runtime::block_on(MathVerifyProvider.invoke(
            &capability_id,
            CAPABILITY_VERSION + 1,
            request(1.0, 1.0),
        ));
        assert!(matches!(
            result,
            Err(InvocationError::UnknownCapability { .. })
        ));
    }

    #[test]
    fn malformed_input_is_invalid_input() {
        let capability_id = CapabilityId::new(CAPABILITY_ID).unwrap();
        let result = tauri::async_runtime::block_on(MathVerifyProvider.invoke(
            &capability_id,
            CAPABILITY_VERSION,
            serde_json::json!({ "response_type": "numeric", "canonical_solution": "not a number" }),
        ));
        assert!(matches!(result, Err(InvocationError::InvalidInput { .. })));
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test --locked capabilities::math_verify::provider -- --nocapture`

Expected: all 10 tests pass. (The `SymbolicExpression` arm's `unimplemented!` is never
reached by these tests — every test here sends `response_type: "numeric"`.)

- [ ] **Step 3: Export it and commit**

In `src-tauri/src/capabilities/math_verify/mod.rs`:

```rust
mod error;
mod provider;
mod types;

pub use error::MathVerifyError;
pub use provider::MathVerifyProvider;
pub use types::{VerifyRequest, VerifyResult};
```

```bash
git add src-tauri/src/capabilities/math_verify/provider.rs src-tauri/src/capabilities/math_verify/mod.rs
git commit -m "feat(math-verify): numeric verification path"
```

---

### Task 5: `MathVerifyProvider` — symbolic-expression path

**Files:**
- Modify: `src-tauri/src/capabilities/math_verify/provider.rs`

**Interfaces:**
- Consumes: `within_tolerance` (Task 4), `MathVerifyError::UnparseableCanonicalSolution`
  (Task 3), `mathcore::MathCore::new()` / `.calculate(&str) -> Result<f64, mathcore::MathError>`
  (Task 1).
- Produces: the completed `verify()` function (no more `unimplemented!`).

- [ ] **Step 1: Write the failing tests**

In `src-tauri/src/capabilities/math_verify/provider.rs`, replace the `verify` function body's
`SymbolicExpression` arm:

```rust
fn verify(request: VerifyRequest) -> Result<VerifyResult, super::error::MathVerifyError> {
    match request {
        VerifyRequest::Numeric {
            canonical_solution,
            student_response,
        } => Ok(VerifyResult {
            is_correct: within_tolerance(student_response, canonical_solution),
            error: None,
        }),
        VerifyRequest::SymbolicExpression {
            canonical_solution,
            student_response,
        } => {
            let math = mathcore::MathCore::new();
            let canonical_value = math.calculate(&canonical_solution).map_err(|error| {
                super::error::MathVerifyError::UnparseableCanonicalSolution {
                    expression: canonical_solution.clone(),
                    message: error.to_string(),
                }
            })?;

            match math.calculate(&student_response) {
                Ok(student_value) => Ok(VerifyResult {
                    is_correct: within_tolerance(student_value, canonical_value),
                    error: None,
                }),
                Err(error) => Ok(VerifyResult {
                    is_correct: false,
                    error: Some(error.to_string()),
                }),
            }
        }
    }
}
```

Add these tests to the `#[cfg(test)] mod tests` block in the same file:

```rust
    fn symbolic_request(canonical_solution: &str, student_response: &str) -> Value {
        serde_json::json!({
            "response_type": "symbolic-expression",
            "canonical_solution": canonical_solution,
            "student_response": student_response,
        })
    }

    #[test]
    fn equivalent_symbolic_forms_are_correct() {
        for student_response in ["tau", "2*3.14159265358979", "2 * pi"] {
            let output = invoke(symbolic_request("2*pi", student_response)).unwrap();
            assert_eq!(
                output["is_correct"], true,
                "expected {student_response:?} to match 2*pi"
            );
        }
    }

    #[test]
    fn wrong_but_parseable_symbolic_answer_is_incorrect() {
        let output = invoke(symbolic_request("2*pi", "pi")).unwrap();
        assert_eq!(output["is_correct"], false);
        assert_eq!(output["error"], Value::Null);
    }

    #[test]
    fn unparseable_student_response_is_incorrect_with_error_set() {
        let output = invoke(symbolic_request("2*pi", "2*")).unwrap();
        assert_eq!(output["is_correct"], false);
        assert!(output["error"].is_string());
    }

    #[test]
    fn unparseable_canonical_solution_is_an_invocation_failure() {
        let capability_id = CapabilityId::new(CAPABILITY_ID).unwrap();
        let result = tauri::async_runtime::block_on(MathVerifyProvider.invoke(
            &capability_id,
            CAPABILITY_VERSION,
            symbolic_request("2*+pi", "tau"),
        ));
        assert!(matches!(result, Err(InvocationError::Failed { .. })));
    }

    #[test]
    fn sine_function_matches_hand_computed_value() {
        // sin(pi/2) = 1
        let output = invoke(symbolic_request("1", "sin(pi/2)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn cosine_function_matches_hand_computed_value() {
        // cos(0) = 1
        let output = invoke(symbolic_request("1", "cos(0)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn tangent_function_matches_hand_computed_value() {
        // tan(pi/4) = 1
        let output = invoke(symbolic_request("1", "tan(pi/4)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn exponential_function_matches_hand_computed_value() {
        // exp(1) = e
        let output = invoke(symbolic_request("e", "exp(1)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn natural_log_function_matches_hand_computed_value() {
        // ln(e^2) = 2
        let output = invoke(symbolic_request("2", "ln(e^2)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn sqrt_function_matches_hand_computed_value() {
        // sqrt(2)^2 = 2
        let output = invoke(symbolic_request("2", "sqrt(2)^2")).unwrap();
        assert_eq!(output["is_correct"], true);
    }
```

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test --locked capabilities::math_verify -- --nocapture`

Expected: all tests in `capabilities::math_verify::provider` pass (the 10 from Task 4 plus
the 10 added here). If `sqrt_function_matches_hand_computed_value` or
`natural_log_function_matches_hand_computed_value` fails because `mathcore` rounds
differently than expected, print the actual computed values
(`math.calculate("sqrt(2)^2")`) with `--nocapture` and confirm the discrepancy is a genuine
floating-point tolerance issue (adjust the test's expected value, never widen the production
`ABSOLUTE_EPSILON`/`RELATIVE_EPSILON` constants to make a test pass) rather than a real
`mathcore` bug — if it looks like a real bug in `mathcore` itself, stop and flag it in the
task's worklog rather than silently working around it.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/capabilities/math_verify/provider.rs
git commit -m "feat(math-verify): symbolic-expression verification path via mathcore"
```

---

### Task 6: First-party manifest and full registry round-trip

**Files:**
- Create: `src-tauri/src/capabilities/math_verify/module.toml`
- Modify: `src-tauri/src/capabilities/math_verify/mod.rs`
- Create: `src-tauri/src/capabilities/math_verify/tests/mod.rs`

**Interfaces:**
- Consumes: `modules::parse`, `modules::ModuleRegistry`, `modules::ModuleInstallation`,
  `modules::CapabilityRequirement`, `modules::CapabilityCall`, `modules::CallEnvelope`,
  `modules::ModuleId` (all already exported from `src-tauri/src/modules/mod.rs`);
  `MathVerifyProvider`, `CAPABILITY_ID`/`CAPABILITY_VERSION` constants (Task 4).
- Produces: `MANIFEST_TOML: &str` (`pub`, exported from `mod.rs`) — the embedded manifest
  text, in case a future app-startup wiring task needs it (that wiring itself is out of
  scope here, per Global Constraints).

- [ ] **Step 1: Write the manifest**

Create `src-tauri/src/capabilities/math_verify/module.toml`:

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

- [ ] **Step 2: Write the failing round-trip test**

Create `src-tauri/src/capabilities/math_verify/tests/mod.rs`:

```rust
use serde_json::json;

use crate::capabilities::math_verify::MathVerifyProvider;
use crate::modules::{
    parse, CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement, ModuleId,
    ModuleInstallation, ModuleRegistry,
};

const MANIFEST_TOML: &str = include_str!("../module.toml");

#[test]
fn manifest_registers_and_verifies_a_correct_numeric_answer_end_to_end() {
    let manifest = parse(MANIFEST_TOML).expect("module.toml must parse");
    let module_id = manifest.id.clone();

    let mut registry = ModuleRegistry::new();
    registry
        .register(manifest, Box::new(MathVerifyProvider))
        .expect("registration must succeed");

    let installation = ModuleInstallation {
        workspace_id: "test-workspace".to_owned(),
        enabled_module_ids: vec![module_id.clone()],
    };
    let requirement = CapabilityRequirement {
        id: CapabilityId::new("math.verify").unwrap(),
        min_version: 1,
    };
    let handle = registry
        .resolve(&installation, &requirement)
        .expect("math.verify must resolve");

    let call = CapabilityCall {
        envelope: CallEnvelope {
            workspace_id: installation.workspace_id.clone(),
            capability_id: CapabilityId::new("math.verify").unwrap(),
            version: 1,
            calling_module_id: ModuleId::new("core.test_caller").unwrap(),
        },
        input: json!({
            "response_type": "numeric",
            "canonical_solution": 42.0,
            "student_response": 42.0,
        }),
    };

    let output: serde_json::Value =
        tauri::async_runtime::block_on(registry.invoke(&handle, &installation, call))
            .expect("invocation must succeed");

    assert_eq!(output["is_correct"], true);
}
```

- [ ] **Step 3: Wire the manifest constant and test module, run the test**

In `src-tauri/src/capabilities/math_verify/mod.rs`:

```rust
mod error;
mod provider;
mod types;

pub use error::MathVerifyError;
pub use provider::MathVerifyProvider;
pub use types::{VerifyRequest, VerifyResult};

/// The embedded first-party manifest for this capability. Not yet wired into any running
/// `ModuleRegistry` — app-startup registration is out of scope for this module (see the
/// implementation plan's Global Constraints).
pub const MANIFEST_TOML: &str = include_str!("module.toml");

#[cfg(test)]
mod tests;
```

Run: `cd src-tauri && cargo test --locked capabilities::math_verify::tests -- --nocapture`

Expected: `manifest_registers_and_verifies_a_correct_numeric_answer_end_to_end` passes.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/capabilities/math_verify/module.toml src-tauri/src/capabilities/math_verify/mod.rs src-tauri/src/capabilities/math_verify/tests
git commit -m "feat(math-verify): embed first-party manifest, full registry round-trip test"
```

---

### Task 7: Full validation pass

**Files:** none (verification only).

**Interfaces:** none.

- [ ] **Step 1: Run the full test suite**

Run: `cd src-tauri && cargo test --locked --quiet`

Expected: every existing test still passes, plus every test added in Tasks 2–6 (26 new
tests: 4 + 1 + 10 + 10 + 1 — count them and confirm the total matches before proceeding).

- [ ] **Step 2: Run clippy**

Run: `cd src-tauri && cargo clippy --all-targets --locked --quiet -- -D warnings`

Expected: no warnings. Fix any that appear (do not `#[allow]` them away without a specific
reason recorded in the task's worklog).

- [ ] **Step 3: Run fmt and the whitespace check**

Run: `cd src-tauri && cargo fmt --all --check`
Run: `cd /home/marcus/axiom && git diff --check`

Expected: both clean.

- [ ] **Step 4: Update the task file and commit**

Update `.ai/tasks/055-math-verify.md`: move `status` to `review`, fill in "What was built /
tested / left out" with the actual validation command output (test count, clippy/fmt
results), and add a `## Review` heading placeholder for the next reviewer, following the
exact shape `.ai/tasks/_archive/054-canonical-problem-schema.md` used.

```bash
git add .ai/tasks/055-math-verify.md
git commit -m "docs(055): mark math.verify implementation complete, ready for review"
```
