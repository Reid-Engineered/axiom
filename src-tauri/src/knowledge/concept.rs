use std::path::Path;

use super::error::KnowledgeError;
use super::frontmatter::split_frontmatter;
use super::identifier::ConceptId;
use super::provenance::convert_provenance_refs;
use super::raw::RawConceptFrontmatter;
use super::types::Concept;

pub(crate) fn parse_concept_file(path: &Path, raw: &str) -> Result<Concept, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let raw_frontmatter: RawConceptFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;

    let id = ConceptId::new(raw_frontmatter.id)?;
    let prerequisite_ids = raw_frontmatter
        .prerequisite_ids
        .into_iter()
        .map(ConceptId::new)
        .collect::<Result<Vec<_>, _>>()?;
    let related_ids = raw_frontmatter
        .related_ids
        .into_iter()
        .map(ConceptId::new)
        .collect::<Result<Vec<_>, _>>()?;

    let description = body.trim().to_owned();
    if description.is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: path.to_owned(),
            field: "description",
        });
    }

    let provenance_refs = convert_provenance_refs(id.as_str(), raw_frontmatter.provenance_refs)?;

    Ok(Concept {
        id,
        name: raw_frontmatter.name,
        topic: raw_frontmatter.topic,
        description: body,
        prerequisite_ids,
        related_ids,
        provenance_refs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ProvenanceKind is only needed here, in this test module, not by
    // parse_concept_file's own body — imported here rather than at the file's outer
    // scope, which would trigger unused_imports on a non-test build (the #[cfg(test)]
    // module that actually uses it doesn't exist in that compilation).
    use crate::knowledge::ProvenanceKind;

    const VALID_CONCEPT: &str = r#"+++
id = "shell.method_vertical_axis"
name = "The Method of Cylindrical Shells"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = ["shell.method_horizontal_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"
+++

A method for calculating the volume of a solid of revolution.
"#;

    #[test]
    fn valid_concept_parses() {
        let concept = parse_concept_file(
            Path::new("concepts/shell.method_vertical_axis.md"),
            VALID_CONCEPT,
        )
        .unwrap();
        assert_eq!(concept.id.as_str(), "shell.method_vertical_axis");
        assert_eq!(concept.related_ids.len(), 1);
        assert_eq!(
            concept.description,
            "A method for calculating the volume of a solid of revolution.\n"
        );
        assert_eq!(concept.provenance_refs.len(), 1);
        assert_eq!(concept.provenance_refs[0].kind, ProvenanceKind::Direct);
    }

    #[test]
    fn empty_description_is_rejected() {
        let raw = VALID_CONCEPT.replace(
            "A method for calculating the volume of a solid of revolution.\n",
            "",
        );
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::EmptyField {
                field: "description",
                ..
            })
        ));
    }

    #[test]
    fn missing_provenance_is_rejected() {
        let raw = VALID_CONCEPT.replace(
            "[[provenance_refs]]\nsource_id = \"src.openstax_calc2\"\nkind = \"direct\"\n[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n",
            "",
        );
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::MissingProvenance { .. })
        ));
    }

    #[test]
    fn unknown_provenance_kind_is_rejected() {
        let raw = VALID_CONCEPT.replace(r#"kind = "direct""#, r#"kind = "inferred""#);
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::UnknownProvenanceKind { .. })
        ));
    }

    #[test]
    fn empty_locator_table_is_rejected() {
        let raw = VALID_CONCEPT.replace(
            "[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n",
            "[provenance_refs.locator]\n",
        );
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::EmptySourceLocator { .. })
        ));
    }

    #[test]
    fn exact_duplicate_provenance_ref_is_rejected() {
        let raw = VALID_CONCEPT.replace(
            "+++\n\nA method",
            "\n[[provenance_refs]]\nsource_id = \"src.openstax_calc2\"\nkind = \"direct\"\n[provenance_refs.locator]\nsection = \"2.3\"\nlabel = \"Rule 2.6\"\n+++\n\nA method",
        );
        assert!(matches!(
            parse_concept_file(Path::new("concepts/shell.method_vertical_axis.md"), &raw),
            Err(KnowledgeError::DuplicateProvenanceRef { .. })
        ));
    }
}
