use serde_json::Value;
use uuid::Uuid;

use crate::modules::{CapabilityId, CapabilityProvider, InvocationError};

use super::error::MathVerifyError;
use super::types::{
    ObjectiveVerifyResult, VerifyMethod, VerifyRequest, VerifyResult, VerifyV2Request,
    VerifyV2Result, VerifyVerdict,
};

pub(crate) const CAPABILITY_ID: &str = "math.verify";
pub(crate) const CAPABILITY_VERSION_V1: u32 = 1;
pub(crate) const CAPABILITY_VERSION_V2: u32 = 2;
pub(crate) const CHECKER_VERSION: &str = "1.0.0";
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
        if capability_id.as_str() != CAPABILITY_ID {
            return Err(InvocationError::UnknownCapability {
                capability_id: capability_id.clone(),
                version,
            });
        }

        match version {
            CAPABILITY_VERSION_V1 => {
                let request: VerifyRequest = serde_json::from_value(input).map_err(|error| {
                    InvocationError::InvalidInput {
                        capability_id: capability_id.clone(),
                        message: error.to_string(),
                    }
                })?;

                let result = verify(request).map_err(|error| InvocationError::Failed {
                    message: error.to_string(),
                })?;

                serde_json::to_value(result).map_err(|error| InvocationError::Failed {
                    message: error.to_string(),
                })
            }
            CAPABILITY_VERSION_V2 => {
                let request: VerifyV2Request = serde_json::from_value(input).map_err(|error| {
                    InvocationError::InvalidInput {
                        capability_id: capability_id.clone(),
                        message: error.to_string(),
                    }
                })?;

                let result = verify_v2(request).map_err(|error| InvocationError::Failed {
                    message: error.to_string(),
                })?;

                serde_json::to_value(result).map_err(|error| InvocationError::Failed {
                    message: error.to_string(),
                })
            }
            _ => Err(InvocationError::UnknownCapability {
                capability_id: capability_id.clone(),
                version,
            }),
        }
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

pub fn verify_v2(request: VerifyV2Request) -> Result<VerifyV2Result, MathVerifyError> {
    let verification_id = format!("ver_{}", Uuid::new_v4());
    let mut limitations = vec![
        "finite-constant-evaluation-only".to_owned(),
        "partial-credit-unsupported".to_owned(),
        "symbolic-equivalence-proof-unsupported".to_owned(),
    ];

    match request.response_type.as_str() {
        "numeric" => {
            let canonical_value = extract_f64(&request.canonical_solution).ok_or_else(|| {
                MathVerifyError::UnparseableCanonicalSolution {
                    expression: request.canonical_solution.to_string(),
                    message: "expected numeric canonical_solution".to_owned(),
                }
            })?;

            let (verdict, student_value, parse_error) = match extract_f64(&request.response) {
                Some(val) if val.is_finite() => {
                    let is_ok = within_tolerance(val, canonical_value);
                    (
                        if is_ok {
                            VerifyVerdict::Correct
                        } else {
                            VerifyVerdict::Incorrect
                        },
                        Some(val),
                        None,
                    )
                }
                Some(_) => (
                    VerifyVerdict::Unverifiable,
                    None,
                    Some("non-finite numeric value".to_owned()),
                ),
                None => {
                    let raw_text = request
                        .response
                        .as_str()
                        .unwrap_or_default()
                        .trim()
                        .to_owned();
                    if let Ok(parsed) = raw_text.parse::<f64>() {
                        if parsed.is_finite() {
                            let is_ok = within_tolerance(parsed, canonical_value);
                            (
                                if is_ok {
                                    VerifyVerdict::Correct
                                } else {
                                    VerifyVerdict::Incorrect
                                },
                                Some(parsed),
                                None,
                            )
                        } else {
                            (
                                VerifyVerdict::Unverifiable,
                                None,
                                Some("non-finite numeric value".to_owned()),
                            )
                        }
                    } else {
                        (
                            VerifyVerdict::Unverifiable,
                            None,
                            Some(format!("could not parse numeric response {raw_text:?}")),
                        )
                    }
                }
            };

            let objective_results = request
                .objective_ids
                .iter()
                .map(|obj_id| ObjectiveVerifyResult {
                    objective_id: obj_id.clone(),
                    verdict,
                    evidence: match verdict {
                        VerifyVerdict::Correct => "numeric-tolerance-match".to_owned(),
                        VerifyVerdict::Incorrect => "numeric-tolerance-mismatch".to_owned(),
                        VerifyVerdict::Partial => "partial".to_owned(),
                        VerifyVerdict::Unverifiable => "unverifiable-input".to_owned(),
                    },
                })
                .collect();

            Ok(VerifyV2Result {
                id: verification_id,
                verdict,
                method: VerifyMethod::NumericTolerance,
                checker_version: CHECKER_VERSION.to_owned(),
                absolute_tolerance: ABSOLUTE_EPSILON,
                relative_tolerance: RELATIVE_EPSILON,
                canonical_value: Some(canonical_value),
                student_value,
                restrictions: request.restrictions,
                parse_error,
                limitations,
                objective_results,
                samples: None,
                terms: None,
            })
        }
        "symbolic-expression" => {
            let canonical_expr = request
                .canonical_solution
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| request.canonical_solution.to_string());
            let student_expr = request
                .response
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| request.response.to_string());

            let math = mathcore::MathCore::new();
            let canonical_eval = math.calculate(&canonical_expr);

            let (verdict, canonical_value, student_value, parse_error) = match canonical_eval {
                Ok(c_val) if c_val.is_finite() => match math.calculate(&student_expr) {
                    Ok(s_val) if s_val.is_finite() => {
                        let is_ok = within_tolerance(s_val, c_val);
                        (
                            if is_ok {
                                VerifyVerdict::Correct
                            } else {
                                VerifyVerdict::Incorrect
                            },
                            Some(c_val),
                            Some(s_val),
                            None,
                        )
                    }
                    Ok(_) => (
                        VerifyVerdict::Unverifiable,
                        Some(c_val),
                        None,
                        Some("student expression evaluated to a non-finite value".to_owned()),
                    ),
                    Err(error) => {
                        limitations.push("parser-or-free-variable-limitation".to_owned());
                        (
                            VerifyVerdict::Unverifiable,
                            Some(c_val),
                            None,
                            Some(error.to_string()),
                        )
                    }
                },
                Ok(_) => {
                    limitations.push("canonical-non-finite-constant".to_owned());
                    (
                        VerifyVerdict::Unverifiable,
                        None,
                        None,
                        Some("canonical solution is non-finite".to_owned()),
                    )
                }
                Err(error) => {
                    return Err(MathVerifyError::UnparseableCanonicalSolution {
                        expression: canonical_expr,
                        message: error.to_string(),
                    });
                }
            };

            let objective_results = request
                .objective_ids
                .iter()
                .map(|obj_id| ObjectiveVerifyResult {
                    objective_id: obj_id.clone(),
                    verdict,
                    evidence: match verdict {
                        VerifyVerdict::Correct => "constant-expression-numeric-match".to_owned(),
                        VerifyVerdict::Incorrect => {
                            "constant-expression-numeric-mismatch".to_owned()
                        }
                        VerifyVerdict::Partial => "partial".to_owned(),
                        VerifyVerdict::Unverifiable => "unverifiable-expression".to_owned(),
                    },
                })
                .collect();

            Ok(VerifyV2Result {
                id: verification_id,
                verdict,
                method: VerifyMethod::ConstantExpressionNumeric,
                checker_version: CHECKER_VERSION.to_owned(),
                absolute_tolerance: ABSOLUTE_EPSILON,
                relative_tolerance: RELATIVE_EPSILON,
                canonical_value,
                student_value,
                restrictions: request.restrictions,
                parse_error,
                limitations,
                objective_results,
                samples: None,
                terms: None,
            })
        }
        _ => {
            limitations.push(format!(
                "unsupported-response-type:{}",
                request.response_type
            ));
            Ok(VerifyV2Result {
                id: verification_id,
                verdict: VerifyVerdict::Unverifiable,
                method: VerifyMethod::ConstantExpressionNumeric,
                checker_version: CHECKER_VERSION.to_owned(),
                absolute_tolerance: ABSOLUTE_EPSILON,
                relative_tolerance: RELATIVE_EPSILON,
                canonical_value: None,
                student_value: None,
                restrictions: request.restrictions,
                parse_error: Some(format!(
                    "response_type {:?} is not supported by math.verify@2",
                    request.response_type
                )),
                limitations,
                objective_results: Vec::new(),
                samples: None,
                terms: None,
            })
        }
    }
}

fn extract_f64(value: &Value) -> Option<f64> {
    if let Some(n) = value.as_f64() {
        return Some(n);
    }
    if let Some(obj) = value.as_object() {
        if let Some(v) = obj.get("value").and_then(Value::as_f64) {
            return Some(v);
        }
    }
    None
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
            CAPABILITY_VERSION_V1,
            input,
        ))
    }

    fn invoke_v2(input: Value) -> Result<Value, InvocationError> {
        let capability_id = CapabilityId::new(CAPABILITY_ID).unwrap();
        tauri::async_runtime::block_on(MathVerifyProvider.invoke(
            &capability_id,
            CAPABILITY_VERSION_V2,
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
            CAPABILITY_VERSION_V1,
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
            CAPABILITY_VERSION_V2 + 1,
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
            CAPABILITY_VERSION_V1,
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
            CAPABILITY_VERSION_V1,
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

    #[test]
    fn v2_verifies_constant_expression_and_reports_honest_method_and_null_samples() {
        let output = invoke_v2(serde_json::json!({
            "workspace_id": "ws-1",
            "attempt_id": "att-1",
            "response": "sin(pi/2)",
            "canonical_solution": "1",
            "response_type": "symbolic-expression",
            "restrictions": ["exact-value"],
            "content_ref": {
                "package_id": "pkg.calc1",
                "package_version": "0.1.0",
                "entity_id": "fam.deriv",
                "schema_version": 2,
                "digest": "sha256:abc"
            },
            "objective_ids": ["obj.trig"]
        }))
        .unwrap();

        assert_eq!(output["verdict"], "correct");
        assert_eq!(output["method"], "constant-expression-numeric");
        assert_eq!(output["checker_version"], "1.0.0");
        assert_eq!(output["samples"], Value::Null);
        assert_eq!(output["terms"], Value::Null);
        assert_eq!(output["objective_results"][0]["objective_id"], "obj.trig");
        assert_eq!(output["objective_results"][0]["verdict"], "correct");
    }

    #[test]
    fn v2_unparseable_student_expression_returns_unverifiable_not_incorrect() {
        let output = invoke_v2(serde_json::json!({
            "workspace_id": "ws-1",
            "attempt_id": "att-1",
            "response": "2*",
            "canonical_solution": "2*pi",
            "response_type": "symbolic-expression",
            "restrictions": [],
            "content_ref": {
                "package_id": "pkg.calc1",
                "package_version": "0.1.0",
                "entity_id": "fam.deriv",
                "schema_version": 2,
                "digest": "sha256:abc"
            },
            "objective_ids": ["obj.trig"]
        }))
        .unwrap();

        assert_eq!(output["verdict"], "unverifiable");
        assert!(output["parse_error"].is_string());
    }
}
