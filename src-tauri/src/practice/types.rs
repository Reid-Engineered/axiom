use serde::{Deserialize, Serialize};

use crate::knowledge::ResponseType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttemptStatus {
    Open,
    Solved,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "response_type", rename_all = "kebab-case")]
pub enum ResponseValue {
    SymbolicExpression { value: String },
    Numeric { value: f64 },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GenerateRequest {
    pub workspace_id: String,
    pub family_id: String,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GenerateResponse {
    pub attempt_id: String,
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EvaluateRequest {
    pub workspace_id: String,
    pub attempt_id: String,
    pub response: ResponseValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvaluateResponse {
    pub correct: bool,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HintRequest {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HintResponse {
    pub hint_text: String,
    pub hints_revealed: u32,
    pub hints_total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_request_deserializes_without_a_seed() {
        let value = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
        });
        let request: GenerateRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.seed, None);
    }

    #[test]
    fn generate_request_deserializes_with_an_explicit_seed() {
        let value = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
            "seed": 42,
        });
        let request: GenerateRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.seed, Some(42));
    }

    #[test]
    fn generate_response_never_serializes_a_canonical_solution_field() {
        let response = GenerateResponse {
            attempt_id: "attempt-1".to_owned(),
            prompt: "Find the volume.".to_owned(),
            response_type: ResponseType::SymbolicExpression,
            hints_total: 2,
        };
        let value = serde_json::to_value(&response).unwrap();
        assert!(value.get("canonical_solution").is_none());
        assert!(value.get("hints").is_none());
    }

    #[test]
    fn numeric_response_value_round_trips() {
        let value = serde_json::json!({ "response_type": "numeric", "value": 4.0 });
        let response: ResponseValue = serde_json::from_value(value).unwrap();
        assert_eq!(response, ResponseValue::Numeric { value: 4.0 });
    }

    #[test]
    fn symbolic_response_value_round_trips() {
        let value = serde_json::json!({
            "response_type": "symbolic-expression",
            "value": "2*pi",
        });
        let response: ResponseValue = serde_json::from_value(value).unwrap();
        assert_eq!(
            response,
            ResponseValue::SymbolicExpression { value: "2*pi".to_owned() }
        );
    }

    #[test]
    fn hint_response_never_serializes_an_unrevealed_hint_list() {
        let response = HintResponse {
            hint_text: "Identify the radius.".to_owned(),
            hints_revealed: 1,
            hints_total: 3,
        };
        let value = serde_json::to_value(&response).unwrap();
        assert!(value.get("hints").is_none());
        assert_eq!(value["hint_text"], "Identify the radius.");
    }
}
