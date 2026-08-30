use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::{CapabilityId, CapabilityRequirement, ModuleId, ModuleManifest};

#[async_trait::async_trait]
pub trait CapabilityProvider: Send + Sync {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, InvocationError>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityCall<Input> {
    pub envelope: CallEnvelope,
    pub input: Input,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallEnvelope {
    pub workspace_id: String,
    pub capability_id: CapabilityId,
    pub version: u32,
    pub calling_module_id: ModuleId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityHandle {
    module_id: ModuleId,
    capability_id: CapabilityId,
    version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleInstallation {
    pub workspace_id: String,
    pub enabled_module_ids: Vec<ModuleId>,
}

struct RegisteredModule {
    manifest: ModuleManifest,
    provider: Box<dyn CapabilityProvider>,
}

pub struct ModuleRegistry {
    modules: HashMap<ModuleId, RegisteredModule>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        manifest: ModuleManifest,
        provider: Box<dyn CapabilityProvider>,
    ) -> Result<ModuleId, RegistryError> {
        let module_id = manifest.id.clone();
        if self.modules.contains_key(&module_id) {
            return Err(RegistryError::DuplicateModuleId(module_id));
        }

        self.modules
            .insert(module_id.clone(), RegisteredModule { manifest, provider });
        Ok(module_id)
    }

    pub fn resolve(
        &self,
        installation: &ModuleInstallation,
        requirement: &CapabilityRequirement,
    ) -> Result<CapabilityHandle, RegistryError> {
        for module_id in &installation.enabled_module_ids {
            let Some(module) = self.modules.get(module_id) else {
                continue;
            };
            let Some(capability) = module.manifest.provides.iter().find(|capability| {
                capability.id == requirement.id && capability.version >= requirement.min_version
            }) else {
                continue;
            };

            return Ok(CapabilityHandle {
                module_id: module_id.clone(),
                capability_id: capability.id.clone(),
                version: capability.version,
            });
        }

        Err(RegistryError::NoCompatibleProvider {
            capability_id: requirement.id.clone(),
            min_version: requirement.min_version,
        })
    }

    pub async fn invoke<Input: Serialize, Output: DeserializeOwned>(
        &self,
        handle: &CapabilityHandle,
        installation: &ModuleInstallation,
        call: CapabilityCall<Input>,
    ) -> Result<Output, RegistryError> {
        if !installation.enabled_module_ids.contains(&handle.module_id) {
            return Err(RegistryError::ModuleDisabled(handle.module_id.clone()));
        }

        let Some(module) = self.modules.get(&handle.module_id) else {
            return Err(RegistryError::NoCompatibleProvider {
                capability_id: handle.capability_id.clone(),
                min_version: handle.version,
            });
        };
        let input = serde_json::to_value(call.input).map_err(|error| {
            self.invocation_failed(
                handle,
                InvocationError::InvalidInput {
                    capability_id: handle.capability_id.clone(),
                    message: error.to_string(),
                },
            )
        })?;
        let output = module
            .provider
            .invoke(&handle.capability_id, handle.version, input)
            .await
            .map_err(|cause| self.invocation_failed(handle, cause))?;

        serde_json::from_value(output).map_err(|error| {
            self.invocation_failed(
                handle,
                InvocationError::Failed {
                    message: format!("provider returned invalid output: {error}"),
                },
            )
        })
    }

    fn invocation_failed(
        &self,
        handle: &CapabilityHandle,
        cause: InvocationError,
    ) -> RegistryError {
        RegistryError::InvocationFailed {
            module_id: handle.module_id.clone(),
            capability_id: handle.capability_id.clone(),
            cause,
        }
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateModuleId(ModuleId),
    NoCompatibleProvider {
        capability_id: CapabilityId,
        min_version: u32,
    },
    ModuleDisabled(ModuleId),
    InvocationFailed {
        module_id: ModuleId,
        capability_id: CapabilityId,
        cause: InvocationError,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvocationError {
    UnknownCapability {
        capability_id: CapabilityId,
        version: u32,
    },
    InvalidInput {
        capability_id: CapabilityId,
        message: String,
    },
    Failed {
        message: String,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateModuleId(module_id) => {
                write!(formatter, "module {module_id} is already registered")
            }
            Self::NoCompatibleProvider {
                capability_id,
                min_version,
            } => write!(
                formatter,
                "no enabled module provides {capability_id} at version {min_version} or newer"
            ),
            Self::ModuleDisabled(module_id) => {
                write!(formatter, "module {module_id} is disabled")
            }
            Self::InvocationFailed {
                module_id,
                capability_id,
                cause,
            } => write!(
                formatter,
                "module {module_id} failed to invoke {capability_id}: {cause}"
            ),
        }
    }
}

impl Error for RegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvocationFailed { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

impl fmt::Display for InvocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCapability {
                capability_id,
                version,
            } => write!(
                formatter,
                "provider does not implement {capability_id} at version {version}"
            ),
            Self::InvalidInput {
                capability_id,
                message,
            } => write!(formatter, "invalid input for {capability_id}: {message}"),
            Self::Failed { message } => formatter.write_str(message),
        }
    }
}

impl Error for InvocationError {}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use serde::Deserialize;
    use serde_json::json;

    use super::*;
    use crate::modules::{axiom_version, CapabilityDescriptor, OfflineCapability};

    struct EchoProvider {
        label: &'static str,
    }

    #[async_trait::async_trait]
    impl CapabilityProvider for EchoProvider {
        async fn invoke(
            &self,
            capability_id: &CapabilityId,
            version: u32,
            input: serde_json::Value,
        ) -> Result<serde_json::Value, InvocationError> {
            let text = input
                .get("text")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| InvocationError::InvalidInput {
                    capability_id: capability_id.clone(),
                    message: "text must be a string".to_owned(),
                })?;

            Ok(json!({
                "echo": text,
                "provider": self.label,
                "version": version,
            }))
        }
    }

    struct FailingProvider;

    #[async_trait::async_trait]
    impl CapabilityProvider for FailingProvider {
        async fn invoke(
            &self,
            _capability_id: &CapabilityId,
            _version: u32,
            _input: serde_json::Value,
        ) -> Result<serde_json::Value, InvocationError> {
            Err(InvocationError::Failed {
                message: "fixture failure".to_owned(),
            })
        }
    }

    #[derive(Serialize)]
    struct EchoInput<'a> {
        text: &'a str,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct EchoOutput {
        echo: String,
        provider: String,
        version: u32,
    }

    fn module_id(value: &str) -> ModuleId {
        ModuleId::new(value).unwrap()
    }

    fn capability_id() -> CapabilityId {
        CapabilityId::new("fixture.echo").unwrap()
    }

    fn manifest(id: &str, capability_version: u32) -> ModuleManifest {
        ModuleManifest {
            id: module_id(id),
            name: id.to_owned(),
            version: Version::new(1, 0, 0),
            minimum_axiom_version: axiom_version(),
            offline: OfflineCapability::Full,
            provides: vec![CapabilityDescriptor {
                id: capability_id(),
                version: capability_version,
            }],
            requires: Vec::new(),
        }
    }

    fn installation(enabled_module_ids: Vec<ModuleId>) -> ModuleInstallation {
        ModuleInstallation {
            workspace_id: "workspace-1".to_owned(),
            enabled_module_ids,
        }
    }

    fn requirement(min_version: u32) -> CapabilityRequirement {
        CapabilityRequirement {
            id: capability_id(),
            min_version,
        }
    }

    fn call<Input>(input: Input) -> CapabilityCall<Input> {
        CapabilityCall {
            envelope: CallEnvelope {
                workspace_id: "workspace-1".to_owned(),
                capability_id: capability_id(),
                version: 1,
                calling_module_id: module_id("org.axiom.caller"),
            },
            input,
        }
    }

    fn invoke_echo(
        registry: &ModuleRegistry,
        installation: &ModuleInstallation,
        handle: &CapabilityHandle,
    ) -> EchoOutput {
        tauri::async_runtime::block_on(registry.invoke(
            handle,
            installation,
            call(EchoInput { text: "hello" }),
        ))
        .unwrap()
    }

    #[test]
    fn rejected_duplicate_does_not_block_later_registration() {
        let first_id = module_id("org.axiom.first");
        let second_id = module_id("org.axiom.second");
        let mut registry = ModuleRegistry::new();

        assert_eq!(
            registry.register(
                manifest(first_id.as_str(), 1),
                Box::new(EchoProvider { label: "first" }),
            ),
            Ok(first_id.clone())
        );
        assert_eq!(
            registry.register(
                manifest(first_id.as_str(), 2),
                Box::new(EchoProvider { label: "duplicate" }),
            ),
            Err(RegistryError::DuplicateModuleId(first_id))
        );
        assert_eq!(
            registry.register(
                manifest(second_id.as_str(), 1),
                Box::new(EchoProvider { label: "second" }),
            ),
            Ok(second_id)
        );
    }

    #[test]
    fn resolution_uses_enabled_module_priority() {
        let first_id = module_id("org.axiom.first");
        let second_id = module_id("org.axiom.second");
        let mut registry = ModuleRegistry::new();
        registry
            .register(
                manifest(first_id.as_str(), 1),
                Box::new(EchoProvider { label: "first" }),
            )
            .unwrap();
        registry
            .register(
                manifest(second_id.as_str(), 2),
                Box::new(EchoProvider { label: "second" }),
            )
            .unwrap();

        let first_installation = installation(vec![first_id.clone(), second_id.clone()]);
        let first_handle = registry
            .resolve(&first_installation, &requirement(1))
            .unwrap();
        let second_installation = installation(vec![second_id, first_id]);
        let second_handle = registry
            .resolve(&second_installation, &requirement(1))
            .unwrap();

        assert_eq!(
            invoke_echo(&registry, &first_installation, &first_handle),
            EchoOutput {
                echo: "hello".to_owned(),
                provider: "first".to_owned(),
                version: 1,
            }
        );
        assert_eq!(
            invoke_echo(&registry, &second_installation, &second_handle),
            EchoOutput {
                echo: "hello".to_owned(),
                provider: "second".to_owned(),
                version: 2,
            }
        );
    }

    #[test]
    fn resolution_rejects_disabled_and_incompatible_providers() {
        let provider_id = module_id("org.axiom.provider");
        let mut registry = ModuleRegistry::new();
        registry
            .register(
                manifest(provider_id.as_str(), 1),
                Box::new(EchoProvider { label: "provider" }),
            )
            .unwrap();

        assert_eq!(
            registry.resolve(&installation(Vec::new()), &requirement(1)),
            Err(RegistryError::NoCompatibleProvider {
                capability_id: capability_id(),
                min_version: 1,
            })
        );
        assert_eq!(
            registry.resolve(&installation(vec![provider_id]), &requirement(2)),
            Err(RegistryError::NoCompatibleProvider {
                capability_id: capability_id(),
                min_version: 2,
            })
        );
    }

    #[test]
    fn typed_invocation_dispatches_through_json_boundary() {
        let provider_id = module_id("org.axiom.provider");
        let mut registry = ModuleRegistry::new();
        registry
            .register(
                manifest(provider_id.as_str(), 3),
                Box::new(EchoProvider { label: "provider" }),
            )
            .unwrap();
        let installation = installation(vec![provider_id]);
        let handle = registry.resolve(&installation, &requirement(1)).unwrap();

        assert_eq!(
            invoke_echo(&registry, &installation, &handle),
            EchoOutput {
                echo: "hello".to_owned(),
                provider: "provider".to_owned(),
                version: 3,
            }
        );
    }

    #[test]
    fn invocation_rechecks_enablement_and_wraps_provider_errors() {
        let provider_id = module_id("org.axiom.provider");
        let mut registry = ModuleRegistry::new();
        registry
            .register(manifest(provider_id.as_str(), 1), Box::new(FailingProvider))
            .unwrap();
        let enabled = installation(vec![provider_id.clone()]);
        let handle = registry.resolve(&enabled, &requirement(1)).unwrap();

        let disabled_result: Result<serde_json::Value, RegistryError> =
            tauri::async_runtime::block_on(registry.invoke(
                &handle,
                &installation(Vec::new()),
                call(json!({})),
            ));
        assert_eq!(
            disabled_result,
            Err(RegistryError::ModuleDisabled(provider_id.clone()))
        );

        let failure_result: Result<serde_json::Value, RegistryError> =
            tauri::async_runtime::block_on(registry.invoke(&handle, &enabled, call(json!({}))));
        assert_eq!(
            failure_result,
            Err(RegistryError::InvocationFailed {
                module_id: provider_id,
                capability_id: capability_id(),
                cause: InvocationError::Failed {
                    message: "fixture failure".to_owned(),
                },
            })
        );
    }
}
