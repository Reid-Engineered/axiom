use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::modules::{
    CapabilityDescriptor, CapabilityId, CapabilityProvider, InvocationError, ModuleManifest,
};

pub(super) type InvocationLog = Arc<Mutex<Vec<(CapabilityId, u32)>>>;

/// Test-only provider that echoes calls for capabilities declared by its fixture manifest.
pub(super) struct EchoProvider {
    label: String,
    capabilities: Vec<CapabilityDescriptor>,
    invocation_log: Option<InvocationLog>,
}

impl EchoProvider {
    pub(super) fn for_manifest(manifest: &ModuleManifest, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            capabilities: manifest.provides.clone(),
            invocation_log: None,
        }
    }

    pub(super) fn with_log(
        manifest: &ModuleManifest,
        label: impl Into<String>,
        invocation_log: InvocationLog,
    ) -> Self {
        Self {
            label: label.into(),
            capabilities: manifest.provides.clone(),
            invocation_log: Some(invocation_log),
        }
    }

    fn supports(&self, capability_id: &CapabilityId, version: u32) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability.id == *capability_id && capability.version == version)
    }
}

#[async_trait::async_trait]
impl CapabilityProvider for EchoProvider {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, InvocationError> {
        if let Some(invocation_log) = &self.invocation_log {
            invocation_log
                .lock()
                .expect("fixture invocation log lock must not be poisoned")
                .push((capability_id.clone(), version));
        }

        if !self.supports(capability_id, version) {
            return Err(InvocationError::UnknownCapability {
                capability_id: capability_id.clone(),
                version,
            });
        }

        Ok(json!({
            "capabilityId": capability_id.as_str(),
            "input": input,
            "provider": self.label,
            "version": version,
        }))
    }
}

/// Test-only provider that deliberately fails every declared capability invocation.
pub(super) struct FailingProvider {
    capabilities: Vec<CapabilityDescriptor>,
    message: String,
}

impl FailingProvider {
    pub(super) fn for_manifest(manifest: &ModuleManifest, message: impl Into<String>) -> Self {
        Self {
            capabilities: manifest.provides.clone(),
            message: message.into(),
        }
    }

    fn supports(&self, capability_id: &CapabilityId, version: u32) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability.id == *capability_id && capability.version == version)
    }
}

#[async_trait::async_trait]
impl CapabilityProvider for FailingProvider {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        _input: serde_json::Value,
    ) -> Result<serde_json::Value, InvocationError> {
        if !self.supports(capability_id, version) {
            return Err(InvocationError::UnknownCapability {
                capability_id: capability_id.clone(),
                version,
            });
        }

        Err(InvocationError::Failed {
            message: self.message.clone(),
        })
    }
}

pub(super) fn invocation_log() -> InvocationLog {
    Arc::new(Mutex::new(Vec::new()))
}
