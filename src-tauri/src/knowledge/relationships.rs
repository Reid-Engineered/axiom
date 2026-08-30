use std::collections::{HashMap, HashSet};

use super::error::KnowledgeError;
use super::identifier::ConceptId;
use super::types::Concept;

#[derive(Clone, Copy, PartialEq)]
enum Color {
    White,
    Gray,
    Black,
}

pub(crate) fn validate_relationships(concepts: &[Concept]) -> Result<(), KnowledgeError> {
    let concept_ids: HashSet<&ConceptId> = concepts.iter().map(|c| &c.id).collect();

    for concept in concepts {
        validate_reference_list(
            &concept.id,
            "prerequisite_ids",
            &concept.prerequisite_ids,
            &concept_ids,
        )?;
        validate_reference_list(
            &concept.id,
            "related_ids",
            &concept.related_ids,
            &concept_ids,
        )?;
    }

    validate_related_symmetry(concepts)?;
    validate_prerequisite_dag(concepts)?;

    Ok(())
}

/// Spec §10's normalized symmetric query view: returns every concept related to
/// `id`, regardless of which side authored the edge.
pub fn related_concepts<'a>(concepts: &'a [Concept], id: &ConceptId) -> Vec<&'a ConceptId> {
    let mut result = Vec::new();
    for concept in concepts {
        if concept.id == *id {
            result.extend(concept.related_ids.iter());
        } else if concept.related_ids.contains(id) {
            result.push(&concept.id);
        }
    }
    result
}

fn validate_reference_list(
    owner: &ConceptId,
    field: &'static str,
    targets: &[ConceptId],
    concept_ids: &HashSet<&ConceptId>,
) -> Result<(), KnowledgeError> {
    let mut seen = HashSet::new();
    for target in targets {
        if target == owner {
            return Err(KnowledgeError::SelfReference {
                entity_id: owner.as_str().to_owned(),
                field,
            });
        }
        if !seen.insert(target) {
            return Err(KnowledgeError::DuplicateReferenceInList {
                entity_id: owner.as_str().to_owned(),
                field,
                target: target.as_str().to_owned(),
            });
        }
        if !concept_ids.contains(target) {
            return Err(KnowledgeError::UnresolvedConcept {
                entity_id: owner.as_str().to_owned(),
                field,
                target: target.as_str().to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_related_symmetry(concepts: &[Concept]) -> Result<(), KnowledgeError> {
    let by_id: HashMap<&ConceptId, &Concept> = concepts.iter().map(|c| (&c.id, c)).collect();
    for concept in concepts {
        for related in &concept.related_ids {
            if let Some(other) = by_id.get(related) {
                if other.related_ids.contains(&concept.id) {
                    return Err(KnowledgeError::ReverseDuplicateRelated {
                        first: concept.id.as_str().to_owned(),
                        second: related.as_str().to_owned(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_prerequisite_dag(concepts: &[Concept]) -> Result<(), KnowledgeError> {
    let by_id: HashMap<&ConceptId, &Concept> = concepts.iter().map(|c| (&c.id, c)).collect();
    let mut colors: HashMap<&ConceptId, Color> =
        concepts.iter().map(|c| (&c.id, Color::White)).collect();
    let mut path: Vec<ConceptId> = Vec::new();

    for id in by_id.keys() {
        if colors.get(id) == Some(&Color::White) {
            visit(id, &by_id, &mut colors, &mut path)?;
        }
    }
    Ok(())
}

fn visit<'a>(
    id: &'a ConceptId,
    by_id: &HashMap<&'a ConceptId, &'a Concept>,
    colors: &mut HashMap<&'a ConceptId, Color>,
    path: &mut Vec<ConceptId>,
) -> Result<(), KnowledgeError> {
    match colors.get(id) {
        Some(Color::Black) => return Ok(()),
        Some(Color::Gray) => {
            let start = path.iter().position(|node| node == id).unwrap_or(0);
            let mut cycle: Vec<String> = path[start..]
                .iter()
                .map(|c| c.as_str().to_owned())
                .collect();
            cycle.push(id.as_str().to_owned());
            return Err(KnowledgeError::PrerequisiteCycle { cycle });
        }
        _ => {}
    }
    colors.insert(id, Color::Gray);
    path.push(id.clone());
    if let Some(concept) = by_id.get(id) {
        for prerequisite in &concept.prerequisite_ids {
            visit(prerequisite, by_id, colors, path)?;
        }
    }
    path.pop();
    colors.insert(id, Color::Black);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::identifier::ConceptId;

    fn concept(id: &str, prerequisites: Vec<&str>, related: Vec<&str>) -> Concept {
        Concept {
            id: ConceptId::new(id).unwrap(),
            name: id.to_owned(),
            topic: None,
            description: "d".to_owned(),
            prerequisite_ids: prerequisites
                .into_iter()
                .map(|p| ConceptId::new(p).unwrap())
                .collect(),
            related_ids: related
                .into_iter()
                .map(|r| ConceptId::new(r).unwrap())
                .collect(),
            provenance_refs: vec![],
        }
    }

    #[test]
    fn acyclic_prerequisites_are_accepted() {
        let concepts = vec![
            concept("shell.a", vec![], vec![]),
            concept("shell.b", vec!["shell.a"], vec![]),
            concept("shell.c", vec!["shell.a", "shell.b"], vec![]),
        ];
        assert!(validate_relationships(&concepts).is_ok());
    }

    #[test]
    fn self_prerequisite_is_rejected() {
        let concepts = vec![concept("shell.a", vec!["shell.a"], vec![])];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::SelfReference {
                field: "prerequisite_ids",
                ..
            })
        ));
    }

    #[test]
    fn duplicate_prerequisite_id_is_rejected() {
        let concepts = vec![
            concept("shell.a", vec![], vec![]),
            concept("shell.b", vec!["shell.a", "shell.a"], vec![]),
        ];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::DuplicateReferenceInList {
                field: "prerequisite_ids",
                ..
            })
        ));
    }

    #[test]
    fn prerequisite_cycle_is_rejected() {
        let concepts = vec![
            concept("shell.a", vec!["shell.c"], vec![]),
            concept("shell.b", vec!["shell.a"], vec![]),
            concept("shell.c", vec!["shell.b"], vec![]),
        ];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::PrerequisiteCycle { .. })
        ));
    }

    #[test]
    fn self_related_is_rejected() {
        let concepts = vec![concept("shell.a", vec![], vec!["shell.a"])];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::SelfReference {
                field: "related_ids",
                ..
            })
        ));
    }

    #[test]
    fn reverse_double_authored_related_is_rejected() {
        let concepts = vec![
            concept("shell.a", vec![], vec!["shell.b"]),
            concept("shell.b", vec![], vec!["shell.a"]),
        ];
        assert!(matches!(
            validate_relationships(&concepts),
            Err(KnowledgeError::ReverseDuplicateRelated { .. })
        ));
    }

    #[test]
    fn related_concepts_exposes_the_relation_symmetrically() {
        let concepts = vec![
            concept("shell.a", vec![], vec!["shell.b"]),
            concept("shell.b", vec![], vec![]),
        ];
        let from_a = related_concepts(&concepts, &ConceptId::new("shell.a").unwrap());
        let from_b = related_concepts(&concepts, &ConceptId::new("shell.b").unwrap());
        assert_eq!(from_a, vec![&ConceptId::new("shell.b").unwrap()]);
        assert_eq!(from_b, vec![&ConceptId::new("shell.a").unwrap()]);
    }

    #[test]
    fn a_long_related_chain_is_not_treated_as_a_cycle_error() {
        // related has no acyclic constraint (spec §10) — this must not error.
        let concepts = vec![
            concept("shell.a", vec![], vec!["shell.b"]),
            concept("shell.b", vec![], vec!["shell.c"]),
            concept("shell.c", vec![], vec!["shell.a"]),
        ];
        assert!(validate_relationships(&concepts).is_ok());
    }
}
