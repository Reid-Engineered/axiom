use serde_json::Value;

use crate::modules::{CapabilityId, CapabilityProvider, InvocationError};

use super::error::MathVerifyError;
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

fn verify(request: VerifyRequest) -> Result<VerifyResult, MathVerifyError> {
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
                MathVerifyError::UnparseableCanonicalSolution {
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

    fn symbolic_request(canonical_solution: &str, student_response: &str) -> Value {
        serde_json::json!({
            "response_type": "symbolic-expression",
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
        let output = invoke(request(1000.0, 1000.0 + 5e-7)).unwrap();
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
            serde_json::json!({
                "response_type": "numeric",
                "canonical_solution": "not a number"
            }),
        ));
        assert!(matches!(result, Err(InvocationError::InvalidInput { .. })));
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
        let output = invoke(symbolic_request("1", "sin(pi/2)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn cosine_function_matches_hand_computed_value() {
        let output = invoke(symbolic_request("1", "cos(0)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn tangent_function_matches_hand_computed_value() {
        let output = invoke(symbolic_request("1", "tan(pi/4)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn exponential_function_matches_hand_computed_value() {
        let output = invoke(symbolic_request("e", "exp(1)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn natural_log_function_matches_hand_computed_value() {
        let output = invoke(symbolic_request("2", "ln(e^2)")).unwrap();
        assert_eq!(output["is_correct"], true);
    }

    #[test]
    fn sqrt_function_matches_hand_computed_value() {
        let output = invoke(symbolic_request("2", "sqrt(2)^2")).unwrap();
        assert_eq!(output["is_correct"], true);
    }
}
