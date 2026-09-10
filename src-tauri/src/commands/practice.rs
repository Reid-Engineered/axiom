use std::sync::Arc;

use tauri::async_runtime::RwLock;

use crate::capabilities::math_verify::MathVerifyProvider;
use crate::knowledge::KnowledgePackage;
use crate::modules::{parse, ModuleId, ModuleInstallation, ModuleRegistry};
use crate::practice::PracticeProvider;

pub fn build_practice_registry(
    knowledge_package: KnowledgePackage,
    connection: rusqlite::Connection,
) -> (Arc<RwLock<ModuleRegistry>>, ModuleInstallation) {
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            parse(crate::capabilities::math_verify::MANIFEST_TOML)
                .expect("math_verify manifest must parse"),
            Box::new(MathVerifyProvider),
        )
        .expect("math_verify must register");
    let registry = Arc::new(RwLock::new(registry));

    let installation = ModuleInstallation {
        workspace_id: String::new(),
        enabled_module_ids: vec![
            ModuleId::new("core.math_verify").expect("static id is valid"),
            ModuleId::new("org.axiom.practice").expect("static id is valid"),
        ],
    };

    let store = crate::practice::PracticeStore::new(connection);
    let provider = PracticeProvider::new(
        store,
        knowledge_package,
        Arc::clone(&registry),
        installation.clone(),
    );
    registry
        .blocking_write()
        .register(
            parse(crate::practice::MANIFEST_TOML).expect("practice manifest must parse"),
            Box::new(provider),
        )
        .expect("practice must register");

    (registry, installation)
}

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::knowledge::ResponseType;
use crate::modules::{
    CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement, RegistryError,
};
use crate::practice::{
    AttemptStatus as PracticeAttemptStatus, DescribeRequest, DescribeResponse, EvaluateRequest,
    EvaluateResponse, GenerateRequest, GenerateResponse, HintRequest, HintResponse, ResponseValue,
};

use super::CommandResult;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateAttemptInput {
    pub workspace_id: String,
    pub family_id: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub attempt_id: String,
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
}

impl From<GenerateResponse> for Attempt {
    fn from(response: GenerateResponse) -> Self {
        Self {
            attempt_id: response.attempt_id,
            prompt: response.prompt,
            response_type: response.response_type,
            hints_total: response.hints_total,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "responseType", rename_all = "kebab-case")]
pub enum ResponseValueInput {
    SymbolicExpression { value: String },
    Numeric { value: f64 },
}

impl From<ResponseValueInput> for ResponseValue {
    fn from(input: ResponseValueInput) -> Self {
        match input {
            ResponseValueInput::SymbolicExpression { value } => {
                ResponseValue::SymbolicExpression { value }
            }
            ResponseValueInput::Numeric { value } => ResponseValue::Numeric { value },
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateAttemptInput {
    pub workspace_id: String,
    pub attempt_id: String,
    pub response: ResponseValueInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AttemptStatus {
    Open,
    Solved,
}

impl From<PracticeAttemptStatus> for AttemptStatus {
    fn from(status: PracticeAttemptStatus) -> Self {
        match status {
            PracticeAttemptStatus::Open => AttemptStatus::Open,
            PracticeAttemptStatus::Solved => AttemptStatus::Solved,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationResult {
    pub correct: bool,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

impl From<EvaluateResponse> for EvaluationResult {
    fn from(response: EvaluateResponse) -> Self {
        Self {
            correct: response.correct,
            status: response.status.into(),
            submission_count: response.submission_count,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestHintInput {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hint {
    pub hint_text: String,
    pub hints_revealed: u32,
    pub hints_total: u32,
}

impl From<HintResponse> for Hint {
    fn from(response: HintResponse) -> Self {
        Self {
            hint_text: response.hint_text,
            hints_revealed: response.hints_revealed,
            hints_total: response.hints_total,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeAttemptInput {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttemptDescription {
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
    pub hints_revealed: u32,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

impl From<DescribeResponse> for AttemptDescription {
    fn from(response: DescribeResponse) -> Self {
        Self {
            prompt: response.prompt,
            response_type: response.response_type,
            hints_total: response.hints_total,
            hints_revealed: response.hints_revealed,
            status: response.status.into(),
            submission_count: response.submission_count,
        }
    }
}

async fn invoke_practice<Input, Output>(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    capability: &str,
    workspace_id: String,
    input: Input,
) -> Result<Output, RegistryError>
where
    Input: Serialize,
    Output: serde::de::DeserializeOwned,
{
    let requirement = CapabilityRequirement {
        id: CapabilityId::new(capability).expect("static capability id is valid"),
        min_version: 1,
    };
    let handle = {
        let registry = registry.read().await;
        registry.resolve(installation, &requirement)?
    };
    let call = CapabilityCall {
        envelope: CallEnvelope {
            workspace_id,
            capability_id: requirement.id.clone(),
            version: 1,
            calling_module_id: ModuleId::new("core.tauri_commands")
                .expect("static module id is valid"),
        },
        input,
    };
    let registry = registry.read().await;
    registry.invoke(&handle, installation, call).await
}

fn practice_error(error: RegistryError) -> String {
    error.to_string()
}

pub async fn generate_attempt_handler(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: GenerateAttemptInput,
) -> CommandResult<Attempt> {
    let response: GenerateResponse = invoke_practice(
        registry,
        installation,
        "practice.generate",
        input.workspace_id.clone(),
        GenerateRequest {
            workspace_id: input.workspace_id,
            family_id: input.family_id,
            seed: None,
        },
    )
    .await
    .map_err(practice_error)?;
    Ok(response.into())
}

#[tauri::command(rename = "generateAttempt", rename_all = "camelCase")]
pub async fn generate_attempt(
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: GenerateAttemptInput,
) -> CommandResult<Attempt> {
    generate_attempt_handler(&registry, &installation, input).await
}

pub async fn evaluate_attempt_handler(
    database: &super::Database,
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: EvaluateAttemptInput,
) -> CommandResult<EvaluationResult> {
    let response: EvaluateResponse = invoke_practice(
        registry,
        installation,
        "practice.evaluate",
        input.workspace_id.clone(),
        EvaluateRequest {
            workspace_id: input.workspace_id,
            attempt_id: input.attempt_id.clone(),
            response: input.response.into(),
        },
    )
    .await
    .map_err(practice_error)?;
    super::session::touch_session_for_attempt(database, &input.attempt_id)?;
    Ok(response.into())
}

#[tauri::command(rename = "evaluateAttempt", rename_all = "camelCase")]
pub async fn evaluate_attempt(
    database: State<'_, super::Database>,
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: EvaluateAttemptInput,
) -> CommandResult<EvaluationResult> {
    evaluate_attempt_handler(&database, &registry, &installation, input).await
}

pub async fn request_hint_handler(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: RequestHintInput,
) -> CommandResult<Hint> {
    let response: HintResponse = invoke_practice(
        registry,
        installation,
        "practice.hint",
        input.workspace_id.clone(),
        HintRequest {
            workspace_id: input.workspace_id,
            attempt_id: input.attempt_id,
        },
    )
    .await
    .map_err(practice_error)?;
    Ok(response.into())
}

#[tauri::command(rename = "requestHint", rename_all = "camelCase")]
pub async fn request_hint(
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: RequestHintInput,
) -> CommandResult<Hint> {
    request_hint_handler(&registry, &installation, input).await
}

pub async fn describe_attempt_handler(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: DescribeAttemptInput,
) -> CommandResult<AttemptDescription> {
    let response: DescribeResponse = invoke_practice(
        registry,
        installation,
        "practice.describe",
        input.workspace_id.clone(),
        DescribeRequest {
            workspace_id: input.workspace_id,
            attempt_id: input.attempt_id,
        },
    )
    .await
    .map_err(practice_error)?;
    Ok(response.into())
}

#[tauri::command(rename = "describeAttempt", rename_all = "camelCase")]
pub async fn describe_attempt(
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: DescribeAttemptInput,
) -> CommandResult<AttemptDescription> {
    describe_attempt_handler(&registry, &installation, input).await
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::modules::{CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement};

    use super::*;

    #[test]
    fn practice_command_names_match_frontend() {
        assert_eq!(__tauri_command_name_generate_attempt!(), "generateAttempt");
        assert_eq!(__tauri_command_name_evaluate_attempt!(), "evaluateAttempt");
        assert_eq!(__tauri_command_name_request_hint!(), "requestHint");
    }

    #[test]
    fn frontend_response_values_translate_to_the_capability_contract() {
        for response in [
            serde_json::json!({"responseType": "symbolic-expression", "value": "0"}),
            serde_json::json!({"responseType": "numeric", "value": 0.0}),
        ] {
            let input: EvaluateAttemptInput = serde_json::from_value(serde_json::json!({
                "workspaceId": "ws-1",
                "attemptId": "attempt-1",
                "response": response,
            }))
            .unwrap();
            let request = EvaluateRequest {
                workspace_id: input.workspace_id,
                attempt_id: input.attempt_id,
                response: input.response.into(),
            };
            let value = serde_json::to_value(request).unwrap();
            assert_eq!(value["workspace_id"], "ws-1");
            assert_eq!(value["attempt_id"], "attempt-1");
            assert_eq!(value["response"]["response_type"], response["responseType"]);
            assert_eq!(value["response"]["value"], response["value"]);
            assert!(value.get("workspaceId").is_none());
        }
    }

    #[test]
    fn missing_provider_maps_to_a_nonempty_command_error() {
        let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
        let installation = ModuleInstallation {
            workspace_id: String::new(),
            enabled_module_ids: Vec::new(),
        };
        let input: GenerateAttemptInput = serde_json::from_value(serde_json::json!({
            "workspaceId": "ws-1",
            "familyId": "problem.shell_y_poly",
        }))
        .unwrap();
        let error = tauri::async_runtime::block_on(generate_attempt_handler(
            &registry,
            &installation,
            input,
        ))
        .unwrap_err();
        assert!(!error.is_empty());
    }

    fn fixture_package() -> KnowledgePackage {
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical");
        crate::knowledge::load_knowledge_package(&fixture_root).unwrap()
    }

    fn seeded_connection() -> rusqlite::Connection {
        let mut connection = crate::db::open_in_memory().unwrap();
        let transaction = connection.transaction().unwrap();
        transaction
            .execute(
                "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
                 VALUES ('ws-1', 'Test', 'goal-1', 0.0, 0)",
                [],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO goals (id, workspace_id, text, state, created_at, updated_at)
                 VALUES ('goal-1', 'ws-1', 'Test goal', 'Guiding', ?1, ?1)",
                ["2026-09-04T12:00:00Z"],
            )
            .unwrap();
        transaction.commit().unwrap();
        connection
    }

    #[test]
    fn build_practice_registry_registers_both_first_party_modules() {
        let (registry, installation) =
            build_practice_registry(fixture_package(), seeded_connection());

        let math_verify_handle = tauri::async_runtime::block_on(registry.read()).resolve(
            &installation,
            &CapabilityRequirement {
                id: CapabilityId::new("math.verify").unwrap(),
                min_version: 1,
            },
        );
        assert!(math_verify_handle.is_ok());

        let practice_handle = tauri::async_runtime::block_on(registry.read()).resolve(
            &installation,
            &CapabilityRequirement {
                id: CapabilityId::new("practice.generate").unwrap(),
                min_version: 1,
            },
        );
        assert!(practice_handle.is_ok());
    }

    #[test]
    fn build_practice_registry_can_actually_generate_an_attempt() {
        let (registry, installation) =
            build_practice_registry(fixture_package(), seeded_connection());
        let handle = tauri::async_runtime::block_on(registry.read())
            .resolve(
                &installation,
                &CapabilityRequirement {
                    id: CapabilityId::new("practice.generate").unwrap(),
                    min_version: 1,
                },
            )
            .unwrap();

        let call = CapabilityCall {
            envelope: CallEnvelope {
                workspace_id: "ws-1".to_owned(),
                capability_id: CapabilityId::new("practice.generate").unwrap(),
                version: 1,
                calling_module_id: ModuleId::new("core.test_caller").unwrap(),
            },
            input: serde_json::json!({
                "workspace_id": "ws-1",
                "family_id": "problem.shell_y_poly",
                "seed": 42,
            }),
        };

        let output: serde_json::Value = tauri::async_runtime::block_on(async {
            let registry = registry.read().await;
            registry.invoke(&handle, &installation, call).await
        })
        .unwrap();

        assert!(output["attempt_id"].is_string());
    }
    #[test]
    fn generate_attempt_translates_response_to_camel_case_shape() {
        let (registry, installation) =
            build_practice_registry(fixture_package(), seeded_connection());

        let attempt = tauri::async_runtime::block_on(generate_attempt_handler(
            &registry,
            &installation,
            GenerateAttemptInput {
                workspace_id: "ws-1".to_owned(),
                family_id: "problem.shell_y_poly".to_owned(),
            },
        ))
        .unwrap();

        assert!(!attempt.attempt_id.is_empty());
        assert!(!attempt.prompt.is_empty());
        assert!(attempt.hints_total >= 1);

        let value = serde_json::to_value(&attempt).unwrap();
        assert!(
            value.get("attemptId").is_some(),
            "expected camelCase attemptId key"
        );
        assert!(
            value.get("attempt_id").is_none(),
            "must not leak snake_case keys"
        );
    }

    #[test]
    fn generate_attempt_with_unknown_family_is_an_error() {
        let (registry, installation) =
            build_practice_registry(fixture_package(), seeded_connection());

        let result = tauri::async_runtime::block_on(generate_attempt_handler(
            &registry,
            &installation,
            GenerateAttemptInput {
                workspace_id: "ws-1".to_owned(),
                family_id: "problem.nonexistent".to_owned(),
            },
        ));

        assert!(result.is_err());
    }

    #[test]
    fn describe_attempt_returns_the_current_state_of_a_generated_attempt() {
        let (registry, installation) =
            build_practice_registry(fixture_package(), seeded_connection());
        let generated = tauri::async_runtime::block_on(generate_attempt_handler(
            &registry,
            &installation,
            GenerateAttemptInput {
                workspace_id: "ws-1".to_owned(),
                family_id: "problem.shell_y_poly".to_owned(),
            },
        ))
        .unwrap();

        let described = tauri::async_runtime::block_on(describe_attempt_handler(
            &registry,
            &installation,
            DescribeAttemptInput {
                workspace_id: "ws-1".to_owned(),
                attempt_id: generated.attempt_id.clone(),
            },
        ))
        .unwrap();

        assert_eq!(described.prompt, generated.prompt);
        assert_eq!(described.hints_total, generated.hints_total);
        assert_eq!(described.hints_revealed, 0);
        assert_eq!(described.status, AttemptStatus::Open);
        assert_eq!(described.submission_count, 0);
    }

    #[test]
    fn describe_attempt_for_an_unknown_id_is_an_error() {
        let (registry, installation) =
            build_practice_registry(fixture_package(), seeded_connection());

        let result = tauri::async_runtime::block_on(describe_attempt_handler(
            &registry,
            &installation,
            DescribeAttemptInput {
                workspace_id: "ws-1".to_owned(),
                attempt_id: "attempt-missing".to_owned(),
            },
        ));

        assert!(result.is_err());
    }

    #[test]
    fn full_generate_evaluate_hint_sequence_round_trips_through_the_command_layer() {
        let (registry, installation) =
            build_practice_registry(fixture_package(), seeded_connection());

        let attempt = tauri::async_runtime::block_on(generate_attempt_handler(
            &registry,
            &installation,
            GenerateAttemptInput {
                workspace_id: "ws-1".to_owned(),
                family_id: "problem.shell_y_poly".to_owned(),
            },
        ))
        .unwrap();

        let hint = tauri::async_runtime::block_on(request_hint_handler(
            &registry,
            &installation,
            RequestHintInput {
                workspace_id: "ws-1".to_owned(),
                attempt_id: attempt.attempt_id.clone(),
            },
        ))
        .unwrap();
        assert_eq!(hint.hints_revealed, 1);

        let evaluation = tauri::async_runtime::block_on(evaluate_attempt_handler(
            &crate::commands::Database::open_in_memory().unwrap(),
            &registry,
            &installation,
            EvaluateAttemptInput {
                workspace_id: "ws-1".to_owned(),
                attempt_id: attempt.attempt_id,
                response: ResponseValueInput::SymbolicExpression {
                    value: "0".to_owned(),
                },
            },
        ))
        .unwrap();
        assert_eq!(evaluation.status, AttemptStatus::Open);
        assert_eq!(evaluation.submission_count, 1);

        let evaluation_value = serde_json::to_value(&evaluation).unwrap();
        assert!(evaluation_value.get("submissionCount").is_some());
    }

    fn bundled_package() -> KnowledgePackage {
        let package_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package");
        crate::knowledge::load_knowledge_package(&package_root)
            .expect("the bundled knowledge-package must load")
    }

    fn bundled_shell_y_poly_family() -> crate::knowledge::ProblemFamily {
        bundled_package()
            .problem_families
            .into_iter()
            .find(|family| family.id.as_str() == "problem.shell_y_poly")
            .expect("bundled knowledge-package must contain problem.shell_y_poly")
    }

    /// Generates an attempt at a known seed by calling `practice.generate` through the
    /// registry, because the `generateAttempt` command deliberately exposes no `seed`
    /// (task 058, spec §8) and the test needs to know which instance it is answering.
    fn seeded_attempt_id(
        registry: &Arc<RwLock<ModuleRegistry>>,
        installation: &ModuleInstallation,
        seed: u64,
    ) -> String {
        let handle = tauri::async_runtime::block_on(registry.read())
            .resolve(
                installation,
                &CapabilityRequirement {
                    id: CapabilityId::new("practice.generate").unwrap(),
                    min_version: 1,
                },
            )
            .unwrap();
        let call = CapabilityCall {
            envelope: CallEnvelope {
                workspace_id: "ws-1".to_owned(),
                capability_id: CapabilityId::new("practice.generate").unwrap(),
                version: 1,
                calling_module_id: ModuleId::new("core.test_caller").unwrap(),
            },
            input: serde_json::json!({
                "workspace_id": "ws-1",
                "family_id": "problem.shell_y_poly",
                "seed": seed,
            }),
        };
        let output: serde_json::Value = tauri::async_runtime::block_on(async {
            let registry = registry.read().await;
            registry.invoke(&handle, installation, call).await
        })
        .unwrap();
        output["attempt_id"].as_str().unwrap().to_owned()
    }

    fn evaluate_symbolic(
        registry: &Arc<RwLock<ModuleRegistry>>,
        installation: &ModuleInstallation,
        attempt_id: &str,
        response: String,
    ) -> EvaluationResult {
        tauri::async_runtime::block_on(evaluate_attempt_handler(
            &crate::commands::Database::open_in_memory().unwrap(),
            registry,
            installation,
            EvaluateAttemptInput {
                workspace_id: "ws-1".to_owned(),
                attempt_id: attempt_id.to_owned(),
                response: ResponseValueInput::SymbolicExpression { value: response },
            },
        ))
        .unwrap()
    }

    /// Task 059: the real bundled content, not the canonical test fixture, reaches the real
    /// `generateAttempt` command and comes back as a usable problem.
    #[test]
    fn generate_attempt_succeeds_against_the_real_bundled_knowledge_package() {
        let (registry, installation) =
            build_practice_registry(bundled_package(), seeded_connection());

        let attempt = tauri::async_runtime::block_on(generate_attempt_handler(
            &registry,
            &installation,
            GenerateAttemptInput {
                workspace_id: "ws-1".to_owned(),
                family_id: "problem.shell_y_poly".to_owned(),
            },
        ))
        .unwrap();

        assert!(!attempt.attempt_id.is_empty());
        assert_eq!(attempt.response_type, ResponseType::SymbolicExpression);
        assert_eq!(attempt.hints_total, 4);
        assert!(attempt.prompt.contains("revolving R around the y-axis"));
        for placeholder in ["{coeff}", "{a}", "{b}"] {
            assert!(
                !attempt.prompt.contains(placeholder),
                "prompt still contains {placeholder}: {}",
                attempt.prompt
            );
        }

        let hint = tauri::async_runtime::block_on(request_hint_handler(
            &registry,
            &installation,
            RequestHintInput {
                workspace_id: "ws-1".to_owned(),
                attempt_id: attempt.attempt_id,
            },
        ))
        .unwrap();
        assert_eq!(hint.hints_total, 4);
        assert!(!hint.hint_text.trim().is_empty());
    }

    /// Task 059: `math.verify` accepts a correct answer to the real bundled family --
    /// independently computed from the closed form rather than echoing the generator's own
    /// canonical string -- and still rejects a wrong one, through the real command handler.
    #[test]
    fn bundled_shell_y_poly_answers_are_accepted_by_math_verify_through_the_command_layer() {
        let (registry, installation) =
            build_practice_registry(bundled_package(), seeded_connection());
        let family = bundled_shell_y_poly_family();

        for seed in [1u64, 7, 42, 1_337, 90_210] {
            let instance = crate::generation::generate_problem_instance(&family, seed).unwrap();
            let coeff = instance.resolved_parameters["coeff"];
            let b = instance.resolved_parameters["b"];
            // V = 2*pi*(c*b^3/3 - b^4/4), derived by hand from Rule 2.6 -- see task 059.
            let volume = 2.0 * std::f64::consts::PI * (coeff * b.powi(3) / 3.0 - b.powi(4) / 4.0);

            let attempt_id = seeded_attempt_id(&registry, &installation, seed);
            let wrong = evaluate_symbolic(
                &registry,
                &installation,
                &attempt_id,
                format!("{}", volume + 1.0),
            );
            assert!(
                !wrong.correct,
                "seed {seed}: an answer off by 1 was accepted"
            );
            assert_eq!(wrong.status, AttemptStatus::Open);

            let right =
                evaluate_symbolic(&registry, &installation, &attempt_id, format!("{volume}"));
            assert!(
                right.correct,
                "seed {seed}: the hand-derived volume {volume} was rejected"
            );
            assert_eq!(right.status, AttemptStatus::Solved);
            assert_eq!(right.submission_count, 2);

            // A second attempt at the same instance, answered in an algebraically different
            // but equivalent form, to show verification is by value and not by string match.
            let equivalent_attempt_id = seeded_attempt_id(&registry, &installation, seed);
            let equivalent = evaluate_symbolic(
                &registry,
                &installation,
                &equivalent_attempt_id,
                format!("pi*(2*{coeff}*{b}^3/3 - {b}^4/2)"),
            );
            assert!(
                equivalent.correct,
                "seed {seed}: an equivalent exact form was rejected"
            );
        }
    }
}
