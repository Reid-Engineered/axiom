use std::path::Path;

use crate::knowledge::{load_knowledge_package, ProvenanceKind};

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical")
}

#[test]
fn canonical_fixture_loads_and_matches_spec_17() {
    let package = load_knowledge_package(&fixture_root()).unwrap();

    assert_eq!(package.id.as_str(), "org.axiom.calculus_shells");
    assert_eq!(package.schema_version, 1);
    assert_eq!(package.sources.len(), 1);
    assert_eq!(package.concepts.len(), 3);
    assert_eq!(package.objectives.len(), 1);
    assert_eq!(package.examples.len(), 1);

    let vertical = package
        .concepts
        .iter()
        .find(|c| c.id.as_str() == "shell.method_vertical_axis")
        .unwrap();
    assert_eq!(vertical.related_ids.len(), 1);
    assert!(vertical
        .provenance_refs
        .iter()
        .any(|r| r.kind == ProvenanceKind::Direct));
    assert!(vertical
        .provenance_refs
        .iter()
        .any(|r| r.kind == ProvenanceKind::Derived));

    let selection = package
        .concepts
        .iter()
        .find(|c| c.id.as_str() == "shell.method_selection")
        .unwrap();
    assert_eq!(selection.prerequisite_ids.len(), 2);

    let example = &package.examples[0];
    assert!(example.solution.contains("63"));
    assert_eq!(example.hints.len(), 1);
}
