use std::fs;
use std::path::{Path, PathBuf};

use crate::knowledge::{load_knowledge_package, KnowledgeError};

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
    expected: fn(&KnowledgeError) -> bool,
) {
    let root = temp_root(case);
    copy_tree(&canonical_root(), &root);
    let path = root.join("problems/problem.shell_y_poly.md");
    let contents = fs::read_to_string(&path).unwrap();
    assert!(
        contents.contains(original),
        "mutation source missing for {case}"
    );
    fs::write(path, contents.replacen(original, replacement, 1)).unwrap();
    let error = load_knowledge_package(&root).unwrap_err();
    assert!(expected(&error), "unexpected error for {case}: {error}");
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
        "[[hints]]\nlevel = 4",
        "[[hints]]\nlevel = 4\n\n[[hints]]\nlevel = 5",
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
