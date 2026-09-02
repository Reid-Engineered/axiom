use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawKnowledgePackage {
    pub id: String,
    pub schema_version: u32,
    pub version: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSourcesFile {
    #[serde(default)]
    pub sources: Vec<RawSource>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSource {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub edition: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawConceptFrontmatter {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub prerequisite_ids: Vec<String>,
    #[serde(default)]
    pub related_ids: Vec<String>,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawObjectiveFrontmatter {
    pub id: String,
    pub concept_id: String,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawExampleFrontmatter {
    pub id: String,
    pub concept_id: String,
    #[serde(default)]
    pub objective_ids: Vec<String>,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProblemFamilyFrontmatter {
    pub id: String,
    pub concept_id: String,
    #[serde(default)]
    pub objective_ids: Vec<String>,
    pub difficulty: RawDifficultyRange,
    pub generator: RawGeneratorRef,
    #[serde(default)]
    pub parameters: std::collections::BTreeMap<String, RawParameterSpec>,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub response_type: String,
    pub canonical_solution: RawCanonicalSolution,
    #[serde(default)]
    pub hints: Vec<RawHint>,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
    pub status: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDifficultyRange {
    pub min: u8,
    pub max: u8,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawGeneratorRef {
    pub id: String,
    pub version: u32,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawBound {
    Literal(f64),
    Reference {
        parameter: String,
        #[serde(default)]
        offset: f64,
    },
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawParameterSpec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub value: Option<RawBound>,
    #[serde(default)]
    pub min: Option<RawBound>,
    #[serde(default)]
    pub max: Option<RawBound>,
    #[serde(default)]
    pub description: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawCanonicalSolution {
    #[serde(default)]
    pub expression: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawHint {
    pub level: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProvenanceRef {
    pub source_id: String,
    #[serde(default)]
    pub locator: Option<RawSourceLocator>,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSourceLocator {
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub pages: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_top_level_key_is_rejected() {
        let toml = r#"
            id = "org.axiom.calculus_shells"
            schema_version = 1
            version = "1.0.0"
            title = "Cylindrical Shells"
            description = "A tiny reference package."
            unexpected = "typo"
        "#;
        assert!(toml::from_str::<RawKnowledgePackage>(toml).is_err());
    }

    #[test]
    fn valid_package_toml_parses() {
        let toml = r#"
            id = "org.axiom.calculus_shells"
            schema_version = 1
            version = "1.0.0"
            title = "Cylindrical Shells"
            description = "A tiny reference package."
        "#;
        let raw: RawKnowledgePackage = toml::from_str(toml).unwrap();
        assert_eq!(raw.id, "org.axiom.calculus_shells");
        assert_eq!(raw.schema_version, 1);
    }

    #[test]
    fn concept_frontmatter_defaults_relationship_fields_to_empty() {
        let toml = r#"
            id = "shell.method_vertical_axis"
            name = "The Method of Cylindrical Shells"
        "#;
        let raw: RawConceptFrontmatter = toml::from_str(toml).unwrap();
        assert!(raw.prerequisite_ids.is_empty());
        assert!(raw.related_ids.is_empty());
        assert!(raw.provenance_refs.is_empty());
    }

    #[test]
    fn source_authors_default_to_empty() {
        let toml = r#"
            id = "src.axiom_original"
            title = "Axiom Original Content"
        "#;
        let raw: RawSource = toml::from_str(toml).unwrap();
        assert!(raw.authors.is_empty());
        assert!(raw.edition.is_none());
        assert!(raw.license.is_none());
    }
}
