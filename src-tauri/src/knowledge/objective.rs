use std::path::Path;

use super::error::KnowledgeError;
use super::frontmatter::split_frontmatter;
use super::identifier::ConceptId;
use super::pattern::{parse_assessment_evidence_list, parse_bloom_level};
use super::provenance::convert_provenance_refs;
use super::raw::{RawObjectiveFrontmatter, RawObjectiveV2Frontmatter};
use super::types::Objective;

#[cfg(test)]
pub(crate) fn parse_objective_file(path: &Path, raw: &str) -> Result<Objective, KnowledgeError> {
    parse_objective_file_with_version(path, raw, 1)
}

pub(crate) fn parse_objective_file_with_version(
    path: &Path,
    raw: &str,
    schema_version: u32,
) -> Result<Objective, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let description = body.trim().to_owned();
    if description.is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: path.to_owned(),
            field: "description",
        });
    }

    if schema_version == 1 {
        let raw_frontmatter: RawObjectiveFrontmatter =
            toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
                path: path.to_owned(),
                source,
            })?;

        let id = super::identifier::ObjectiveId::new(raw_frontmatter.id)?;
        let concept_id = ConceptId::new(raw_frontmatter.concept_id)?;
        let provenance_refs =
            convert_provenance_refs(id.as_str(), raw_frontmatter.provenance_refs)?;

        Ok(Objective {
            id,
            concept_id,
            description: body,
            provenance_refs,
            bloom_level: None,
            assessment_evidence: None,
        })
    } else {
        let raw_frontmatter: RawObjectiveV2Frontmatter =
            toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
                path: path.to_owned(),
                source,
            })?;

        let id = super::identifier::ObjectiveId::new(raw_frontmatter.id)?;
        let entity_id = id.as_str().to_owned();
        let concept_id = ConceptId::new(raw_frontmatter.concept_id)?;
        let bloom_level = parse_bloom_level(&entity_id, &raw_frontmatter.bloom_level)?;
        let assessment_evidence = parse_assessment_evidence_list(
            &entity_id,
            "assessment_evidence",
            &raw_frontmatter.assessment_evidence,
        )?;
        let provenance_refs = convert_provenance_refs(&entity_id, raw_frontmatter.provenance_refs)?;

        Ok(Objective {
            id,
            concept_id,
            description: body,
            provenance_refs,
            bloom_level: Some(bloom_level),
            assessment_evidence: Some(assessment_evidence),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const VALID_OBJECTIVE: &str = r#"+++
id = "shell.setup_radius_height"
concept_id = "shell.method_vertical_axis"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"
+++

Identify and express the shell radius and shell height.
"#;

    #[test]
    fn valid_objective_parses() {
        let objective = parse_objective_file(
            Path::new("objectives/shell.setup_radius_height.md"),
            VALID_OBJECTIVE,
        )
        .unwrap();
        assert_eq!(objective.id.as_str(), "shell.setup_radius_height");
        assert_eq!(objective.concept_id.as_str(), "shell.method_vertical_axis");
        assert_eq!(objective.provenance_refs.len(), 1);
    }

    #[test]
    fn missing_provenance_is_rejected() {
        let raw = VALID_OBJECTIVE.replace(
            "[[provenance_refs]]\nsource_id = \"src.openstax_calc2\"\nkind = \"direct\"\n[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n",
            "",
        );
        assert!(matches!(
            parse_objective_file(Path::new("objectives/shell.setup_radius_height.md"), &raw),
            Err(KnowledgeError::MissingProvenance { .. })
        ));
    }
}
