use std::fs;
use std::path::{Path, PathBuf};

use crate::knowledge::{
    load_knowledge_package, CanonicalSolution, KnowledgeError, KnowledgePackage,
};

use super::support::temp_root;

fn canonical_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn assert_mutation(
    case: &str,
    original: &str,
    replacement: &str,
    expected: impl Fn(&KnowledgeError) -> bool,
) {
    let error = load_mutated_family(case, |contents| {
        assert!(
            contents.contains(original),
            "mutation source missing for {case}"
        );
        contents.replacen(original, replacement, 1)
    })
    .unwrap_err();
    assert!(expected(&error), "unexpected error for {case}: {error}");
}

fn load_mutated_family(
    case: &str,
    mutate: impl FnOnce(String) -> String,
) -> Result<KnowledgePackage, KnowledgeError> {
    let root = temp_root(case);
    copy_tree(&canonical_root(), &root);
    let path = root.join("problems/problem.shell_y_poly.md");
    let contents = fs::read_to_string(&path).unwrap();
    fs::write(path, mutate(contents)).unwrap();
    load_knowledge_package(&root)
}

#[test]
fn dangling_parameter_reference_is_rejected_end_to_end() {
    assert_mutation(
        "problem_dangling",
        "parameter = \"coeff\"",
        "parameter = \"missing\"",
        |error| matches!(error, KnowledgeError::DanglingParameterReference { .. }),
    );
}

#[test]
fn parameter_cycle_is_rejected_end_to_end() {
    assert_mutation(
        "problem_cycle",
        "max = 6",
        "max = { parameter = \"b\" }",
        |error| matches!(error, KnowledgeError::ParameterReferenceCycle { .. }),
    );
}

#[test]
fn unknown_constraint_parameter_is_rejected_end_to_end() {
    assert_mutation(
        "problem_constraint",
        "status = \"verified\"",
        "status = \"verified\"\nconstraints = [\"z <= coeff\"]",
        |error| matches!(error, KnowledgeError::ConstraintUnknownParameter { .. }),
    );
}

#[test]
fn solution_mismatch_is_rejected_end_to_end() {
    assert_mutation(
        "problem_solution",
        "expression = \"2*pi*(coeff*b^3/3 - b^4/4)\"",
        "value = 1.0",
        |error| matches!(error, KnowledgeError::ResponseTypeSolutionMismatch { .. }),
    );
}

#[test]
fn invalid_difficulty_is_rejected_end_to_end() {
    assert_mutation(
        "problem_difficulty",
        "difficulty = { min = 1, max = 2 }",
        "difficulty = { min = 3, max = 2 }",
        |error| matches!(error, KnowledgeError::InvalidDifficultyRange { .. }),
    );
}

#[test]
fn hint_count_mismatch_is_rejected_end_to_end() {
    assert_mutation(
        "problem_hint_count",
        "level = 4",
        "level = 4\n\n[[hints]]\nlevel = 5",
        |error| matches!(error, KnowledgeError::ProblemFamilyHintCountMismatch { .. }),
    );
}

#[test]
fn duplicate_hint_level_is_rejected_end_to_end() {
    assert_mutation("problem_hint_level", "level = 2", "level = 1", |error| {
        matches!(error, KnowledgeError::DuplicateHintLevel { .. })
    });
}

#[test]
fn unresolved_concept_is_rejected_end_to_end() {
    assert_mutation(
        "problem_concept",
        "concept_id = \"shell.method_vertical_axis\"",
        "concept_id = \"shell.missing\"",
        |error| matches!(error, KnowledgeError::UnresolvedConcept { .. }),
    );
}

#[test]
fn unresolved_objective_is_rejected_end_to_end() {
    assert_mutation(
        "problem_objective",
        "objective_ids = [\"shell.setup_radius_height\"]",
        "objective_ids = [\"shell.missing\"]",
        |error| matches!(error, KnowledgeError::UnresolvedObjective { .. }),
    );
}

#[test]
fn cross_concept_objective_is_rejected_end_to_end() {
    assert_mutation(
        "problem_cross_concept",
        "concept_id = \"shell.method_vertical_axis\"",
        "concept_id = \"shell.method_horizontal_axis\"",
        |error| {
            matches!(
                error,
                KnowledgeError::ProblemFamilyCrossConceptObjective { .. }
            )
        },
    );
}

#[test]
fn empty_or_omitted_objectives_are_rejected_end_to_end() {
    for (case, replacement) in [
        ("problem_empty_objectives", "objective_ids = []"),
        ("problem_omitted_objectives", ""),
    ] {
        assert_mutation(
            case,
            "objective_ids = [\"shell.setup_radius_height\"]",
            replacement,
            |error| matches!(error, KnowledgeError::MissingProblemFamilyObjectives { entity_id } if entity_id == "problem.shell_y_poly"),
        );
    }
}

#[test]
fn zero_hint_level_is_rejected_end_to_end() {
    assert_mutation(
        "problem_zero_hint_level",
        "level = 1",
        "level = 0",
        |error| matches!(error, KnowledgeError::InvalidHintLevel { entity_id, level: 0 } if entity_id == "problem.shell_y_poly"),
    );
}

#[test]
fn non_finite_bounds_are_rejected_end_to_end() {
    for value in ["nan", "+nan", "-nan", "inf", "+inf", "-inf"] {
        for (field, parameter, original) in [
            ("value", "a", "value = 0"),
            ("min", "coeff", "min = 2"),
            ("max", "b", "max = { parameter = \"coeff\" }"),
        ] {
            for bound in [
                value.to_owned(),
                format!("{{ parameter = \"coeff\", offset = {value} }}"),
            ] {
                assert_mutation(
                    "problem_non_finite_bound",
                    original,
                    &format!("{field} = {bound}"),
                    |error| matches!(error, KnowledgeError::NonFiniteParameterBound { entity_id, parameter: actual_parameter, field: actual_field } if entity_id == "problem.shell_y_poly" && actual_parameter == parameter && *actual_field == field),
                );
            }
        }
    }
}

#[test]
fn deeply_nested_constraint_is_rejected_end_to_end() {
    let constraint = format!("b >= {}5{}", "(".repeat(10_000), ")".repeat(10_000));
    assert_mutation(
        "problem_deep_constraint",
        "status = \"verified\"",
        &format!("status = \"verified\"\nconstraints = [\"{constraint}\"]"),
        |error| matches!(error, KnowledgeError::ConstraintParseError { message, .. } if message.contains("maximum depth")),
    );
}

#[test]
fn long_flat_operator_chains_are_rejected_end_to_end() {
    for operator in ["+", "-", "*", "/"] {
        let constraint = format!("b >= {}1", format!("1{operator}").repeat(199_999));
        assert_mutation(
            "problem_flat_constraint",
            "status = \"verified\"",
            &format!("status = \"verified\"\nconstraints = [\"{constraint}\"]"),
            |error| {
                matches!(error, KnowledgeError::ConstraintParseError { entity_id, message, .. }
                if entity_id == "problem.shell_y_poly" && message.contains("tree exceeds maximum depth"))
            },
        );
    }
}

#[test]
fn numeric_canonical_solution_must_be_finite_end_to_end() {
    for value in [
        "nan", "+nan", "-nan", "inf", "+inf", "-inf", "-5.5", "0.0", "5.5",
    ] {
        let result = load_mutated_family("problem_numeric_solution", |contents| {
            contents
                .replace(
                    "response_type = \"symbolic-expression\"",
                    "response_type = \"numeric\"",
                )
                .replace(
                    "expression = \"2*pi*(coeff*b^3/3 - b^4/4)\"",
                    &format!("value = {value}"),
                )
        });
        match value.parse::<f64>() {
            Ok(number) if number.is_finite() => {
                assert_eq!(
                    result.unwrap().problem_families[0].canonical_solution,
                    CanonicalSolution::Numeric { value: number }
                );
            }
            _ => assert!(
                matches!(result, Err(KnowledgeError::NonFiniteCanonicalSolution { entity_id }) if entity_id == "problem.shell_y_poly"),
                "accepted {value}"
            ),
        }
    }
}

#[test]
fn unknown_bound_reference_fields_are_rejected_end_to_end() {
    for table in [
        "{ parameter = \"coeff\", offst = 1 }",
        "{ parameter = \"coeff\", offset = 1, extra = 2 }",
    ] {
        assert_mutation(
            "problem_unknown_bound_field",
            "{ parameter = \"coeff\" }",
            table,
            |error| matches!(error, KnowledgeError::TomlSyntax { .. }),
        );
    }
}

#[test]
fn parameter_names_must_be_referenceable_end_to_end() {
    for name in ["and", "", "1a", "a.b", "a-b", "a b", " a", "α"] {
        assert_mutation(
            "problem_invalid_parameter_name",
            "[parameters.a]",
            &format!("[parameters.\"{name}\"]"),
            |error| {
                matches!(error, KnowledgeError::InvalidParameterName { entity_id, parameter }
                if entity_id == "problem.shell_y_poly" && parameter == name)
            },
        );
    }
    for name in ["a", "A1", "_", "_a", "a_1", "and1"] {
        let package = load_mutated_family("problem_valid_parameter_name", |contents| {
            contents
                .replace("[parameters.a]", &format!("[parameters.{name}]"))
                .replace(
                    "status = \"verified\"",
                    &format!(
                        "status = \"verified\"\nconstraints = [\"{name} >= 0 and b >= {name}\"]"
                    ),
                )
        })
        .unwrap();
        assert!(package.problem_families[0].parameters.contains_key(name));
    }
}

#[test]
fn hint_levels_must_ascend_in_body_order_end_to_end() {
    let result = load_mutated_family("problem_out_of_order_hints", |contents| {
        contents.replace("level = 1", "level = 5")
    });
    assert!(
        matches!(result, Err(KnowledgeError::OutOfOrderHintLevel { entity_id, previous_level: 5, level: 2 }) if entity_id == "problem.shell_y_poly")
    );
    let original = load_mutated_family("problem_original_hints", |contents| contents).unwrap();
    let gapped = load_mutated_family("problem_gapped_hints", |contents| {
        contents.replace("level = 4", "level = 8")
    })
    .unwrap();
    assert_eq!(
        gapped.problem_families[0]
            .hints
            .iter()
            .map(|hint| hint.level)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 8]
    );
    assert_eq!(
        gapped.problem_families[0]
            .hints
            .iter()
            .map(|hint| &hint.text)
            .collect::<Vec<_>>(),
        original.problem_families[0]
            .hints
            .iter()
            .map(|hint| &hint.text)
            .collect::<Vec<_>>()
    );
}

#[test]
fn generator_ids_are_validated_end_to_end() {
    for id in ["generator", "gen.bad-id", "Gen.a"] {
        assert_mutation(
            "problem_invalid_generator",
            "gen.shell_y_poly",
            id,
            |error| matches!(error, KnowledgeError::InvalidIdentifier { value } if value == id),
        );
    }
    let package = load_mutated_family("problem_valid_generator", |contents| contents).unwrap();
    assert_eq!(
        package.problem_families[0].generator.id.as_str(),
        "gen.shell_y_poly"
    );
}
