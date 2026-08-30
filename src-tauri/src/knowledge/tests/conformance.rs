use std::fs;

use crate::knowledge::{load_knowledge_package, KnowledgeError};

use super::support::{temp_root, write_base_package};

#[test]
fn base_package_loads() {
    let root = temp_root("base_package_loads");
    write_base_package(&root);
    assert!(load_knowledge_package(&root).is_ok());
}

#[test]
fn invalid_identifier() {
    let root = temp_root("invalid_identifier");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("id = \"shell.a\"", "id = \"Shell-A\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::InvalidIdentifier { .. })
    ));
}

#[test]
fn filename_id_mismatch() {
    let root = temp_root("filename_id_mismatch");
    write_base_package(&root);
    fs::rename(
        root.join("concepts/shell.a.md"),
        root.join("concepts/shell.renamed.md"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::FilenameIdMismatch { .. })
    ));
}

#[test]
fn unresolved_concept_reference() {
    let root = temp_root("unresolved_concept_reference");
    write_base_package(&root);
    fs::write(
        root.join("objectives/shell.obj.md"),
        fs::read_to_string(root.join("objectives/shell.obj.md"))
            .unwrap()
            .replace("concept_id = \"shell.a\"", "concept_id = \"shell.missing\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnresolvedConcept { .. })
    ));
}

#[test]
fn unresolved_objective_reference() {
    let root = temp_root("unresolved_objective_reference");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md"))
            .unwrap()
            .replace(
                "objective_ids = [\"shell.obj\"]",
                "objective_ids = [\"shell.missing\"]",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnresolvedObjective { .. })
    ));
}

#[test]
fn unresolved_source() {
    let root = temp_root("unresolved_source");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("source_id = \"src.a\"", "source_id = \"src.missing\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnresolvedSource { .. })
    ));
}

#[test]
fn cross_concept_objective() {
    let root = temp_root("cross_concept_objective");
    write_base_package(&root);
    fs::write(
        root.join("objectives/shell.obj.md"),
        fs::read_to_string(root.join("objectives/shell.obj.md"))
            .unwrap()
            .replace("concept_id = \"shell.a\"", "concept_id = \"shell.b\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::CrossConceptObjective { .. })
    ));
}

#[test]
fn self_prerequisite() {
    let root = temp_root("self_prerequisite");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.b.md"),
        fs::read_to_string(root.join("concepts/shell.b.md"))
            .unwrap()
            .replace(
                "prerequisite_ids = [\"shell.a\"]",
                "prerequisite_ids = [\"shell.b\"]",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::SelfReference {
            field: "prerequisite_ids",
            ..
        })
    ));
}

#[test]
fn prerequisite_cycle() {
    let root = temp_root("prerequisite_cycle");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("prerequisite_ids = []", "prerequisite_ids = [\"shell.b\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::PrerequisiteCycle { .. })
    ));
}

#[test]
fn self_related() {
    let root = temp_root("self_related");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("related_ids = []", "related_ids = [\"shell.a\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::SelfReference {
            field: "related_ids",
            ..
        })
    ));
}

#[test]
fn reverse_double_authored_related_edge() {
    let root = temp_root("reverse_double_authored_related_edge");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("related_ids = []", "related_ids = [\"shell.b\"]"),
    )
    .unwrap();
    fs::write(
        root.join("concepts/shell.b.md"),
        fs::read_to_string(root.join("concepts/shell.b.md"))
            .unwrap()
            .replace("related_ids = []", "related_ids = [\"shell.a\"]"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::ReverseDuplicateRelated { .. })
    ));
}

#[test]
fn duplicate_id_inside_a_list() {
    let root = temp_root("duplicate_id_inside_a_list");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.b.md"),
        fs::read_to_string(root.join("concepts/shell.b.md"))
            .unwrap()
            .replace(
                "prerequisite_ids = [\"shell.a\"]",
                "prerequisite_ids = [\"shell.a\", \"shell.a\"]",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::DuplicateReferenceInList { .. })
    ));
}

#[test]
fn missing_provenance() {
    let root = temp_root("missing_provenance");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace(
                "[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n",
                "",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::MissingProvenance { .. })
    ));
}

#[test]
fn unknown_provenance_kind() {
    let root = temp_root("unknown_provenance_kind");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace("kind = \"direct\"", "kind = \"inferred\""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnknownProvenanceKind { .. })
    ));
}

#[test]
fn duplicate_provenance_ref() {
    let root = temp_root("duplicate_provenance_ref");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace(
                "+++\n\nConcept A body.",
                "\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nConcept A body.",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::DuplicateProvenanceRef { .. })
    ));
}

#[test]
fn empty_source_locator() {
    let root = temp_root("empty_source_locator");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        fs::read_to_string(root.join("concepts/shell.a.md"))
            .unwrap()
            .replace(
                "kind = \"direct\"\n",
                "kind = \"direct\"\n[provenance_refs.locator]\n",
            ),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::EmptySourceLocator { .. })
    ));
}

#[test]
fn unsupported_schema_version() {
    let root = temp_root("unsupported_schema_version");
    write_base_package(&root);
    fs::write(
        root.join("package.toml"),
        fs::read_to_string(root.join("package.toml"))
            .unwrap()
            .replace("schema_version = 1", "schema_version = 2"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnsupportedSchemaVersion { found: 2 })
    ));
}

#[test]
fn unknown_schema_key() {
    let root = temp_root("unknown_schema_key");
    write_base_package(&root);
    fs::write(
        root.join("package.toml"),
        fs::read_to_string(root.join("package.toml")).unwrap() + "unexpected = \"typo\"\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::TomlSyntax { .. })
    ));
}

#[test]
fn missing_example_problem() {
    let root = temp_root("missing_example_problem");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md"))
            .unwrap()
            .replace("## Problem\n\nP.\n\n", ""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::MissingExampleSection {
            section: "Problem",
            ..
        })
    ));
}

#[test]
fn missing_example_solution() {
    let root = temp_root("missing_example_solution");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md"))
            .unwrap()
            .replace("## Solution\n\nS.\n", ""),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::MissingExampleSection {
            section: "Solution",
            ..
        })
    ));
}

#[test]
fn duplicate_example_heading() {
    let root = temp_root("duplicate_example_heading");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md")).unwrap() + "\n## Solution\n\nS2.\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::DuplicateExampleSection { .. })
    ));
}

#[test]
fn unknown_example_heading() {
    let root = temp_root("unknown_example_heading");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md")).unwrap() + "\n## Notes\n\nExtra.\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnknownExampleSection { .. })
    ));
}

#[test]
fn non_list_hints_content() {
    let root = temp_root("non_list_hints_content");
    write_base_package(&root);
    fs::write(
        root.join("examples/shell.ex.md"),
        fs::read_to_string(root.join("examples/shell.ex.md")).unwrap()
            + "\n## Hints\n\nJust a paragraph.\n",
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::InvalidHintLine { .. })
    ));
}

#[test]
fn malformed_toml_frontmatter() {
    let root = temp_root("malformed_toml_frontmatter");
    write_base_package(&root);
    fs::write(
        root.join("concepts/shell.a.md"),
        "+++\nid = \"shell.a\nname = \"A\"\n+++\n\nBody.\n", // unterminated string
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::TomlSyntax { .. })
    ));
}

#[test]
fn byte_order_mark() {
    let root = temp_root("byte_order_mark");
    write_base_package(&root);
    let content = fs::read_to_string(root.join("concepts/shell.a.md")).unwrap();
    fs::write(
        root.join("concepts/shell.a.md"),
        format!("\u{FEFF}{content}"),
    )
    .unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::Bom { .. })
    ));
}

#[test]
fn nested_entity_directory() {
    let root = temp_root("nested_entity_directory");
    write_base_package(&root);
    fs::create_dir_all(root.join("concepts/nested")).unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::NestedEntityDirectory { .. })
    ));
}

#[test]
fn unknown_entity_file_extension() {
    let root = temp_root("unknown_entity_file_extension");
    write_base_package(&root);
    fs::write(root.join("concepts/notes.txt"), "stray file").unwrap();
    assert!(matches!(
        load_knowledge_package(&root),
        Err(KnowledgeError::UnexpectedEntityFile { .. })
    ));
}

#[test]
fn valid_crlf_fixture_is_accepted() {
    let root = temp_root("valid_crlf_fixture_is_accepted");
    write_base_package(&root);
    let crlf = fs::read_to_string(root.join("concepts/shell.a.md"))
        .unwrap()
        .replace('\n', "\r\n");
    fs::write(root.join("concepts/shell.a.md"), crlf).unwrap();
    assert!(load_knowledge_package(&root).is_ok());
}

#[test]
fn root_documentation_files_are_ignored() {
    let root = temp_root("root_documentation_files_are_ignored");
    write_base_package(&root);
    fs::write(
        root.join("synthesis-report.md"),
        "Human rationale, not schema data.\n",
    )
    .unwrap();
    fs::write(root.join("README.md"), "Non-schema documentation.\n").unwrap();
    assert!(load_knowledge_package(&root).is_ok());
}
