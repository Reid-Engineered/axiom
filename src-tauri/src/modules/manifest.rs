use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use semver::Version;
use serde::{Deserialize, Serialize};

use super::axiom_version;
use super::identifier::{CapabilityId, ModuleId};

const SUPPORTED_MANIFEST_VERSIONS: &[u32] = &[1];

#[derive(Debug, Clone, Deserialize)]
pub struct RawModuleManifest {
    pub manifest_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub minimum_axiom_version: String,
    pub offline: String,
    #[serde(default)]
    pub provides: Vec<RawCapabilityDescriptor>,
    #[serde(default)]
    pub requires: Vec<RawCapabilityRequirement>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCapabilityDescriptor {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCapabilityRequirement {
    pub id: String,
    pub min_version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub id: ModuleId,
    pub name: String,
    pub version: Version,
    pub minimum_axiom_version: Version,
    pub offline: OfflineCapability,
    pub provides: Vec<CapabilityDescriptor>,
    pub requires: Vec<CapabilityRequirement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub id: CapabilityId,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub id: CapabilityId,
    pub min_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OfflineCapability {
    Full,
    Enhanced,
    Required,
}

#[derive(Debug)]
pub enum ManifestError {
    UnsupportedManifestVersion {
        found: u32,
        supported: &'static [u32],
    },
    MissingModuleId,
    MalformedVersion {
        field: &'static str,
        value: String,
    },
    InvalidIdentifier {
        value: String,
    },
    DuplicateCapability {
        module_id: String,
        capability_id: String,
    },
    IncompatibleAxiomVersion {
        required: Version,
        running: Version,
    },
    TomlSyntax(toml::de::Error),
}

pub fn parse(raw_toml: &str) -> Result<ModuleManifest, ManifestError> {
    let raw = toml::from_str(raw_toml).map_err(ManifestError::TomlSyntax)?;
    validate(raw)
}

pub fn validate(raw: RawModuleManifest) -> Result<ModuleManifest, ManifestError> {
    if !SUPPORTED_MANIFEST_VERSIONS.contains(&raw.manifest_version) {
        return Err(ManifestError::UnsupportedManifestVersion {
            found: raw.manifest_version,
            supported: SUPPORTED_MANIFEST_VERSIONS,
        });
    }

    if raw.id.is_empty() {
        return Err(ManifestError::MissingModuleId);
    }

    let module_id_text = raw.id;
    let module_id = ModuleId::new(module_id_text.clone())?;
    let version = parse_version("version", raw.version)?;
    let minimum_axiom_version = parse_version("minimum_axiom_version", raw.minimum_axiom_version)?;
    let running_axiom_version = axiom_version();
    if minimum_axiom_version > running_axiom_version {
        return Err(ManifestError::IncompatibleAxiomVersion {
            required: minimum_axiom_version,
            running: running_axiom_version,
        });
    }
    let offline = toml::Value::String(raw.offline)
        .try_into()
        .map_err(ManifestError::TomlSyntax)?;

    let mut seen_capabilities = HashSet::new();
    let mut provides = Vec::with_capacity(raw.provides.len());
    for raw_capability in raw.provides {
        let capability_id_text = raw_capability.id;
        let capability_id = CapabilityId::new(capability_id_text.clone())?;
        if !seen_capabilities.insert((capability_id.clone(), raw_capability.version)) {
            return Err(ManifestError::DuplicateCapability {
                module_id: module_id_text,
                capability_id: capability_id_text,
            });
        }
        provides.push(CapabilityDescriptor {
            id: capability_id,
            version: raw_capability.version,
        });
    }

    let requires = raw
        .requires
        .into_iter()
        .map(|requirement| {
            Ok(CapabilityRequirement {
                id: CapabilityId::new(requirement.id)?,
                min_version: requirement.min_version,
            })
        })
        .collect::<Result<_, ManifestError>>()?;

    Ok(ModuleManifest {
        id: module_id,
        name: raw.name,
        version,
        minimum_axiom_version,
        offline,
        provides,
        requires,
    })
}

fn parse_version(field: &'static str, value: String) -> Result<Version, ManifestError> {
    Version::parse(&value).map_err(|_| ManifestError::MalformedVersion { field, value })
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedManifestVersion { found, supported } => write!(
                formatter,
                "unsupported manifest version {found}; supported versions: {supported:?}"
            ),
            Self::MissingModuleId => formatter.write_str("module id is empty"),
            Self::MalformedVersion { field, value } => {
                write!(formatter, "{field} is not a semantic version: {value}")
            }
            Self::InvalidIdentifier { value } => {
                write!(formatter, "invalid module or capability identifier: {value}")
            }
            Self::DuplicateCapability {
                module_id,
                capability_id,
            } => write!(
                formatter,
                "module {module_id} declares capability {capability_id} more than once at the same version"
            ),
            Self::IncompatibleAxiomVersion { required, running } => write!(
                formatter,
                "module requires Axiom {required}, but this build is {running}"
            ),
            Self::TomlSyntax(error) => write!(formatter, "invalid module manifest TOML: {error}"),
        }
    }
}

impl Error for ManifestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TomlSyntax(error) => Some(error),
            _ => None,
        }
    }
}

impl From<toml::de::Error> for ManifestError {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlSyntax(error)
    }
}
