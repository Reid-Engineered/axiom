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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::modules::{
        CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement,
    };

    use super::*;

    fn fixture_package() -> KnowledgePackage {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/knowledge/tests/fixtures/canonical");
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
        let (registry, installation) = build_practice_registry(fixture_package(), seeded_connection());

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
        let (registry, installation) = build_practice_registry(fixture_package(), seeded_connection());
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
}
