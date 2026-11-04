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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GenerateRequest {
    pub workspace_id: String,
    pub family_id: String,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub attempt_id: String,
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StartRequest {
    pub workspace_id: String,
    pub concept_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StartResponse {
    pub attempt_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ConceptsRequest {
    pub workspace_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptsResponse {
    pub concepts: Vec<ConceptDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptDescriptor {
    pub concept_id: String,
    pub name: String,
    pub topic: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DescribeRequest {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DescribeResponse {
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
    pub hints_revealed: u32,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EvaluateRequest {
    pub workspace_id: String,
    pub attempt_id: String,
    pub response: ResponseValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluateResponse {
    pub correct: bool,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct HintRequest {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HintResponse {
    pub hint_text: String,
    pub hints_revealed: u32,
    pub hints_total: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnavailableReason {
    NoEvidence,
    LegacyOnly,
    Unsupported,
    ProviderUnavailable,
    ContentUnavailable,
    PolicyUnapproved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRef {
    pub id: String,
    pub revision: u32,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentRef {
    pub package_id: String,
    pub package_version: String,
    pub schema_version: u32,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStampRef {
    pub module_id: String,
    pub module_version: String,
    pub capability_id: String,
    pub capability_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stamp {
    pub schema_version: u32,
    pub workspace_id: String,
    pub provider: ProviderStampRef,
    pub policy_ref: Option<PolicyRef>,
    pub content_ref: Option<ContentRef>,
    pub evidence_ids: Vec<String>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastKnownValue<T> {
    pub value: T,
    pub stamp: Stamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum Available<T> {
    Available {
        value: T,
        stamp: Stamp,
    },
    Unavailable {
        reason: UnavailableReason,
        #[serde(rename = "lastKnown")]
        last_known: Option<LastKnownValue<T>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PracticeDifficulty {
    Steady,
    Push,
    Exam,
}

impl PracticeDifficulty {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Steady => "Steady",
            Self::Push => "Push",
            Self::Exam => "Exam",
        }
    }

    pub fn hint_budget(self) -> u32 {
        match self {
            Self::Steady => 3,
            Self::Push => 1,
            Self::Exam => 0,
        }
    }

    pub fn cost_per_hint(self) -> f64 {
        match self {
            Self::Steady => 0.20,
            Self::Push => 0.35,
            Self::Exam => 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecayDetail {
    pub from_snapshot_id: String,
    pub from_score: f64,
    pub elapsed_ms: i64,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterySnapshot {
    pub id: String,
    pub concept_id: String,
    pub objective_ids: Vec<String>,
    pub score: f64,
    pub previous_snapshot_id: Option<String>,
    pub previous_score: Option<f64>,
    pub delta: Option<f64>,
    pub named_state: String,
    pub as_of: String,
    pub last_practiced_at: String,
    pub decay: Option<DecayDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloomEvidence {
    pub demonstrated_level: String,
    pub objective_ids: Vec<String>,
    pub qualifying_attempt_ids: Vec<String>,
    pub previous_level: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrerequisiteReadiness {
    pub concept_id: String,
    pub snapshot_id: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Readiness {
    pub state: String,
    pub prerequisites: Vec<PrerequisiteReadiness>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConceptEvidence {
    pub concept_id: String,
    pub mastery: Available<MasterySnapshot>,
    pub bloom: Available<BloomEvidence>,
    pub readiness: Available<Readiness>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribution {
    pub id: String,
    pub attempt_id: String,
    pub pattern_id: String,
    pub rule_revision: u32,
    pub opportunity_id: String,
    pub eligible: bool,
    pub outcome: String,
    pub evidence_ids: Vec<String>,
    pub detector_id: Option<String>,
    pub detector_version: Option<u32>,
    pub explanation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concept_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternResolutionInfo {
    pub required_clean: u32,
    pub eligible_clean: u32,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternSnapshot {
    pub id: String,
    pub pattern_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concept_ids: Option<Vec<String>>,
    pub status: String,
    pub opportunities: u32,
    pub occurrences: u32,
    pub unknown_count: u32,
    pub clean_streak: u32,
    pub first_seen_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub recent_attribution_ids: Vec<String>,
    pub rule_revision: u32,
    pub resolution: PatternResolutionInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternSessionBucket {
    pub session_id: String,
    pub recorded_at: String,
    pub opportunities: u32,
    pub occurrences: u32,
    pub clean_count: u32,
    pub unknown_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternDetail {
    pub snapshot: PatternSnapshot,
    pub explanation: String,
    pub next_action: String,
    pub attributions: Vec<Attribution>,
    pub session_buckets: Vec<PatternSessionBucket>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternListResponse {
    pub patterns: Available<Vec<PatternSnapshot>>,
    pub total_answers: u32,
    pub since_date: Option<String>,
    pub required_clean_default: u32,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanPhase {
    pub id: String,
    pub name: String,
    pub reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanQueueSlot {
    pub id: String,
    pub phase_id: String,
    pub concept_id: String,
    pub objective_ids: Vec<String>,
    pub family_id: String,
    pub seed: String,
    pub attempt_id: Option<String>,
    pub state: String,
    pub reason: String,
    pub evidence_ids: Vec<String>,
    pub replaces_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanChange {
    pub event_id: String,
    pub removed_slot_id: String,
    pub added_slot_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanGoalSnapshot {
    pub id: String,
    pub text: String,
    pub state: String,
    pub inferred: serde_json::Value,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub revision: u32,
    pub goal_id: Option<String>,
    pub goal_snapshot: Option<PlanGoalSnapshot>,
    pub target_ms: Option<i64>,
    pub difficulty: PracticeDifficulty,
    pub policy_ref: PolicyRef,
    pub phases: Vec<PlanPhase>,
    pub queue: Vec<PlanQueueSlot>,
    pub changes: Vec<PlanChange>,
    pub availability: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationObjectiveDto {
    pub objective_id: String,
    pub verdict: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationDto {
    pub id: String,
    pub verdict: String,
    pub method: String,
    pub checker_version: String,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
    pub canonical_value: Option<f64>,
    pub student_value: Option<f64>,
    pub restrictions: Vec<String>,
    pub parse_error: Option<String>,
    pub limitations: Vec<String>,
    pub objective_results: Vec<VerificationObjectiveDto>,
    pub samples: Option<serde_json::Value>,
    pub terms: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptState {
    pub id: String,
    pub slot_id: String,
    pub status: String,
    pub instance_ref: String,
    pub prompt: String,
    pub response_type: ResponseType,
    pub restrictions: Vec<String>,
    pub bloom_target: Option<String>,
    pub draft_revision: u32,
    pub answer_text: String,
    pub working_text: String,
    pub hints_total: u32,
    pub hints_revealed: u32,
    pub hint_budget: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revealed_hints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_solution_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worked_solution: Option<String>,
    pub submission_ids: Vec<String>,
    pub last_feedback_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verification: Option<VerificationDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTiming {
    pub elapsed_ms: i64,
    pub attempt_elapsed_ms: i64,
    pub target_ms: Option<i64>,
    pub checkpoint_sequence: u32,
    pub active: bool,
    pub measured_at: String,
    pub recovered: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionQuestionRow {
    pub slot_id: String,
    pub attempt_id: String,
    pub phase_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concept_id: Option<String>,
    pub prompt: String,
    pub result: String,
    pub feedback_id: Option<String>,
    pub before_snapshot_id: Option<String>,
    pub after_snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyHistoryEntry {
    pub slot_id: String,
    pub difficulty: PracticeDifficulty,
    pub policy_ref: PolicyRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session_id: String,
    pub completed_at: String,
    pub timing: Available<SessionTiming>,
    pub difficulty_history: Vec<DifficultyHistoryEntry>,
    pub correct: u32,
    pub assessed_total: u32,
    pub attempted_total: u32,
    pub skipped_total: u32,
    pub unverifiable_total: u32,
    pub hints_used: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_budget_total: Option<u32>,
    pub mastery_changes: Vec<Available<MasterySnapshot>>,
    pub repeated_pattern_ids: Vec<String>,
    pub question_rows: Vec<SessionQuestionRow>,
    pub next_plan: Available<Plan>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionActionReceipt {
    pub operation_id: String,
    pub session_id: String,
    pub revision: u32,
    pub event_ids: Vec<String>,
    pub attempt: Option<AttemptState>,
    pub mastery_changes: Vec<Available<MasterySnapshot>>,
    pub bloom_changes: Vec<Available<BloomEvidence>>,
    pub pattern_changes: Vec<Available<PatternSnapshot>>,
    pub plan: Plan,
    pub timing: Available<SessionTiming>,
    pub summary: Available<SessionSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanInput {
    pub workspace_id: String,
    pub operation_id: String,
    #[serde(default)]
    pub goal_id: Option<String>,
    #[serde(default)]
    pub concept_ids: Vec<String>,
    #[serde(default)]
    pub target_ms: Option<i64>,
    #[serde(default)]
    pub question_limit: Option<u32>,
    #[serde(default)]
    pub difficulty: Option<PracticeDifficulty>,
    #[serde(default)]
    pub pattern_id: Option<String>,
    #[serde(default)]
    pub preview_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionActionInput {
    pub workspace_id: String,
    pub session_id: String,
    pub operation_id: String,
    pub expected_revision: u32,
    pub action: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub elapsed_delta_ms: Option<i64>,
    #[serde(default)]
    pub attempt_elapsed_delta_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditPatternRuleInput {
    pub workspace_id: String,
    pub operation_id: String,
    pub expected_revision: u32,
    pub pattern_id: String,
    pub rule_revision: u32,
    pub required_clean: u32,
    #[serde(default)]
    pub preview_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHistoryEntry {
    pub session_id: String,
    pub workspace_id: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub difficulty: Option<PracticeDifficulty>,
    pub summary: Available<SessionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceReadRequest {
    pub workspace_id: String,
    #[serde(default)]
    pub concept_ids: Vec<String>,
    #[serde(default)]
    pub as_of: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternsReadRequest {
    pub workspace_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternDetailReadRequest {
    pub workspace_id: String,
    pub pattern_id: String,
    #[serde(default)]
    pub rule_revision: Option<u32>,
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
            ResponseValue::SymbolicExpression {
                value: "2*pi".to_owned()
            }
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
