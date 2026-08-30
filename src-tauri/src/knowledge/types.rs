use semver::Version;
use serde::{Deserialize, Serialize};

use super::identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};

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
}
