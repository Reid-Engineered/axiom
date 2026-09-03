use serde_json::json;

use crate::capabilities::math_verify::MathVerifyProvider;
use crate::modules::{
    parse, CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement, ModuleId,
    ModuleInstallation, ModuleRegistry,
};

const MANIFEST_TOML: &str = include_str!("../module.toml");

#[test]
fn manifest_registers_and_verifies_a_correct_numeric_answer_end_to_end() {
    let manifest = parse(MANIFEST_TOML).expect("module.toml must parse");
    let module_id = manifest.id.clone();

    let mut registry = ModuleRegistry::new();
    registry
        .register(manifest, Box::new(MathVerifyProvider))
        .expect("registration must succeed");

    let installation = ModuleInstallation {
        workspace_id: "test-workspace".to_owned(),
        enabled_module_ids: vec![module_id],
    };
    let requirement = CapabilityRequirement {
        id: CapabilityId::new("math.verify").unwrap(),
        min_version: 1,
    };
    let handle = registry
        .resolve(&installation, &requirement)
        .expect("math.verify must resolve");

    let call = CapabilityCall {
        envelope: CallEnvelope {
            workspace_id: installation.workspace_id.clone(),
            capability_id: CapabilityId::new("math.verify").unwrap(),
            version: 1,
            calling_module_id: ModuleId::new("core.test_caller").unwrap(),
        },
        input: json!({
            "response_type": "numeric",
            "canonical_solution": 42.0,
            "student_response": 42.0,
        }),
    };

    let output: serde_json::Value =
        tauri::async_runtime::block_on(registry.invoke(&handle, &installation, call))
            .expect("invocation must succeed");

    assert_eq!(output["is_correct"], true);
}
