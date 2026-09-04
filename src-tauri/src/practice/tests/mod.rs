use std::sync::Arc;

use tauri::async_runtime::RwLock;

use crate::capabilities::math_verify::MathVerifyProvider;
use crate::modules::{
    parse, CapabilityId, CapabilityRequirement, ModuleId, ModuleInstallation, ModuleRegistry,
};

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
    assert_eq!(manifest.provides.len(), 3);
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
