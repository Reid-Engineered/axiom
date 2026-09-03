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
