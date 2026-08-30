use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::modules::{
    parse, CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement,
    EmbeddedManifestSource, InvocationError, ManifestError, ManifestSource, ModuleId,
    ModuleInstallation, ModuleManifest, ModuleRegistry, RegistryError,
};

use super::fixture;
use super::providers::{EchoProvider, FailingProvider};

const INTEGRATION_MANIFESTS: &[(&str, &str)] = &[
    (
        "org.axiom.integration_echo",
        include_str!("fixtures/parse-register-valid-echo.toml"),
    ),
    (
        "org.axiom.integration_broken",
        include_str!("fixtures/parse-register-malformed.toml"),
    ),
    (
        "org.axiom.integration_secondary",
        include_str!("fixtures/parse-register-valid-secondary.toml"),
    ),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SerializableInput {
    prompt: String,
    count: u32,
}

fn installation(workspace_id: &str, enabled_module_ids: Vec<ModuleId>) -> ModuleInstallation {
    ModuleInstallation {
        workspace_id: workspace_id.to_owned(),
        enabled_module_ids,
    }
}

fn requirement(capability_id: &str, min_version: u32) -> CapabilityRequirement {
    CapabilityRequirement {
        id: CapabilityId::new(capability_id).unwrap(),
        min_version,
    }
}

fn call(
    workspace_id: &str,
    capability_id: &str,
    version: u32,
) -> CapabilityCall<serde_json::Value> {
    CapabilityCall {
        envelope: CallEnvelope {
            workspace_id: workspace_id.to_owned(),
            capability_id: CapabilityId::new(capability_id).unwrap(),
            version,
            calling_module_id: ModuleId::new("org.axiom.test_caller").unwrap(),
        },
        input: json!({ "probe": true }),
    }
}

fn invoke_value(
    registry: &ModuleRegistry,
    installation: &ModuleInstallation,
    handle: &crate::modules::CapabilityHandle,
    capability_id: &str,
    version: u32,
) -> serde_json::Value {
    tauri::async_runtime::block_on(registry.invoke(
        handle,
        installation,
        call(&installation.workspace_id, capability_id, version),
    ))
    .unwrap()
}

#[test]
fn duplicate_module_id_rejects_second_registration_without_corrupting_first() {
    let first = parse(fixture("duplicate-module-id-first.toml")).unwrap();
    let second = parse(fixture("duplicate-module-id-second.toml")).unwrap();
    let module_id = first.id.clone();
    let mut registry = ModuleRegistry::new();

    registry
        .register(
            first.clone(),
            Box::new(EchoProvider::for_manifest(&first, "first")),
        )
        .unwrap();
    assert_eq!(
        registry.register(
            second.clone(),
            Box::new(EchoProvider::for_manifest(&second, "second")),
        ),
        Err(RegistryError::DuplicateModuleId(module_id.clone()))
    );

    let installation = installation("duplicate-workspace", vec![module_id]);
    let handle = registry
        .resolve(&installation, &requirement("fixture.echo", 1))
        .unwrap();
    let output = invoke_value(&registry, &installation, &handle, "fixture.echo", 1);
    assert_eq!(output["provider"], "first");
}

#[test]
fn missing_dependency_returns_no_compatible_provider() {
    let registry = ModuleRegistry::new();
    let installation = installation("missing-workspace", Vec::new());

    assert_eq!(
        registry.resolve(&installation, &requirement("missing.dependency", 1)),
        Err(RegistryError::NoCompatibleProvider {
            capability_id: CapabilityId::new("missing.dependency").unwrap(),
            min_version: 1,
        })
    );
}

#[test]
fn incompatible_version_returns_no_compatible_provider() {
    let manifest = parse(fixture("duplicate-provider-first.toml")).unwrap();
    let module_id = manifest.id.clone();
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            manifest.clone(),
            Box::new(EchoProvider::for_manifest(&manifest, "version-one")),
        )
        .unwrap();

    assert_eq!(
        registry.resolve(
            &installation("version-workspace", vec![module_id]),
            &requirement("practice.generate", 2),
        ),
        Err(RegistryError::NoCompatibleProvider {
            capability_id: CapabilityId::new("practice.generate").unwrap(),
            min_version: 2,
        })
    );
}

#[test]
fn duplicate_provider_resolution_follows_installation_order() {
    let first = parse(fixture("duplicate-provider-first.toml")).unwrap();
    let second = parse(fixture("duplicate-provider-second.toml")).unwrap();
    let first_id = first.id.clone();
    let second_id = second.id.clone();
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            first.clone(),
            Box::new(EchoProvider::for_manifest(&first, "first")),
        )
        .unwrap();
    registry
        .register(
            second.clone(),
            Box::new(EchoProvider::for_manifest(&second, "second")),
        )
        .unwrap();

    let first_installation = installation(
        "priority-workspace",
        vec![first_id.clone(), second_id.clone()],
    );
    let first_handle = registry
        .resolve(&first_installation, &requirement("practice.generate", 1))
        .unwrap();
    let second_installation = installation("priority-workspace", vec![second_id, first_id]);
    let second_handle = registry
        .resolve(&second_installation, &requirement("practice.generate", 1))
        .unwrap();

    assert_eq!(
        invoke_value(
            &registry,
            &first_installation,
            &first_handle,
            "practice.generate",
            1,
        )["provider"],
        "first"
    );
    assert_eq!(
        invoke_value(
            &registry,
            &second_installation,
            &second_handle,
            "practice.generate",
            1,
        )["provider"],
        "second"
    );
}

#[test]
fn disabled_module_is_skipped_during_resolution() {
    let manifest = parse(fixture("duplicate-provider-first.toml")).unwrap();
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            manifest.clone(),
            Box::new(EchoProvider::for_manifest(&manifest, "disabled")),
        )
        .unwrap();

    assert_eq!(
        registry.resolve(
            &installation("disabled-workspace", Vec::new()),
            &requirement("practice.generate", 1),
        ),
        Err(RegistryError::NoCompatibleProvider {
            capability_id: CapabilityId::new("practice.generate").unwrap(),
            min_version: 1,
        })
    );
}

#[test]
fn workspace_installations_isolate_enabled_providers() {
    let practice = parse(fixture("duplicate-provider-first.toml")).unwrap();
    let secondary = parse(fixture("workspace-secondary-provider.toml")).unwrap();
    let practice_id = practice.id.clone();
    let secondary_id = secondary.id.clone();
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            practice.clone(),
            Box::new(EchoProvider::for_manifest(&practice, "practice")),
        )
        .unwrap();
    registry
        .register(
            secondary.clone(),
            Box::new(EchoProvider::for_manifest(&secondary, "secondary")),
        )
        .unwrap();

    let practice_workspace = installation("workspace-a", vec![practice_id]);
    let secondary_workspace = installation("workspace-b", vec![secondary_id]);
    assert!(registry
        .resolve(&practice_workspace, &requirement("practice.generate", 1),)
        .is_ok());
    assert_eq!(
        registry.resolve(&secondary_workspace, &requirement("practice.generate", 1),),
        Err(RegistryError::NoCompatibleProvider {
            capability_id: CapabilityId::new("practice.generate").unwrap(),
            min_version: 1,
        })
    );
}

#[test]
fn invocation_failure_is_wrapped_with_registry_context() {
    let manifest = parse(fixture("duplicate-provider-first.toml")).unwrap();
    let module_id = manifest.id.clone();
    let capability_id = CapabilityId::new("practice.generate").unwrap();
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            manifest.clone(),
            Box::new(FailingProvider::for_manifest(
                &manifest,
                "deliberate fixture failure",
            )),
        )
        .unwrap();
    let installation = installation("failure-workspace", vec![module_id.clone()]);
    let handle = registry
        .resolve(&installation, &requirement("practice.generate", 1))
        .unwrap();

    let result: Result<serde_json::Value, RegistryError> =
        tauri::async_runtime::block_on(registry.invoke(
            &handle,
            &installation,
            call("failure-workspace", "practice.generate", 1),
        ));
    assert_eq!(
        result,
        Err(RegistryError::InvocationFailed {
            module_id,
            capability_id,
            cause: InvocationError::Failed {
                message: "deliberate fixture failure".to_owned(),
            },
        })
    );
}

#[test]
fn runtime_types_round_trip_through_json() {
    let manifest = parse(fixture("duplicate-provider-first.toml")).unwrap();
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    assert_eq!(
        serde_json::from_str::<ModuleManifest>(&manifest_json).unwrap(),
        manifest
    );

    let envelope = CallEnvelope {
        workspace_id: "serialization-workspace".to_owned(),
        capability_id: CapabilityId::new("practice.generate").unwrap(),
        version: 1,
        calling_module_id: ModuleId::new("org.axiom.test_caller").unwrap(),
    };
    let envelope_json = serde_json::to_string(&envelope).unwrap();
    assert_eq!(
        serde_json::from_str::<CallEnvelope>(&envelope_json).unwrap(),
        envelope
    );

    let call = CapabilityCall {
        envelope,
        input: SerializableInput {
            prompt: "generate one problem".to_owned(),
            count: 1,
        },
    };
    let call_json = serde_json::to_string(&call).unwrap();
    assert_eq!(
        serde_json::from_str::<CapabilityCall<SerializableInput>>(&call_json).unwrap(),
        call
    );
}

#[test]
fn embedded_source_parse_validate_register_isolates_rejected_manifest() {
    let discovered = EmbeddedManifestSource::new(INTEGRATION_MANIFESTS)
        .discover()
        .unwrap();
    assert_eq!(discovered.len(), 3);
    let mut registry = ModuleRegistry::new();
    let mut loaded = Vec::new();
    let mut rejected = Vec::new();

    for (source_id, raw_toml) in discovered {
        match parse(&raw_toml) {
            Ok(manifest) => {
                assert_eq!(manifest.id, source_id);
                let descriptor = manifest.provides[0].clone();
                registry
                    .register(
                        manifest.clone(),
                        Box::new(EchoProvider::for_manifest(&manifest, source_id.as_str())),
                    )
                    .unwrap();
                loaded.push((source_id, descriptor));
            }
            Err(error) => rejected.push((source_id, error)),
        }
    }

    assert_eq!(loaded.len(), 2);
    assert!(matches!(
        rejected.as_slice(),
        [(module_id, ManifestError::TomlSyntax(_))]
            if module_id == &ModuleId::new("org.axiom.integration_broken").unwrap()
    ));

    let installation = installation(
        "integration-workspace",
        loaded
            .iter()
            .map(|(module_id, _)| module_id.clone())
            .collect(),
    );
    for (_, capability) in loaded {
        assert!(registry
            .resolve(
                &installation,
                &CapabilityRequirement {
                    id: capability.id,
                    min_version: capability.version,
                },
            )
            .is_ok());
    }
}
