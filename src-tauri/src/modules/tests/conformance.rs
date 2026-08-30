use crate::modules::{
    CapabilityId, CapabilityProvider, CapabilityRequirement, InvocationError, ModuleInstallation,
    ModuleManifest, ModuleRegistry,
};

use super::fixture;
use super::providers::{invocation_log, EchoProvider};

pub(super) fn assert_provider_conforms(
    manifest: ModuleManifest,
    provider: Box<dyn CapabilityProvider>,
) {
    let declared_capabilities = manifest.provides.clone();
    let undeclared_capability = undeclared_capability_id(&manifest);

    tauri::async_runtime::block_on(async {
        for capability in &declared_capabilities {
            match provider
                .invoke(&capability.id, capability.version, serde_json::Value::Null)
                .await
            {
                Ok(_) => {}
                Err(InvocationError::InvalidInput { capability_id, .. }) => {
                    assert_eq!(capability_id, capability.id);
                }
                Err(error) => panic!(
                    "provider does not conform to declared capability {}@{}: {error}",
                    capability.id, capability.version
                ),
            }
        }

        assert!(matches!(
            provider
                .invoke(&undeclared_capability, 1, serde_json::Value::Null)
                .await,
            Err(InvocationError::UnknownCapability {
                capability_id,
                version: 1,
            }) if capability_id == undeclared_capability
        ));
    });

    let module_id = manifest.id.clone();
    let mut registry = ModuleRegistry::new();
    registry.register(manifest, provider).unwrap();
    let installation = ModuleInstallation {
        workspace_id: "conformance-workspace".to_owned(),
        enabled_module_ids: vec![module_id],
    };

    for capability in declared_capabilities {
        registry
            .resolve(
                &installation,
                &CapabilityRequirement {
                    id: capability.id,
                    min_version: capability.version,
                },
            )
            .unwrap();
    }
}

fn undeclared_capability_id(manifest: &ModuleManifest) -> CapabilityId {
    let mut suffix = 0;
    loop {
        let candidate = if suffix == 0 {
            "conformance.undeclared".to_owned()
        } else {
            format!("conformance.undeclared_{suffix}")
        };
        let candidate = CapabilityId::new(candidate).unwrap();
        if manifest
            .provides
            .iter()
            .all(|capability| capability.id != candidate)
        {
            return candidate;
        }
        suffix += 1;
    }
}

#[test]
fn generic_conformance_harness_checks_declared_and_undeclared_capabilities() {
    let manifest =
        crate::modules::parse(fixture("conformance-multiple-capabilities.toml")).unwrap();
    let declared = manifest.provides.clone();
    let invocation_log = invocation_log();
    let provider = Box::new(EchoProvider::with_log(
        &manifest,
        "conforming-provider",
        invocation_log.clone(),
    ));

    assert_provider_conforms(manifest, provider);

    let invocations = invocation_log
        .lock()
        .expect("fixture invocation log lock must not be poisoned");
    for capability in declared {
        assert!(invocations.contains(&(capability.id, capability.version)));
    }
    assert!(invocations.iter().any(|(capability_id, _)| {
        capability_id.as_str().starts_with("conformance.undeclared")
    }));
}
