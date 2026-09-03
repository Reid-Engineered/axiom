mod error;
mod rng;
mod sampling;
mod template;

pub use error::GenerationError;

use crate::knowledge::{CanonicalSolution, ProblemFamily, ProblemInstance, ResolvedSolution};

use rng::DeterministicRng;

pub fn generate_problem_instance(
    family: &ProblemFamily,
    seed: u64,
) -> Result<ProblemInstance, GenerationError> {
    match family.generator.id.as_str() {
        "gen.shell_y_poly" => generate_generic(family, seed),
        _ => Err(GenerationError::UnknownGenerator {
            id: family.generator.id.clone(),
        }),
    }
}

fn generate_generic(family: &ProblemFamily, seed: u64) -> Result<ProblemInstance, GenerationError> {
    let mut rng = DeterministicRng::new(seed);
    let resolved_parameters = sampling::resolve_parameters(family, &mut rng)?;

    let prompt = template::substitute_braces(&family.prompt, &resolved_parameters);
    let hints = family
        .hints
        .iter()
        .map(|hint| template::substitute_braces(&hint.text, &resolved_parameters))
        .collect();

    let canonical_solution = match &family.canonical_solution {
        CanonicalSolution::Numeric { value } => ResolvedSolution::Numeric(*value),
        CanonicalSolution::Symbolic { expression } => ResolvedSolution::Symbolic(
            template::substitute_identifiers(expression, &resolved_parameters),
        ),
    };

    Ok(ProblemInstance {
        family_id: family.id.clone(),
        seed,
        resolved_parameters,
        prompt,
        canonical_solution,
        hints,
    })
}

#[cfg(test)]
mod unit_tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use crate::knowledge::{load_knowledge_package, GeneratorId, GeneratorRef};

    use super::*;

    fn shell_y_poly_family() -> ProblemFamily {
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical");
        let package = load_knowledge_package(&fixture_root).unwrap();
        package
            .problem_families
            .into_iter()
            .find(|family| family.id.as_str() == "problem.shell_y_poly")
            .expect("fixture must contain problem.shell_y_poly")
    }

    #[test]
    fn unknown_generator_id_is_rejected() {
        let mut family = shell_y_poly_family();
        family.generator = GeneratorRef {
            id: GeneratorId::new("gen.nonexistent").unwrap(),
            version: 1,
        };
        assert!(matches!(
            generate_problem_instance(&family, 1),
            Err(GenerationError::UnknownGenerator { .. })
        ));
    }

    #[test]
    fn same_seed_produces_byte_identical_instances() {
        let family = shell_y_poly_family();
        let first = generate_problem_instance(&family, 42).unwrap();
        let second = generate_problem_instance(&family, 42).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn seed_actually_affects_the_sampled_parameters() {
        let family = shell_y_poly_family();
        let distinct_outcomes: BTreeSet<Vec<(String, i64)>> = (0..100u64)
            .map(|seed| {
                generate_problem_instance(&family, seed)
                    .unwrap()
                    .resolved_parameters
                    .into_iter()
                    .map(|(name, value)| (name, value as i64))
                    .collect()
            })
            .collect();
        assert!(
            distinct_outcomes.len() > 1,
            "100 different seeds all produced the same resolved parameters"
        );
    }

    #[test]
    fn produced_instance_has_no_unsubstituted_parameter_placeholders_or_identifiers() {
        let family = shell_y_poly_family();
        let instance = generate_problem_instance(&family, 7).unwrap();

        for name in family.parameters.keys() {
            let placeholder = format!("{{{name}}}");
            assert!(!instance.prompt.contains(&placeholder));
            for hint in &instance.hints {
                assert!(!hint.contains(&placeholder));
            }
        }

        let ResolvedSolution::Symbolic(expression) = &instance.canonical_solution else {
            panic!("problem.shell_y_poly is a SymbolicExpression family");
        };
        for name in family.parameters.keys() {
            assert!(
                !expression
                    .split(|character: char| {
                        !character.is_ascii_alphanumeric() && character != '_'
                    })
                    .any(|token| token == name),
                "expression {expression:?} still contains bare parameter {name:?}"
            );
        }
    }

    #[test]
    fn canonical_solution_expression_is_parseable_by_mathcore() {
        let family = shell_y_poly_family();
        let instance = generate_problem_instance(&family, 7).unwrap();
        let ResolvedSolution::Symbolic(expression) = &instance.canonical_solution else {
            panic!("problem.shell_y_poly is a SymbolicExpression family");
        };
        mathcore::MathCore::new()
            .calculate(expression)
            .unwrap_or_else(|error| panic!("{expression:?} did not parse: {error}"));
    }
}

#[cfg(test)]
mod tests;
