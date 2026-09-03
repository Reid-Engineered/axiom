use semver::Version;
use serde::{Deserialize, Serialize};

use super::identifier::{
    ConceptId, ExampleId, GeneratorId, KnowledgePackageId, ObjectiveId, ProblemFamilyId, SourceId,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgePackage {
    pub id: KnowledgePackageId,
    pub schema_version: u32,
    pub version: Version,
    pub title: String,
    pub description: String,
    pub concepts: Vec<Concept>,
    pub objectives: Vec<Objective>,
    pub examples: Vec<Example>,
    pub problem_families: Vec<ProblemFamily>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    pub id: ConceptId,
    pub name: String,
    pub topic: Option<String>,
    pub description: String,
    pub prerequisite_ids: Vec<ConceptId>,
    pub related_ids: Vec<ConceptId>,
    pub provenance_refs: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub id: ObjectiveId,
    pub concept_id: ConceptId,
    pub description: String,
    pub provenance_refs: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Example {
    pub id: ExampleId,
    pub concept_id: ConceptId,
    pub objective_ids: Vec<ObjectiveId>,
    pub problem: String,
    pub solution: String,
    pub hints: Vec<String>,
    pub provenance_refs: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemFamily {
    pub id: ProblemFamilyId,
    pub concept_id: ConceptId,
    pub objective_ids: Vec<ObjectiveId>,
    pub difficulty: DifficultyRange,
    pub generator: GeneratorRef,
    pub parameters: std::collections::BTreeMap<String, ParameterSpec>,
    pub constraints: Vec<crate::knowledge::constraint::ConstraintExpr>,
    pub prompt: String,
    pub solution_structure: String,
    pub response_type: ResponseType,
    pub canonical_solution: CanonicalSolution,
    pub hints: Vec<Hint>,
    pub provenance_refs: Vec<ProvenanceRef>,
    pub status: ProblemFamilyStatus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifficultyRange {
    pub min: u8,
    pub max: u8,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratorRef {
    pub id: GeneratorId,
    pub version: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Integer,
    Float,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Bound {
    Literal(f64),
    Reference {
        parameter: String,
        #[serde(default)]
        offset: f64,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSpec {
    #[serde(rename = "type")]
    pub kind: ParameterType,
    #[serde(default)]
    pub value: Option<Bound>,
    #[serde(default)]
    pub min: Option<Bound>,
    #[serde(default)]
    pub max: Option<Bound>,
    #[serde(default)]
    pub description: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResponseType {
    SymbolicExpression,
    Numeric,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CanonicalSolution {
    Symbolic { expression: String },
    Numeric { value: f64 },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hint {
    pub level: u32,
    pub text: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProblemFamilyStatus {
    Verified,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemInstance {
    pub family_id: ProblemFamilyId,
    pub seed: u64,
    pub resolved_parameters: std::collections::BTreeMap<String, f64>,
    pub prompt: String,
    pub canonical_solution: ResolvedSolution,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum ResolvedSolution {
    Symbolic(String),
    Numeric(f64),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub title: String,
    pub authors: Vec<String>,
    pub edition: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProvenanceKind {
    Direct,
    Derived,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRef {
    pub source_id: SourceId,
    pub locator: Option<SourceLocator>,
    pub kind: ProvenanceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocator {
    pub section: Option<String>,
    pub pages: Option<String>,
    pub label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::identifier::{ConceptId, ExampleId, ObjectiveId, SourceId};

    #[test]
    fn example_round_trips_through_json() {
        let example = Example {
            id: ExampleId::new("shell.example_basic").unwrap(),
            concept_id: ConceptId::new("shell.method_vertical_axis").unwrap(),
            objective_ids: vec![ObjectiveId::new("shell.setup_radius_height").unwrap()],
            problem: "Find the volume...".to_owned(),
            solution: "V = 8pi/3".to_owned(),
            hints: vec!["Identify the radius first.".to_owned()],
            provenance_refs: vec![ProvenanceRef {
                source_id: SourceId::new("src.openstax_calc2").unwrap(),
                locator: Some(SourceLocator {
                    section: Some("2.3".to_owned()),
                    pages: None,
                    label: Some("Example 2.13".to_owned()),
                }),
                kind: ProvenanceKind::Direct,
            }],
        };

        let json = serde_json::to_string(&example).unwrap();
        let round_tripped: Example = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, example);
    }

    #[test]
    fn problem_family_round_trips_through_json() {
        let family = ProblemFamily {
            id: ProblemFamilyId::new("problem.shell_y_poly").unwrap(),
            concept_id: ConceptId::new("shell.method_vertical_axis").unwrap(),
            objective_ids: vec![],
            difficulty: DifficultyRange { min: 1, max: 2 },
            generator: GeneratorRef {
                id: GeneratorId::new("gen.shell_y_poly").unwrap(),
                version: 1,
            },
            parameters: std::collections::BTreeMap::new(),
            constraints: vec![],
            prompt: "Define R...".to_owned(),
            solution_structure: "V = ...".to_owned(),
            response_type: ResponseType::Numeric,
            canonical_solution: CanonicalSolution::Numeric { value: 1.0 },
            hints: vec![],
            provenance_refs: vec![ProvenanceRef {
                source_id: SourceId::new("src.openstax_calc2").unwrap(),
                locator: None,
                kind: ProvenanceKind::Direct,
            }],
            status: ProblemFamilyStatus::Verified,
        };
        let json = serde_json::to_string(&family).unwrap();
        assert_eq!(
            serde_json::to_value(&family).unwrap()["generator"]["id"],
            "gen.shell_y_poly"
        );
        assert_eq!(
            serde_json::from_str::<ProblemFamily>(&json).unwrap(),
            family
        );
    }

    #[test]
    fn problem_instance_round_trips_through_json() {
        let instance = ProblemInstance {
            family_id: ProblemFamilyId::new("problem.shell_y_poly").unwrap(),
            seed: 42,
            resolved_parameters: std::collections::BTreeMap::from([("coeff".to_owned(), 4.0)]),
            prompt: "Define R...".to_owned(),
            canonical_solution: ResolvedSolution::Symbolic("2*pi".to_owned()),
            hints: vec!["Identify the radius.".to_owned()],
        };
        let json = serde_json::to_string(&instance).unwrap();
        assert!(json.contains("\"kind\":\"symbolic\""));
        assert_eq!(
            serde_json::from_str::<ProblemInstance>(&json).unwrap(),
            instance
        );
    }
}
