use std::path::Path;

use crate::knowledge::{
    load_knowledge_package, Bound, CanonicalSolution, ProblemFamilyStatus, ProvenanceKind,
    ResponseType,
};

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

/// Task 059 -- the first production `ProblemFamily`. Task 058 shipped the Practice command
/// layer against a package with zero families, so generation success was fixture-only; this
/// pins that the *bundled* package now carries a loadable, fully-resolved family whose
/// declared shape is the one `gen.shell_y_poly` and `math.verify` were built for.
#[test]
fn bundled_package_exposes_the_verified_shell_y_poly_problem_family() {
    let package = load_knowledge_package(&migrated_package_root()).unwrap();

    assert_eq!(package.problem_families.len(), 1);
    let family = &package.problem_families[0];
    assert_eq!(family.id.as_str(), "problem.shell_y_poly");
    assert_eq!(family.status, ProblemFamilyStatus::Verified);
    assert_eq!(family.response_type, ResponseType::SymbolicExpression);
    assert_eq!(family.generator.id.as_str(), "gen.shell_y_poly");
    assert_eq!(family.generator.version, 1);
    assert_eq!(
        family.canonical_solution,
        CanonicalSolution::Symbolic {
            expression: "2*pi*(coeff*b^3/3 - b^4/4)".to_owned()
        }
    );

    // `load_knowledge_package` already enforces that concept_id/objective_ids resolve and
    // that every objective belongs to the family's own concept (validate.rs); these pin
    // *which* production entities it attached to, which is the part a content edit can get
    // silently wrong.
    assert_eq!(family.concept_id.as_str(), "shell.method_vertical_axis");
    let objective_ids: Vec<&str> = family.objective_ids.iter().map(|id| id.as_str()).collect();
    assert_eq!(
        objective_ids,
        vec![
            "shell.setup_radius_height_y_axis",
            "shell.compute_volume_y_axis_single_curve",
        ]
    );

    // The generator is bound-driven, not constraint-driven: `b <= coeff` is expressed as a
    // parameter reference so no instance is ever rejected and resampled.
    assert!(family.constraints.is_empty());
    assert_eq!(family.parameters.len(), 3);
    assert_eq!(family.parameters["a"].value, Some(Bound::Literal(0.0)));
    assert_eq!(family.parameters["coeff"].min, Some(Bound::Literal(2.0)));
    assert_eq!(family.parameters["coeff"].max, Some(Bound::Literal(6.0)));
    assert_eq!(family.parameters["b"].min, Some(Bound::Literal(1.0)));
    assert_eq!(
        family.parameters["b"].max,
        Some(Bound::Reference {
            parameter: "coeff".to_owned(),
            offset: 0.0
        })
    );

    let levels: Vec<u32> = family.hints.iter().map(|hint| hint.level).collect();
    assert_eq!(levels, vec![1, 2, 3, 4]);
    assert!(family.hints.iter().all(|hint| !hint.text.trim().is_empty()));

    // Cites the rule it transcribes and the example it generalizes, with the
    // direct/derived split the synthesis report defines.
    let source_ids: Vec<&str> = family
        .provenance_refs
        .iter()
        .map(|reference| reference.source_id.as_str())
        .collect();
    assert_eq!(source_ids, vec!["src.openstax_calc2", "src.openstax_calc2"]);
    let labels: Vec<(&ProvenanceKind, Option<&str>)> = family
        .provenance_refs
        .iter()
        .map(|reference| {
            (
                &reference.kind,
                reference
                    .locator
                    .as_ref()
                    .and_then(|locator| locator.label.as_deref()),
            )
        })
        .collect();
    assert_eq!(
        labels,
        vec![
            (&ProvenanceKind::Direct, Some("Rule 2.6")),
            (&ProvenanceKind::Derived, Some("Example 2.13")),
        ]
    );
}
