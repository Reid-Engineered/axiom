use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VerifyResult {
    pub is_correct: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContentRefInput {
    pub package_id: String,
    pub package_version: String,
    pub entity_id: String,
    pub schema_version: u32,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VerifyV2Request {
    pub workspace_id: String,
    pub attempt_id: String,
    pub response: serde_json::Value,
    pub canonical_solution: serde_json::Value,
    pub response_type: String,
    #[serde(default)]
    pub restrictions: Vec<String>,
    pub content_ref: ContentRefInput,
    #[serde(default)]
    pub objective_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerifyVerdict {
    Correct,
    Incorrect,
    Partial,
    Unverifiable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerifyMethod {
    NumericTolerance,
    ConstantExpressionNumeric,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ObjectiveVerifyResult {
    pub objective_id: String,
    pub verdict: VerifyVerdict,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VerifySample {
    pub bindings: std::collections::BTreeMap<String, f64>,
    pub expected: f64,
    pub actual: f64,
    pub error: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct VerifyTerm {
    pub expression: String,
    pub verdict: VerifyVerdict,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VerifyV2Result {
    pub id: String,
    pub verdict: VerifyVerdict,
    pub method: VerifyMethod,
    pub checker_version: String,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
    pub canonical_value: Option<f64>,
    pub student_value: Option<f64>,
    pub restrictions: Vec<String>,
    pub parse_error: Option<String>,
    pub limitations: Vec<String>,
    pub objective_results: Vec<ObjectiveVerifyResult>,
    pub samples: Option<Vec<VerifySample>>,
    pub terms: Option<Vec<VerifyTerm>>,
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
