use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum KnowledgeError {
    InvalidIdentifier {
        value: String,
    },
    TomlSyntax {
        path: PathBuf,
        source: toml::de::Error,
    },
    UnsupportedSchemaVersion {
        found: u32,
    },
    MalformedVersion {
        field: &'static str,
        value: String,
    },
    EmptyField {
        path: PathBuf,
        field: &'static str,
    },
    DuplicateSourceId {
        id: String,
    },
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { value } => write!(f, "invalid Knowledge identifier: {value}"),
            Self::TomlSyntax { path, source } => {
                write!(f, "invalid TOML in {}: {source}", path.display())
            }
            Self::UnsupportedSchemaVersion { found } => {
                write!(f, "unsupported schema_version {found}; only 1 is accepted")
            }
            Self::MalformedVersion { field, value } => {
                write!(f, "{field} is not a semantic version: {value}")
            }
            Self::EmptyField { path, field } => {
                write!(f, "{} has an empty required field: {field}", path.display())
            }
            Self::DuplicateSourceId { id } => write!(f, "duplicate Source id: {id}"),
        }
    }
}

impl Error for KnowledgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TomlSyntax { source, .. } => Some(source),
            _ => None,
        }
    }
}
