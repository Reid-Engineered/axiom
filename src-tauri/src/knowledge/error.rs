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
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    MissingPackageToml {
        path: PathBuf,
    },
    MissingSourcesToml {
        path: PathBuf,
    },
    UnexpectedEntityFile {
        path: PathBuf,
    },
    NestedEntityDirectory {
        path: PathBuf,
    },
    FilenameIdMismatch {
        path: PathBuf,
        filename_id: String,
        frontmatter_id: String,
    },
    UnresolvedConcept {
        entity_id: String,
        field: &'static str,
        target: String,
    },
    UnresolvedObjective {
        entity_id: String,
        field: &'static str,
        target: String,
    },
    UnresolvedSource {
        entity_id: String,
        target: String,
    },
    CrossConceptObjective {
        example_id: String,
        objective_id: String,
    },
    SelfReference {
        entity_id: String,
        field: &'static str,
    },
    DuplicateReferenceInList {
        entity_id: String,
        field: &'static str,
        target: String,
    },
    PrerequisiteCycle {
        cycle: Vec<String>,
    },
    ReverseDuplicateRelated {
        first: String,
        second: String,
    },
    ConstraintParseError {
        entity_id: String,
        constraint: String,
        message: String,
    },
    MissingProblemFamilySection {
        entity_id: String,
        section: &'static str,
    },
    DuplicateProblemFamilySection {
        entity_id: String,
        section: &'static str,
    },
    OutOfOrderProblemFamilySection {
        entity_id: String,
        section: &'static str,
    },
    UnknownProblemFamilySection {
        entity_id: String,
        heading: String,
    },
    ContentBeforePrompt {
        entity_id: String,
    },
    InvalidProblemFamilyHintLine {
        entity_id: String,
        line: String,
    },
    ProblemFamilyHintCountMismatch {
        entity_id: String,
        frontmatter_count: usize,
        body_count: usize,
    },
    UnknownParameterType {
        entity_id: String,
        value: String,
    },
    DanglingParameterReference {
        entity_id: String,
        parameter: String,
        target: String,
    },
    ParameterReferenceCycle {
        entity_id: String,
        cycle: Vec<String>,
    },
    ParameterValueAndBoundsConflict {
        entity_id: String,
        parameter: String,
    },
    ConstraintUnknownParameter {
        entity_id: String,
        parameter: String,
    },
    UnknownResponseType {
        entity_id: String,
        value: String,
    },
    ResponseTypeSolutionMismatch {
        entity_id: String,
        response_type: &'static str,
    },
    InvalidDifficultyRange {
        entity_id: String,
        min: u8,
        max: u8,
    },
    DuplicateHintLevel {
        entity_id: String,
        level: u32,
    },
    UnknownProblemFamilyStatus {
        entity_id: String,
        value: String,
    },
    ProblemFamilyCrossConceptObjective {
        problem_family_id: String,
        objective_id: String,
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
            Self::Io { path, source } => write!(f, "I/O error reading {}: {source}", path.display()),
            Self::MissingPackageToml { path } => write!(f, "{} is missing package.toml", path.display()),
            Self::MissingSourcesToml { path } => write!(f, "{} is missing sources.toml", path.display()),
            Self::UnexpectedEntityFile { path } => write!(f, "{} is not a recognized entity file (expected <id>.md)", path.display()),
            Self::NestedEntityDirectory { path } => write!(f, "{} is a nested directory, not permitted inside an entity directory", path.display()),
            Self::FilenameIdMismatch { path, filename_id, frontmatter_id } => write!(
                f,
                "{}: filename implies id \"{filename_id}\" but frontmatter declares \"{frontmatter_id}\"",
                path.display()
            ),
            Self::UnresolvedConcept { entity_id, field, target } => write!(f, "{entity_id}.{field} references unknown Concept: {target}"),
            Self::UnresolvedObjective { entity_id, field, target } => write!(f, "{entity_id}.{field} references unknown Objective: {target}"),
            Self::UnresolvedSource { entity_id, target } => write!(f, "{entity_id} references unknown Source: {target}"),
            Self::CrossConceptObjective { example_id, objective_id } => write!(
                f,
                "example {example_id} references objective {objective_id} belonging to a different concept"
            ),
            Self::SelfReference { entity_id, field } => write!(f, "{entity_id}.{field} references itself"),
            Self::DuplicateReferenceInList { entity_id, field, target } => write!(f, "{entity_id}.{field} lists {target} more than once"),
            Self::PrerequisiteCycle { cycle } => write!(f, "prerequisite cycle detected: {}", cycle.join(" -> ")),
            Self::ReverseDuplicateRelated { first, second } => write!(
                f,
                "related_ids declared on both {first} and {second}; author it on exactly one side"
            ),
            Self::ConstraintParseError { entity_id, constraint, message } => write!(f, "{entity_id}: constraint \"{constraint}\" failed to parse: {message}"),
            Self::MissingProblemFamilySection { entity_id, section } => write!(f, "problem family {entity_id} is missing required section ## {section}"),
            Self::DuplicateProblemFamilySection { entity_id, section } => write!(f, "problem family {entity_id} declares ## {section} more than once"),
            Self::OutOfOrderProblemFamilySection { entity_id, section } => write!(f, "problem family {entity_id}: ## {section} appears out of order"),
            Self::UnknownProblemFamilySection { entity_id, heading } => write!(f, "problem family {entity_id} contains unrecognized heading: {heading}"),
            Self::ContentBeforePrompt { entity_id } => write!(f, "problem family {entity_id} has content before Prompt"),
            Self::InvalidProblemFamilyHintLine { entity_id, line } => write!(f, "problem family {entity_id}: invalid hint line: {line}"),
            Self::ProblemFamilyHintCountMismatch { entity_id, frontmatter_count, body_count } => write!(f, "problem family {entity_id} declares {frontmatter_count} hints but body has {body_count}"),
            Self::UnknownParameterType { entity_id, value } => write!(f, "{entity_id} declares unknown parameter type: {value}"),
            Self::DanglingParameterReference { entity_id, parameter, target } => write!(f, "{entity_id}.parameters.{parameter} references undeclared parameter: {target}"),
            Self::ParameterReferenceCycle { entity_id, cycle } => write!(f, "{entity_id} has a parameter-reference cycle: {}", cycle.join(" -> ")),
            Self::ParameterValueAndBoundsConflict { entity_id, parameter } => write!(f, "{entity_id}.parameters.{parameter} declares both a fixed value and bounds"),
            Self::ConstraintUnknownParameter { entity_id, parameter } => write!(f, "{entity_id} has a constraint referencing undeclared parameter: {parameter}"),
            Self::UnknownResponseType { entity_id, value } => write!(f, "{entity_id} declares unknown response_type: {value}"),
            Self::ResponseTypeSolutionMismatch { entity_id, response_type } => write!(f, "{entity_id}'s canonical_solution does not match response_type {response_type}"),
            Self::InvalidDifficultyRange { entity_id, min, max } => write!(f, "{entity_id} has invalid difficulty range: {min} > {max}"),
            Self::DuplicateHintLevel { entity_id, level } => write!(f, "{entity_id} declares hint level {level} more than once"),
            Self::UnknownProblemFamilyStatus { entity_id, value } => write!(f, "{entity_id} declares unknown status: {value}"),
            Self::ProblemFamilyCrossConceptObjective { problem_family_id, objective_id } => write!(f, "problem family {problem_family_id} references objective {objective_id} belonging to a different concept"),
        }
    }
}

impl Error for KnowledgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TomlSyntax { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
