use std::sync::Arc;

use serde_json::Value;
use tauri::async_runtime::RwLock;

use crate::knowledge::KnowledgePackage;
use crate::modules::{
    CapabilityId, CapabilityProvider, InvocationError, ModuleInstallation, ModuleRegistry,
};

use super::error::PracticeError;
use super::store::PracticeStore;
use super::types::{GenerateRequest, GenerateResponse};

pub struct PracticeProvider {
    store: PracticeStore,
    knowledge_package: KnowledgePackage,
    // Holds the registry it will itself be registered into (Task 6/9's registration test) —
    // an intentional Arc reference cycle, acceptable because ModuleRegistry is a
    // process-lifetime singleton, never freed before exit. See spec §4. Must be the
    // async-aware RwLock (tauri::async_runtime::RwLock, not std::sync::RwLock) because
    // Task 7's evaluate() holds the read guard across an inner `.await`, and only an
    // async-aware lock's guard is Send.
    #[allow(dead_code)]
    registry: Arc<RwLock<ModuleRegistry>>,
    installation: ModuleInstallation,
}

impl PracticeProvider {
    pub fn new(
        store: PracticeStore,
        knowledge_package: KnowledgePackage,
        registry: Arc<RwLock<ModuleRegistry>>,
        installation: ModuleInstallation,
    ) -> Self {
        Self {
            store,
            knowledge_package,
            registry,
            installation,
        }
    }

    async fn handle_generate(&self, input: Value) -> Result<Value, InvocationError> {
        let request: GenerateRequest = serde_json::from_value(input).map_err(|error| {
            InvocationError::InvalidInput {
                capability_id: capability_id("practice.generate"),
                message: error.to_string(),
            }
        })?;
        let response = self
            .generate(request)
            .await
            .map_err(|error| to_invocation_error("practice.generate", error))?;
        serde_json::to_value(response).map_err(|error| InvocationError::Failed {
            message: error.to_string(),
        })
    }

    async fn generate(
        &self,
        request: GenerateRequest,
    ) -> Result<GenerateResponse, PracticeError> {
        let family = self
            .knowledge_package
            .problem_families
            .iter()
            .find(|family| family.id.as_str() == request.family_id)
            .ok_or_else(|| PracticeError::FamilyNotFound {
                family_id: request.family_id.clone(),
            })?;

        let seed = request.seed.unwrap_or_else(random_seed);
        let instance = crate::generation::generate_problem_instance(family, seed)?;

        let attempt_id = format!("attempt-{}", uuid::Uuid::new_v4());
        self.store.insert_attempt(
            &attempt_id,
            &request.workspace_id,
            request.family_id.as_str(),
            seed,
            &instance,
        )?;

        Ok(GenerateResponse {
            attempt_id,
            prompt: instance.prompt,
            response_type: family.response_type,
            hints_total: instance.hints.len() as u32,
        })
    }
}

#[async_trait::async_trait]
impl CapabilityProvider for PracticeProvider {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        input: Value,
    ) -> Result<Value, InvocationError> {
        match (capability_id.as_str(), version) {
            ("practice.generate", 1) => self.handle_generate(input).await,
            _ => Err(InvocationError::UnknownCapability {
                capability_id: capability_id.clone(),
                version,
            }),
        }
    }
}

fn capability_id(value: &str) -> CapabilityId {
    CapabilityId::new(value).expect("static capability id is valid")
}

fn to_invocation_error(capability: &str, error: PracticeError) -> InvocationError {
    match error {
        PracticeError::FamilyNotFound { .. }
        | PracticeError::AttemptNotFound { .. }
        | PracticeError::NoMoreHints { .. }
        | PracticeError::AlreadySolved { .. }
        | PracticeError::ResponseTypeMismatch { .. } => InvocationError::InvalidInput {
            capability_id: capability_id(capability),
            message: error.to_string(),
        },
        PracticeError::GenerationFailed(_)
        | PracticeError::VerificationFailed(_)
        | PracticeError::Storage(_) => InvocationError::Failed {
            message: error.to_string(),
        },
    }
}

/// A seed with no reproducibility requirement (real `practice.generate` calls, as opposed to
/// tests that pass an explicit `seed`). `RandomState`'s keys are drawn from OS randomness by
/// `std`, with zero new dependency — deliberately not the `rand` crate, matching the project's
/// existing no-new-dependency pattern (see the problem-generation design's own RNG decision).
fn random_seed() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::modules::ModuleId;

    fn fixture_package() -> KnowledgePackage {
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical");
        crate::knowledge::load_knowledge_package(&fixture_root).unwrap()
    }

    fn provider() -> PracticeProvider {
        let store = PracticeStore::new(crate::db::open_in_memory().unwrap());
        {
            let mut connection = store.connection_for_test();
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
        }
        let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
        let installation = ModuleInstallation {
            workspace_id: "ws-1".to_owned(),
            enabled_module_ids: vec![ModuleId::new("org.axiom.practice").unwrap()],
        };
        PracticeProvider::new(store, fixture_package(), registry, installation)
    }

    #[test]
    fn generate_with_an_explicit_seed_matches_the_generation_engine_directly() {
        let provider = provider();
        let family = provider
            .knowledge_package
            .problem_families
            .iter()
            .find(|family| family.id.as_str() == "problem.shell_y_poly")
            .unwrap();
        let expected = crate::generation::generate_problem_instance(family, 42).unwrap();

        let request = GenerateRequest {
            workspace_id: "ws-1".to_owned(),
            family_id: "problem.shell_y_poly".to_owned(),
            seed: Some(42),
        };
        let response = tauri::async_runtime::block_on(provider.generate(request)).unwrap();

        assert_eq!(response.prompt, expected.prompt);
        assert_eq!(response.hints_total, expected.hints.len() as u32);
    }

    #[test]
    fn generate_response_never_exposes_the_canonical_solution() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.generate").unwrap();
        let input = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
            "seed": 42,
        });

        let output = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, input))
            .unwrap();

        assert!(output.get("canonical_solution").is_none());
        assert!(output.get("hints").is_none());
        assert!(output["attempt_id"].is_string());
    }

    #[test]
    fn generate_with_an_unknown_family_id_is_invalid_input() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.generate").unwrap();
        let input = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.nonexistent",
        });

        let result = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, input));

        assert!(matches!(result, Err(InvocationError::InvalidInput { .. })));
    }

    #[test]
    fn generate_without_a_seed_still_succeeds_and_produces_a_persisted_attempt() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.generate").unwrap();
        let input = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
        });

        let output = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, input))
            .unwrap();

        let attempt_id = output["attempt_id"].as_str().unwrap();
        assert!(provider.store.load_attempt(attempt_id, "ws-1").is_ok());
    }

    #[test]
    fn unknown_capability_id_is_rejected() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.other").unwrap();
        let result =
            tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, Value::Null));
        assert!(matches!(
            result,
            Err(InvocationError::UnknownCapability { .. })
        ));
    }
}
