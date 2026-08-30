use std::collections::{HashMap, HashSet};

use super::discover::DiscoveredEntities;
use super::error::KnowledgeError;
use super::identifier::SourceId;
use super::types::{ProvenanceRef, Source};

pub(crate) fn validate_references(
    entities: &DiscoveredEntities,
    sources: &[Source],
) -> Result<(), KnowledgeError> {
    let concept_ids: HashSet<_> = entities.concepts.iter().map(|c| &c.id).collect();
    let objectives_by_id: HashMap<_, _> = entities.objectives.iter().map(|o| (&o.id, o)).collect();
    let source_ids: HashSet<_> = sources.iter().map(|s| &s.id).collect();

    for objective in &entities.objectives {
        if !concept_ids.contains(&objective.concept_id) {
            return Err(KnowledgeError::UnresolvedConcept {
                entity_id: objective.id.as_str().to_owned(),
                field: "concept_id",
                target: objective.concept_id.as_str().to_owned(),
            });
        }
    }

    for example in &entities.examples {
        if !concept_ids.contains(&example.concept_id) {
            return Err(KnowledgeError::UnresolvedConcept {
                entity_id: example.id.as_str().to_owned(),
                field: "concept_id",
                target: example.concept_id.as_str().to_owned(),
            });
        }
        for objective_id in &example.objective_ids {
            let Some(objective) = objectives_by_id.get(objective_id) else {
                return Err(KnowledgeError::UnresolvedObjective {
                    entity_id: example.id.as_str().to_owned(),
                    field: "objective_ids",
                    target: objective_id.as_str().to_owned(),
                });
            };
            if objective.concept_id != example.concept_id {
                return Err(KnowledgeError::CrossConceptObjective {
                    example_id: example.id.as_str().to_owned(),
                    objective_id: objective_id.as_str().to_owned(),
                });
            }
        }
    }

    validate_provenance_sources(
        entities
            .concepts
            .iter()
            .map(|c| (c.id.as_str(), &c.provenance_refs)),
        &source_ids,
    )?;
    validate_provenance_sources(
        entities
            .objectives
            .iter()
            .map(|o| (o.id.as_str(), &o.provenance_refs)),
        &source_ids,
    )?;
    validate_provenance_sources(
        entities
            .examples
            .iter()
            .map(|e| (e.id.as_str(), &e.provenance_refs)),
        &source_ids,
    )?;

    Ok(())
}

fn validate_provenance_sources<'a>(
    entities: impl Iterator<Item = (&'a str, &'a Vec<ProvenanceRef>)>,
    source_ids: &HashSet<&SourceId>,
) -> Result<(), KnowledgeError> {
    for (entity_id, refs) in entities {
        for provenance_ref in refs {
            if !source_ids.contains(&provenance_ref.source_id) {
                return Err(KnowledgeError::UnresolvedSource {
                    entity_id: entity_id.to_owned(),
                    target: provenance_ref.source_id.as_str().to_owned(),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::identifier::{ConceptId, ExampleId, ObjectiveId, SourceId};
    use crate::knowledge::types::{Concept, Example, Objective, ProvenanceKind, ProvenanceRef};

    fn source(id: &str) -> Source {
        Source {
            id: SourceId::new(id).unwrap(),
            title: "Source".to_owned(),
            authors: vec![],
            edition: None,
            license: None,
        }
    }

    fn provenance(source_id: &str) -> Vec<ProvenanceRef> {
        vec![ProvenanceRef {
            source_id: SourceId::new(source_id).unwrap(),
            locator: None,
            kind: ProvenanceKind::Direct,
        }]
    }

    fn concept(id: &str) -> Concept {
        Concept {
            id: ConceptId::new(id).unwrap(),
            name: id.to_owned(),
            topic: None,
            description: "d".to_owned(),
            prerequisite_ids: vec![],
            related_ids: vec![],
            provenance_refs: provenance("src.a"),
        }
    }

    fn objective(id: &str, concept_id: &str) -> Objective {
        Objective {
            id: ObjectiveId::new(id).unwrap(),
            concept_id: ConceptId::new(concept_id).unwrap(),
            description: "d".to_owned(),
            provenance_refs: provenance("src.a"),
        }
    }

    fn example(id: &str, concept_id: &str, objective_ids: Vec<&str>) -> Example {
        Example {
            id: ExampleId::new(id).unwrap(),
            concept_id: ConceptId::new(concept_id).unwrap(),
            objective_ids: objective_ids
                .into_iter()
                .map(|o| ObjectiveId::new(o).unwrap())
                .collect(),
            problem: "p".to_owned(),
            solution: "s".to_owned(),
            hints: vec![],
            provenance_refs: provenance("src.a"),
        }
    }

    #[test]
    fn valid_references_resolve() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![objective("shell.obj", "shell.a")],
            examples: vec![example("shell.ex", "shell.a", vec!["shell.obj"])],
        };
        assert!(validate_references(&entities, &[source("src.a")]).is_ok());
    }

    #[test]
    fn unresolved_objective_concept_id_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![objective("shell.obj", "shell.missing")],
            examples: vec![],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::UnresolvedConcept { .. })
        ));
    }

    #[test]
    fn unresolved_example_objective_id_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![],
            examples: vec![example("shell.ex", "shell.a", vec!["shell.missing"])],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::UnresolvedObjective { .. })
        ));
    }

    #[test]
    fn cross_concept_objective_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a"), concept("shell.b")],
            objectives: vec![objective("shell.obj", "shell.b")],
            examples: vec![example("shell.ex", "shell.a", vec!["shell.obj"])],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::CrossConceptObjective { .. })
        ));
    }

    #[test]
    fn unresolved_provenance_source_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![],
            examples: vec![],
        };
        assert!(matches!(
            validate_references(&entities, &[]),
            Err(KnowledgeError::UnresolvedSource { .. })
        ));
    }
}
