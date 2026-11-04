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
    MissingProblemFamilyObjectives {
        entity_id: String,
    },
    NonFiniteParameterBound {
        entity_id: String,
        parameter: String,
        field: &'static str,
    },
    InvalidHintLevel {
        entity_id: String,
        level: u32,
    },
    OutOfOrderHintLevel {
        entity_id: String,
        previous_level: u32,
        level: u32,
    },
    InvalidParameterName {
        entity_id: String,
        parameter: String,
    },
    NonFiniteCanonicalSolution {
        entity_id: String,
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
    PatternsDirectoryInSchemaV1 {
        path: PathBuf,
    },
    UnknownBloomLevel {
        entity_id: String,
        value: String,
    },
    UnknownAssessmentEvidence {
        entity_id: String,
        value: String,
    },
    EmptyAssessmentEvidence {
        entity_id: String,
        field: &'static str,
    },
    DuplicateAssessmentEvidence {
        entity_id: String,
        field: &'static str,
        value: String,
    },
    InvalidDetectorVersion {
        entity_id: String,
        version: u32,
    },
    MissingPatternConcepts {
        entity_id: String,
    },
    MissingPatternObjectives {
        entity_id: String,
    },
    MissingPatternSection {
        entity_id: String,
        section: &'static str,
    },
    DuplicatePatternSection {
        entity_id: String,
        section: &'static str,
    },
    OutOfOrderPatternSection {
        entity_id: String,
        section: &'static str,
    },
    UnknownPatternSection {
        entity_id: String,
        heading: String,
    },
    ContentBeforePatternExplanation {
        entity_id: String,
    },
    MissingAssessedObjectives {
        entity_id: String,
    },
    AssessedObjectiveNotInFamily {
        problem_family_id: String,
        objective_id: String,
    },
    UnresolvedPattern {
        entity_id: String,
        target: String,
    },
    DuplicateOpportunityId {
        opportunity_id: String,
    },
    DuplicatePatternInFamily {
        problem_family_id: String,
        pattern_id: String,
    },
    OpportunityPatternScopeMismatch {
        problem_family_id: String,
        opportunity_id: String,
        pattern_id: String,
    },
    OpportunityMissingRequiredEvidence {
        problem_family_id: String,
        opportunity_id: String,
        missing: String,
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
                write!(f, "unsupported schema_version {found}; only 1 and 2 are accepted")
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
            Self::MissingProblemFamilyObjectives { entity_id } => write!(f, "problem family {entity_id} has no objective_ids; at least one is required"),
            Self::NonFiniteParameterBound { entity_id, parameter, field } => write!(f, "{entity_id}.parameters.{parameter}.{field} must contain a finite bound or reference offset"),
            Self::InvalidHintLevel { entity_id, level } => write!(f, "{entity_id} declares invalid hint level {level}; levels must be positive"),
            Self::OutOfOrderHintLevel { entity_id, previous_level, level } => write!(f, "{entity_id} declares hint level {level} after {previous_level}; levels must ascend in body bullet order"),
            Self::InvalidParameterName { entity_id, parameter } => write!(f, "{entity_id} declares parameter name {parameter:?} that cannot be referenced in constraints"),
            Self::NonFiniteCanonicalSolution { entity_id } => write!(f, "{entity_id}.canonical_solution.value must be finite"),
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
            Self::PatternsDirectoryInSchemaV1 { path } => write!(f, "{} contains a patterns/ directory, which requires schema_version = 2", path.display()),
            Self::UnknownBloomLevel { entity_id, value } => write!(f, "{entity_id} declares unknown bloom_level: {value}"),
            Self::UnknownAssessmentEvidence { entity_id, value } => write!(f, "{entity_id} declares unknown assessment_evidence: {value}"),
            Self::EmptyAssessmentEvidence { entity_id, field } => write!(f, "{entity_id}.{field} must contain at least one assessment evidence item"),
            Self::DuplicateAssessmentEvidence { entity_id, field, value } => write!(f, "{entity_id}.{field} lists {value} more than once"),
            Self::InvalidDetectorVersion { entity_id, version } => write!(f, "{entity_id} declares invalid detector version {version}; version must be >= 1"),
            Self::MissingPatternConcepts { entity_id } => write!(f, "pattern {entity_id} has no concept_ids; at least one is required"),
            Self::MissingPatternObjectives { entity_id } => write!(f, "pattern {entity_id} has no objective_ids; at least one is required"),
            Self::MissingPatternSection { entity_id, section } => write!(f, "pattern {entity_id} is missing required section ## {section}"),
            Self::DuplicatePatternSection { entity_id, section } => write!(f, "pattern {entity_id} declares ## {section} more than once"),
            Self::OutOfOrderPatternSection { entity_id, section } => write!(f, "pattern {entity_id}: ## {section} appears out of order"),
            Self::UnknownPatternSection { entity_id, heading } => write!(f, "pattern {entity_id} contains unrecognized heading: {heading}"),
            Self::ContentBeforePatternExplanation { entity_id } => write!(f, "pattern {entity_id} has non-whitespace content before ## Explanation"),
            Self::MissingAssessedObjectives { entity_id } => write!(f, "problem family {entity_id} has no assessed_objective_ids; at least one is required"),
            Self::AssessedObjectiveNotInFamily { problem_family_id, objective_id } => write!(f, "problem family {problem_family_id} lists assessed objective {objective_id} not present in objective_ids"),
            Self::UnresolvedPattern { entity_id, target } => write!(f, "{entity_id} references unknown Pattern: {target}"),
            Self::DuplicateOpportunityId { opportunity_id } => write!(f, "duplicate Opportunity id across package: {opportunity_id}"),
            Self::DuplicatePatternInFamily { problem_family_id, pattern_id } => write!(f, "problem family {problem_family_id} declares pattern {pattern_id} more than once"),
            Self::OpportunityPatternScopeMismatch { problem_family_id, opportunity_id, pattern_id } => write!(f, "problem family {problem_family_id} opportunity {opportunity_id} references pattern {pattern_id} whose objectives do not intersect the family's assessed_objective_ids"),
            Self::OpportunityMissingRequiredEvidence { problem_family_id, opportunity_id, missing } => write!(f, "problem family {problem_family_id} opportunity {opportunity_id} omits required evidence {missing}"),
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
