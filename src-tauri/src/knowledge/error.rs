use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum KnowledgeError {
    InvalidIdentifier {
        value: String,
    },
    Bom {
        path: PathBuf,
    },
    MissingFrontmatterDelimiter {
        path: PathBuf,
    },
    UnterminatedFrontmatter {
        path: PathBuf,
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
    MissingProvenance {
        entity_id: String,
    },
    DuplicateProvenanceRef {
        entity_id: String,
        source_id: String,
    },
    UnknownProvenanceKind {
        entity_id: String,
        value: String,
    },
    EmptySourceLocator {
        entity_id: String,
    },
    MissingExampleSection {
        entity_id: String,
        section: &'static str,
    },
    DuplicateExampleSection {
        entity_id: String,
        section: &'static str,
    },
    OutOfOrderExampleSection {
        entity_id: String,
        section: &'static str,
    },
    UnknownExampleSection {
        entity_id: String,
        heading: String,
    },
    ContentBeforeProblem {
        entity_id: String,
    },
    InvalidHintLine {
        entity_id: String,
        line: String,
    },
    EmptyHintsSection {
        entity_id: String,
    },
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { value } => write!(f, "invalid Knowledge identifier: {value}"),
            Self::Bom { path } => write!(
                f,
                "{} starts with a byte-order mark, which is rejected",
                path.display()
            ),
            Self::MissingFrontmatterDelimiter { path } => write!(
                f,
                "{} does not start with the '+++' frontmatter delimiter",
                path.display()
            ),
            Self::UnterminatedFrontmatter { path } => write!(
                f,
                "{} opens frontmatter with '+++' but never closes it",
                path.display()
            ),
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
            Self::MissingProvenance { entity_id } => write!(
                f,
                "{entity_id} has no provenance_refs; at least one is required"
            ),
            Self::DuplicateProvenanceRef {
                entity_id,
                source_id,
            } => write!(
                f,
                "{entity_id} declares a duplicate provenance reference to {source_id}"
            ),
            Self::UnknownProvenanceKind { entity_id, value } => {
                write!(f, "{entity_id} declares unknown ProvenanceKind: {value}")
            }
            Self::EmptySourceLocator { entity_id } => write!(
                f,
                "{entity_id} declares a provenance locator with no section, pages, or label"
            ),
            Self::MissingExampleSection { entity_id, section } => write!(f, "example {entity_id} is missing required section ## {section}"),
            Self::DuplicateExampleSection { entity_id, section } => write!(f, "example {entity_id} declares ## {section} more than once"),
            Self::OutOfOrderExampleSection { entity_id, section } => write!(f, "example {entity_id}: ## {section} appears out of the required Problem/Solution/Hints order"),
            Self::UnknownExampleSection { entity_id, heading } => write!(f, "example {entity_id} contains unrecognized heading: {heading}"),
            Self::ContentBeforeProblem { entity_id } => write!(f, "example {entity_id} has non-whitespace content before ## Problem"),
            Self::InvalidHintLine { entity_id, line } => write!(f, "example {entity_id}: invalid line under ## Hints (expected \"- <hint>\"): {line}"),
            Self::EmptyHintsSection { entity_id } => write!(f, "example {entity_id} declares ## Hints with no hint items"),
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
