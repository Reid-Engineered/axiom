use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflinePartialAvailability {
    pub available_count: i64,
    pub total_count: i64,
    pub limit_reason: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineKindAvailability {
    pub kind: String,
    pub enabled: bool,
    pub size_bytes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial: Option<OfflinePartialAvailability>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub guiding_goal_id: String,
    pub progress: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_concept_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity_at: Option<String>,
    pub paused: bool,
    pub offline_availability: Vec<OfflineKindAvailability>,
    pub enabled_module_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceActivityEvent {
    pub id: String,
    pub workspace_id: String,
    pub occurred_at: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceInput {
    pub subject: String,
    pub goal_text: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalInferredStructure {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mastery_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concept_scope: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: String,
    pub workspace_id: String,
    pub text: String,
    pub state: String,
    pub inferred: GoalInferredStructure,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub achieved_summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConceptDiagnostic {
    pub id: String,
    pub expression: String,
    #[serde(rename = "type")]
    pub diagnostic_type: String,
    pub note: String,
    pub occurred_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Concept {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub chapter: String,
    pub mastery_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub was_mastery_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decayed_at: Option<String>,
    pub meaning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_for_review_in_days: Option<i64>,
    pub on_exam: bool,
    pub blocks_concept_ids: Vec<String>,
    pub prerequisite_concept_ids: Vec<String>,
    pub related_concept_ids: Vec<String>,
    pub leads_to_concept_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learner_heuristic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heuristic_evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_it_shows_up: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_diagnostics: Option<Vec<ConceptDiagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity_at: Option<String>,
    pub notes_count: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub id: String,
    pub name: String,
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learner_count_label: Option<String>,
    pub developer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_value_detail: Option<String>,
    pub context_seen: String,
    pub offline_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_concept_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub works_with_module_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suits: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_notes: Option<Vec<String>>,
    pub enabled: bool,
    pub visibility: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tool_count: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIntent {
    pub activity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_minutes: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TutorExchange {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub occurred_at: String,
    pub pinned_to_visualization: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub workspace_id: String,
    pub concept_id: String,
    pub status: String,
    pub intent: SessionIntent,
    pub resume_summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    pub elapsed_minutes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_count: Option<i64>,
    pub exchanges: Vec<TutorExchange>,
    pub settled_conclusions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_question: Option<String>,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionInput {
    pub workspace_id: String,
    pub concept_id: String,
    pub intent: SessionIntent,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterSegment {
    pub label: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    pub id: String,
    pub workspace_id: String,
    pub title: String,
    pub edition: String,
    pub total_pages: i64,
    pub total_chapters: i64,
    pub segments: Vec<ChapterSegment>,
    pub highlights_count: i64,
    pub notes_count: i64,
    pub most_marked_sections: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialResult {
    pub id: String,
    pub kind: String,
    pub page: i64,
    pub title: String,
    pub reason: String,
    pub concept_id: String,
    pub in_syllabus: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercise_total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercise_attempted: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub workspace_id: String,
    pub concept_id: String,
    pub text: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleWorkspaceSeed {
    pub sample_workspace_id: String,
    pub workspaces: Vec<Workspace>,
    pub workspace_activity: Vec<WorkspaceActivityEvent>,
    pub goals: Vec<Goal>,
    pub concepts: Vec<Concept>,
    pub modules: Vec<Module>,
    pub workspace_templates: Vec<WorkspaceTemplate>,
    pub sessions: Vec<Session>,
    pub materials: Vec<Material>,
    pub material_results: Vec<MaterialResult>,
    pub notes: Vec<Note>,
}
