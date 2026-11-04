use std::sync::Arc;

use tauri::async_runtime::RwLock;

use crate::capabilities::math_verify::MathVerifyProvider;
use crate::modules::{
    parse, CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement, ModuleId,
    ModuleInstallation, ModuleRegistry,
};
use crate::practice::{ConceptsRequest, ConceptsResponse};

use super::store::PracticeStore;
use super::PracticeProvider;

fn fixture_package() -> crate::knowledge::KnowledgePackage {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/knowledge/tests/fixtures/canonical");
    crate::knowledge::load_knowledge_package(&fixture_root).unwrap()
}

#[test]
fn practice_manifest_parses_and_declares_math_verify_as_a_requirement() {
    let manifest = parse(super::MANIFEST_TOML).expect("module.toml must parse");
    assert_eq!(manifest.id.as_str(), "org.axiom.practice");
    assert!(manifest.requires.iter().any(|requirement| {
        requirement.id.as_str() == "math.verify" && requirement.min_version == 1
    }));
    assert_eq!(manifest.provides.len(), 10);
    for capability_id in [
        "practice.generate",
        "practice.evaluate",
        "practice.hint",
        "practice.start",
        "practice.describe",
        "practice.concepts",
        "practice.plan",
        "practice.session",
        "practice.evidence",
        "practice.patterns",
    ] {
        assert!(manifest
            .provides
            .iter()
            .any(|provided| provided.id.as_str() == capability_id && provided.version == 1));
    }
}

#[test]
fn practice_concepts_returns_the_bundled_curriculum() {
    let package_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package");
    let package = crate::knowledge::load_knowledge_package(&package_root).unwrap();
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let installation = ModuleInstallation {
        workspace_id: String::new(),
        enabled_module_ids: vec![ModuleId::new("org.axiom.practice").unwrap()],
    };
    let provider = PracticeProvider::new(
        PracticeStore::new(crate::db::open_in_memory().unwrap()),
        package,
        Arc::clone(&registry),
        installation.clone(),
    );
    registry
        .blocking_write()
        .register(parse(super::MANIFEST_TOML).unwrap(), Box::new(provider))
        .unwrap();

    let requirement = CapabilityRequirement {
        id: CapabilityId::new("practice.concepts").unwrap(),
        min_version: 1,
    };
    let response: ConceptsResponse = tauri::async_runtime::block_on(async {
        let registry = registry.read().await;
        let handle = registry.resolve(&installation, &requirement).unwrap();
        registry
            .invoke(
                &handle,
                &installation,
                CapabilityCall {
                    envelope: CallEnvelope {
                        workspace_id: "workspace-new".to_owned(),
                        capability_id: requirement.id,
                        version: 1,
                        calling_module_id: ModuleId::new("core.workspace").unwrap(),
                    },
                    input: ConceptsRequest {
                        workspace_id: "workspace-new".to_owned(),
                    },
                },
            )
            .await
    })
    .unwrap();

    assert_eq!(response.concepts.len(), 3);
    let shell = response
        .concepts
        .iter()
        .find(|concept| concept.concept_id == "shell.method_vertical_axis")
        .unwrap();
    assert_eq!(
        shell.name,
        "The Method of Cylindrical Shells (Vertical Axis of Revolution)"
    );
    assert_eq!(shell.topic, "2.3 Volumes of Revolution: Cylindrical Shells");
    assert!(!shell.summary.trim().is_empty());
}

#[test]
fn practice_resolves_math_verify_through_a_real_registry_end_to_end() {
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    registry
        .blocking_write()
        .register(
            parse(crate::capabilities::math_verify::MANIFEST_TOML).unwrap(),
            Box::new(MathVerifyProvider),
        )
        .unwrap();

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
    let installation = ModuleInstallation {
        workspace_id: "ws-1".to_owned(),
        enabled_module_ids: vec![
            ModuleId::new("core.math_verify").unwrap(),
            ModuleId::new("org.axiom.practice").unwrap(),
        ],
    };
    let provider = PracticeProvider::new(
        store,
        fixture_package(),
        Arc::clone(&registry),
        installation.clone(),
    );
    registry
        .blocking_write()
        .register(parse(super::MANIFEST_TOML).unwrap(), Box::new(provider))
        .unwrap();

    let handle = registry
        .blocking_read()
        .resolve(
            &installation,
            &CapabilityRequirement {
                id: CapabilityId::new("practice.generate").unwrap(),
                min_version: 1,
            },
        )
        .expect("practice.generate must resolve");

    let call = crate::modules::CapabilityCall {
        envelope: crate::modules::CallEnvelope {
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
    .expect("practice.generate invocation must succeed");

    assert!(output["attempt_id"].is_string());
    assert!(output.get("canonical_solution").is_none());
}

/// Builds a registry with the real `math.verify` provider and Practice over the *bundled*
/// knowledge package, against a seeded in-memory database with one workspace (`ws-1`).
fn bundled_practice_registry() -> (Arc<RwLock<ModuleRegistry>>, ModuleInstallation) {
    let package_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package");
    let package = crate::knowledge::load_knowledge_package(&package_root).unwrap();

    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    registry
        .blocking_write()
        .register(
            parse(crate::capabilities::math_verify::MANIFEST_TOML).unwrap(),
            Box::new(MathVerifyProvider),
        )
        .unwrap();

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

    let installation = ModuleInstallation {
        workspace_id: "ws-1".to_owned(),
        enabled_module_ids: vec![
            ModuleId::new("core.math_verify").unwrap(),
            ModuleId::new("org.axiom.practice").unwrap(),
        ],
    };
    let provider =
        PracticeProvider::new(store, package, Arc::clone(&registry), installation.clone());
    registry
        .blocking_write()
        .register(parse(super::MANIFEST_TOML).unwrap(), Box::new(provider))
        .unwrap();
    (registry, installation)
}

fn invoke_practice<Input: serde::Serialize, Output: serde::de::DeserializeOwned>(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    capability: &str,
    input: Input,
) -> Output {
    let requirement = CapabilityRequirement {
        id: CapabilityId::new(capability).unwrap(),
        min_version: 1,
    };
    tauri::async_runtime::block_on(async {
        let registry = registry.read().await;
        let handle = registry.resolve(installation, &requirement).unwrap();
        registry
            .invoke(
                &handle,
                installation,
                CapabilityCall {
                    envelope: CallEnvelope {
                        workspace_id: "ws-1".to_owned(),
                        capability_id: requirement.id.clone(),
                        version: 1,
                        calling_module_id: ModuleId::new("core.test_caller").unwrap(),
                    },
                    input,
                },
            )
            .await
    })
    .unwrap_or_else(|error| panic!("{capability} failed: {error:?}"))
}

fn generate_corpus_attempt(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    seed: u64,
) -> crate::practice::GenerateResponse {
    invoke_practice(
        registry,
        installation,
        "practice.generate",
        crate::practice::GenerateRequest {
            workspace_id: "ws-1".to_owned(),
            family_id: crate::generation::tests::corpus::SHELL_Y_POLY_FAMILY_ID.to_owned(),
            seed: Some(seed),
        },
    )
}

fn evaluate_expression(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    attempt_id: &str,
    expression: String,
) -> crate::practice::EvaluateResponse {
    invoke_practice(
        registry,
        installation,
        "practice.evaluate",
        crate::practice::EvaluateRequest {
            workspace_id: "ws-1".to_owned(),
            attempt_id: attempt_id.to_owned(),
            response: crate::practice::ResponseValue::SymbolicExpression { value: expression },
        },
    )
}

/// Task 067: every committed regression-corpus case, served through the real Practice
/// provider, must present exactly the committed prompt and hint ladder, accept the committed
/// canonical solution through `math.verify`, and reject a known-wrong answer. This pins the
/// generator *and* verification agreeing on committed content, which the generator-only
/// corpus runner can't see.
#[test]
fn regression_corpus_cases_round_trip_through_practice_and_math_verify() {
    use crate::practice::{AttemptStatus, HintRequest, HintResponse};

    let (registry, installation) = bundled_practice_registry();

    for case in crate::generation::tests::corpus::load_shell_y_poly_cases() {
        let seed = case.seed;
        let attempt = generate_corpus_attempt(&registry, &installation, seed);
        assert_eq!(attempt.prompt, case.prompt, "seed {seed}: prompt drifted");
        assert_eq!(
            attempt.response_type, case.response_type,
            "seed {seed}: response type drifted"
        );
        assert_eq!(
            attempt.hints_total as usize,
            case.hints.len(),
            "seed {seed}: hint count drifted"
        );

        for (index, expected) in case.hints.iter().enumerate() {
            let hint: HintResponse = invoke_practice(
                &registry,
                &installation,
                "practice.hint",
                HintRequest {
                    workspace_id: "ws-1".to_owned(),
                    attempt_id: attempt.attempt_id.clone(),
                },
            );
            assert_eq!(
                &hint.hint_text, expected,
                "seed {seed}: hints[{index}] drifted"
            );
            assert_eq!(hint.hints_revealed as usize, index + 1);
        }

        let wrong = evaluate_expression(
            &registry,
            &installation,
            &attempt.attempt_id,
            format!("({}) + 1", case.canonical_expression()),
        );
        assert!(
            !wrong.correct,
            "seed {seed}: a known-wrong answer was accepted"
        );
        assert_eq!(wrong.status, AttemptStatus::Open);

        let right = evaluate_expression(
            &registry,
            &installation,
            &attempt.attempt_id,
            case.canonical_expression().to_owned(),
        );
        assert!(
            right.correct,
            "seed {seed}: the committed canonical solution {:?} was rejected",
            case.canonical_expression()
        );
        assert_eq!(right.status, AttemptStatus::Solved);
        assert_eq!(right.submission_count, 2);
    }
}
