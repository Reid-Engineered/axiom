use std::path::Path;

use super::discover::{discover_entities, read_package_toml, read_sources_toml};
use super::error::KnowledgeError;
use super::package::{parse_package_toml, parse_sources_toml};
use super::relationships::validate_relationships;
use super::types::KnowledgePackage;
use super::validate::validate_references;

/// Loads and fully validates a Knowledge Package from `root`. Atomic: either every
/// entity parses and every invariant in the spec holds, and a complete, validated
/// `KnowledgePackage` is returned, or the first failure is returned and nothing is.
/// There is no partially-loaded package.
pub fn load_knowledge_package(root: &Path) -> Result<KnowledgePackage, KnowledgeError> {
    let package_toml = read_package_toml(root)?;
    let identity = parse_package_toml(&package_toml)?;

    let sources_toml = read_sources_toml(root)?;
    let sources = parse_sources_toml(&sources_toml)?;

    let entities = discover_entities(root)?;

    validate_references(&entities, &sources)?;
    validate_relationships(&entities.concepts)?;

    Ok(KnowledgePackage {
        id: identity.id,
        schema_version: identity.schema_version,
        version: identity.version,
        title: identity.title,
        description: identity.description,
        concepts: entities.concepts,
        objectives: entities.objectives,
        examples: entities.examples,
        problem_families: entities.problem_families,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("axiom_knowledge_loader_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_minimal_valid_package(root: &Path) {
        fs::write(
            root.join("package.toml"),
            "id = \"org.axiom.test\"\nschema_version = 1\nversion = \"1.0.0\"\ntitle = \"Test\"\ndescription = \"A test package.\"\n",
        )
        .unwrap();
        fs::write(
            root.join("sources.toml"),
            "[[sources]]\nid = \"src.a\"\ntitle = \"Source A\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("concepts")).unwrap();
        fs::write(
            root.join("concepts/shell.a.md"),
            "+++\nid = \"shell.a\"\nname = \"A\"\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nBody.\n",
        )
        .unwrap();
    }

    #[test]
    fn minimal_valid_package_loads() {
        let root = temp_root("minimal_valid_package_loads");
        write_minimal_valid_package(&root);
        let package = load_knowledge_package(&root).unwrap();
        assert_eq!(package.id.as_str(), "org.axiom.test");
        assert_eq!(package.concepts.len(), 1);
        assert_eq!(package.sources.len(), 1);
    }

    #[test]
    fn one_broken_entity_invalidates_the_whole_package() {
        let root = temp_root("one_broken_entity_invalidates_the_whole_package");
        write_minimal_valid_package(&root);
        // A second concept, structurally fine on its own, but citing a Source that
        // does not exist. This must fail the whole load — not silently drop shell.b
        // and return a package containing only shell.a.
        fs::write(
            root.join("concepts/shell.b.md"),
            "+++\nid = \"shell.b\"\nname = \"B\"\n\n[[provenance_refs]]\nsource_id = \"src.missing\"\nkind = \"direct\"\n+++\n\nBody.\n",
        )
        .unwrap();
        assert!(matches!(
            load_knowledge_package(&root),
            Err(KnowledgeError::UnresolvedSource { .. })
        ));
    }

    #[test]
    fn missing_package_toml_fails_before_touching_entities() {
        let root = temp_root("missing_package_toml_fails_before_touching_entities");
        assert!(matches!(
            load_knowledge_package(&root),
            Err(KnowledgeError::MissingPackageToml { .. })
        ));
    }
}
