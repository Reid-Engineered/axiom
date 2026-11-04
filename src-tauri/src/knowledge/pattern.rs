use std::collections::HashSet;
use std::path::Path;

use super::error::KnowledgeError;
use super::frontmatter::split_frontmatter;
use super::identifier::{ConceptId, DetectorId, ObjectiveId, PatternId, PolicyId};
use super::provenance::convert_provenance_refs;
use super::raw::RawPatternFrontmatter;
use super::types::{AssessmentEvidence, BloomLevel, DetectorRef, Pattern};

const EXPLANATION: &str = "## Explanation";
const NEXT_ACTION: &str = "## Next action";
const RECOGNIZED_HEADINGS: [&str; 2] = [EXPLANATION, NEXT_ACTION];

pub(crate) fn parse_bloom_level(entity_id: &str, raw: &str) -> Result<BloomLevel, KnowledgeError> {
    match raw {
        "remember" => Ok(BloomLevel::Remember),
        "understand" => Ok(BloomLevel::Understand),
        "apply" => Ok(BloomLevel::Apply),
        "analyze" => Ok(BloomLevel::Analyze),
        "evaluate" => Ok(BloomLevel::Evaluate),
        "create" => Ok(BloomLevel::Create),
        other => Err(KnowledgeError::UnknownBloomLevel {
            entity_id: entity_id.to_owned(),
            value: other.to_owned(),
        }),
    }
}

pub(crate) fn parse_assessment_evidence_list(
    entity_id: &str,
    field: &'static str,
    raw_list: &[String],
) -> Result<Vec<AssessmentEvidence>, KnowledgeError> {
    if raw_list.is_empty() {
        return Err(KnowledgeError::EmptyAssessmentEvidence {
            entity_id: entity_id.to_owned(),
            field,
        });
    }
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(raw_list.len());
    for item in raw_list {
        let parsed = match item.as_str() {
            "final-answer" => AssessmentEvidence::FinalAnswer,
            "structured-step" => AssessmentEvidence::StructuredStep,
            "method-selection" => AssessmentEvidence::MethodSelection,
            other => {
                return Err(KnowledgeError::UnknownAssessmentEvidence {
                    entity_id: entity_id.to_owned(),
                    value: other.to_owned(),
                })
            }
        };
        if !seen.insert(parsed) {
            return Err(KnowledgeError::DuplicateAssessmentEvidence {
                entity_id: entity_id.to_owned(),
                field,
                value: item.clone(),
            });
        }
        out.push(parsed);
    }
    Ok(out)
}

pub(crate) fn parse_pattern_file(path: &Path, raw: &str) -> Result<Pattern, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let raw_frontmatter: RawPatternFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;

    let id = PatternId::new(raw_frontmatter.id)?;
    let entity_id = id.as_str().to_owned();

    if raw_frontmatter.name.trim().is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: path.to_owned(),
            field: "name",
        });
    }

    if raw_frontmatter.concept_ids.is_empty() {
        return Err(KnowledgeError::MissingPatternConcepts {
            entity_id: entity_id.clone(),
        });
    }
    let mut seen_concepts = HashSet::new();
    let mut concept_ids = Vec::with_capacity(raw_frontmatter.concept_ids.len());
    for raw_concept_id in raw_frontmatter.concept_ids {
        let concept_id = ConceptId::new(raw_concept_id)?;
        if !seen_concepts.insert(concept_id.clone()) {
            return Err(KnowledgeError::DuplicateReferenceInList {
                entity_id: entity_id.clone(),
                field: "concept_ids",
                target: concept_id.as_str().to_owned(),
            });
        }
        concept_ids.push(concept_id);
    }

    if raw_frontmatter.objective_ids.is_empty() {
        return Err(KnowledgeError::MissingPatternObjectives {
            entity_id: entity_id.clone(),
        });
    }
    let mut seen_objectives = HashSet::new();
    let mut objective_ids = Vec::with_capacity(raw_frontmatter.objective_ids.len());
    for raw_objective_id in raw_frontmatter.objective_ids {
        let objective_id = ObjectiveId::new(raw_objective_id)?;
        if !seen_objectives.insert(objective_id.clone()) {
            return Err(KnowledgeError::DuplicateReferenceInList {
                entity_id: entity_id.clone(),
                field: "objective_ids",
                target: objective_id.as_str().to_owned(),
            });
        }
        objective_ids.push(objective_id);
    }

    if raw_frontmatter.detector.version == 0 {
        return Err(KnowledgeError::InvalidDetectorVersion {
            entity_id: entity_id.clone(),
            version: 0,
        });
    }
    let detector = DetectorRef {
        id: DetectorId::new(raw_frontmatter.detector.id)?,
        version: raw_frontmatter.detector.version,
    };

    let required_evidence = parse_assessment_evidence_list(
        &entity_id,
        "required_evidence",
        &raw_frontmatter.required_evidence,
    )?;

    let resolution_policy_id = PolicyId::new(raw_frontmatter.resolution_policy_id)?;
    let provenance_refs = convert_provenance_refs(&entity_id, raw_frontmatter.provenance_refs)?;
    let (explanation, next_action) = parse_pattern_body(&entity_id, &body)?;

    Ok(Pattern {
        id,
        name: raw_frontmatter.name,
        concept_ids,
        objective_ids,
        detector,
        required_evidence,
        resolution_policy_id,
        explanation,
        next_action,
        provenance_refs,
    })
}

fn parse_pattern_body(entity_id: &str, body: &str) -> Result<(String, String), KnowledgeError> {
    let mut sections: Vec<(&str, Vec<&str>)> = Vec::new();
    let mut current: Option<(&str, Vec<&str>)> = None;
    let mut preamble = Vec::new();

    for line in body.lines() {
        let trimmed_end = line.trim_end_matches('\r');
        if let Some(heading) = RECOGNIZED_HEADINGS.iter().find(|h| trimmed_end == **h) {
            if let Some(finished) = current.take() {
                sections.push(finished);
            }
            current = Some((*heading, Vec::new()));
        } else if trimmed_end.starts_with("## ") {
            return Err(KnowledgeError::UnknownPatternSection {
                entity_id: entity_id.to_owned(),
                heading: trimmed_end.trim().to_owned(),
            });
        } else if let Some((_, content)) = current.as_mut() {
            content.push(line);
        } else {
            preamble.push(line);
        }
    }
    if let Some(finished) = current {
        sections.push(finished);
    }

    if preamble.iter().any(|line| !line.trim().is_empty()) {
        return Err(KnowledgeError::ContentBeforePatternExplanation {
            entity_id: entity_id.to_owned(),
        });
    }

    let mut seen = Vec::new();
    for (heading, _) in &sections {
        if seen.contains(heading) {
            return Err(KnowledgeError::DuplicatePatternSection {
                entity_id: entity_id.to_owned(),
                section: section_name(heading),
            });
        }
        seen.push(*heading);
    }

    let mut highest_seen = None;
    for (heading, _) in &sections {
        let position = RECOGNIZED_HEADINGS
            .iter()
            .position(|h| h == heading)
            .expect("recognized heading");
        if highest_seen.is_some_and(|highest| position < highest) {
            return Err(KnowledgeError::OutOfOrderPatternSection {
                entity_id: entity_id.to_owned(),
                section: section_name(heading),
            });
        }
        highest_seen = Some(position);
    }

    let explanation = section_text(&sections, EXPLANATION, entity_id, "Explanation")?;
    let next_action = section_text(&sections, NEXT_ACTION, entity_id, "Next action")?;
    Ok((explanation, next_action))
}

fn section_name(heading: &str) -> &'static str {
    match heading {
        EXPLANATION => "Explanation",
        NEXT_ACTION => "Next action",
        _ => unreachable!(),
    }
}

fn section_text(
    sections: &[(&str, Vec<&str>)],
    heading: &str,
    entity_id: &str,
    section: &'static str,
) -> Result<String, KnowledgeError> {
    let text = sections
        .iter()
        .find(|(candidate, _)| *candidate == heading)
        .map(|(_, lines)| lines.join("\n").trim().to_owned())
        .ok_or_else(|| KnowledgeError::MissingPatternSection {
            entity_id: entity_id.to_owned(),
            section,
        })?;
    if text.is_empty() {
        return Err(KnowledgeError::MissingPatternSection {
            entity_id: entity_id.to_owned(),
            section,
        });
    }
    Ok(text)
}
