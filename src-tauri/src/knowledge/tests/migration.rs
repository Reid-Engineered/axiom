use std::path::Path;

use crate::knowledge::load_knowledge_package;

fn migrated_package_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package")
}

#[test]
fn migrated_calc_ii_package_loads_and_is_structurally_complete() {
    let package = load_knowledge_package(&migrated_package_root()).expect(
        "migrated knowledge-package/ must load and pass every spec §12/§13 invariant: \
                 all references resolve, the prerequisite graph is acyclic, related_ids is \
                 authored on at most one side, and every entity has at least one provenance ref",
    );

    assert_eq!(
        package.id.as_str(),
        "org.axiom.reference.calculus.cylindrical_shells"
    );
    assert_eq!(package.schema_version, 1);
    assert_eq!(
        package.sources.len(),
        1,
        "provenance.json's 11 entries must have collapsed to exactly 1 Source"
    );

    assert_eq!(package.concepts.len(), 3);
    assert_eq!(package.objectives.len(), 6);
    assert_eq!(
        package.examples.len(),
        6,
        "6 of the 7 prior problem families become Examples; pf-method-select-integral-count \
         does not migrate"
    );

    let example_ids: Vec<&str> = package.examples.iter().map(|e| e.id.as_str()).collect();
    assert!(!example_ids.iter().any(|id| id.starts_with("pf-")));
    assert!(!example_ids.contains(&"shell.example_method_select_integral_count"));

    for concept in &package.concepts {
        assert!(
            !concept.provenance_refs.is_empty(),
            "{} has no provenance_refs",
            concept.id
        );
    }
    for objective in &package.objectives {
        assert!(
            !objective.provenance_refs.is_empty(),
            "{} has no provenance_refs",
            objective.id
        );
    }
    for example in &package.examples {
        assert!(
            !example.provenance_refs.is_empty(),
            "{} has no provenance_refs",
            example.id
        );
        assert!(!example.problem.is_empty());
        assert!(!example.solution.is_empty());
    }
}

#[test]
fn no_deprecated_json_or_problem_families_artifacts_remain() {
    let root = migrated_package_root();
    assert!(!root.join("package.json").exists());
    assert!(!root.join("provenance.json").exists());
    assert!(!root.join("problem-families").exists());
    for entry in std::fs::read_dir(root.join("concepts")).unwrap() {
        let path = entry.unwrap().path();
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("md"));
    }
    for entry in std::fs::read_dir(root.join("objectives")).unwrap() {
        let path = entry.unwrap().path();
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("md"));
    }
}
