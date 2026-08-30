pub(crate) mod identifier;
mod manifest;
mod registry;
mod source;

pub use identifier::{CapabilityId, ModuleId};
pub use manifest::{
    parse, validate, CapabilityDescriptor, CapabilityRequirement, ManifestError, ModuleManifest,
    OfflineCapability, RawCapabilityDescriptor, RawCapabilityRequirement, RawModuleManifest,
};
pub use registry::{
    CallEnvelope, CapabilityCall, CapabilityHandle, CapabilityProvider, InvocationError,
    ModuleInstallation, ModuleRegistry, RegistryError,
};
pub use source::{EmbeddedManifestSource, ManifestSource};

/// Returns the running Axiom package version used for manifest compatibility checks.
pub fn axiom_version() -> semver::Version {
    semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("CARGO_PKG_VERSION must contain valid semantic versioning")
}

#[cfg(test)]
mod tests;
