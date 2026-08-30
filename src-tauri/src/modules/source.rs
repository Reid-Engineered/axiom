use super::{ManifestError, ModuleId};

const EMBEDDED_MANIFESTS: &[(&str, &str)] = &[(
    "org.axiom.test_fixture",
    include_str!("tests/fixtures/valid.toml"),
)];

pub trait ManifestSource {
    fn discover(&self) -> Result<Vec<(ModuleId, String)>, ManifestError>;
}

#[derive(Debug, Clone)]
pub struct EmbeddedManifestSource {
    manifests: &'static [(&'static str, &'static str)],
}

impl EmbeddedManifestSource {
    pub const fn new(manifests: &'static [(&'static str, &'static str)]) -> Self {
        Self { manifests }
    }
}

impl Default for EmbeddedManifestSource {
    fn default() -> Self {
        Self::new(EMBEDDED_MANIFESTS)
    }
}

impl ManifestSource for EmbeddedManifestSource {
    fn discover(&self) -> Result<Vec<(ModuleId, String)>, ManifestError> {
        self.manifests
            .iter()
            .map(|(id, raw_toml)| Ok((ModuleId::new(*id)?, (*raw_toml).to_owned())))
            .collect()
    }
}
