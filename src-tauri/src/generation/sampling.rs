use std::collections::BTreeMap;

use crate::knowledge::{Bound, ParameterSpec, ParameterType, ProblemFamily};

use super::error::GenerationError;
use super::rng::DeterministicRng;

const MAX_RESAMPLE_ATTEMPTS: u32 = 1000;

pub(crate) fn resolve_parameters(
    family: &ProblemFamily,
    rng: &mut DeterministicRng,
) -> Result<BTreeMap<String, f64>, GenerationError> {
    for _ in 0..MAX_RESAMPLE_ATTEMPTS {
        let mut resolved = BTreeMap::new();
        for name in family.parameters.keys() {
            resolve_parameter(name, &family.parameters, &mut resolved, rng, family)?;
        }
        if family
            .constraints
            .iter()
            .all(|constraint| constraint.holds(&resolved))
        {
            return Ok(resolved);
        }
    }

    Err(GenerationError::ConstraintsUnsatisfiable {
        family_id: family.id.clone(),
        attempts: MAX_RESAMPLE_ATTEMPTS,
    })
}

fn resolve_parameter(
    name: &str,
    parameters: &BTreeMap<String, ParameterSpec>,
    resolved: &mut BTreeMap<String, f64>,
    rng: &mut DeterministicRng,
    family: &ProblemFamily,
) -> Result<f64, GenerationError> {
    if let Some(value) = resolved.get(name) {
        return Ok(*value);
    }

    let spec = &parameters[name];
    let value = if let Some(bound) = &spec.value {
        resolve_bound(bound, parameters, resolved, rng, family)?
    } else {
        match (&spec.min, &spec.max) {
            (Some(min), Some(max)) => {
                let min = resolve_bound(min, parameters, resolved, rng, family)?;
                let max = resolve_bound(max, parameters, resolved, rng, family)?;
                match spec.kind {
                    ParameterType::Integer => {
                        rng.sample_integer(min.round() as i64, max.round() as i64) as f64
                    }
                    ParameterType::Float => rng.sample_float(min, max),
                }
            }
            _ => {
                return Err(GenerationError::UnderspecifiedParameter {
                    family_id: family.id.clone(),
                    parameter: name.to_owned(),
                });
            }
        }
    };
    resolved.insert(name.to_owned(), value);
    Ok(value)
}

fn resolve_bound(
    bound: &Bound,
    parameters: &BTreeMap<String, ParameterSpec>,
    resolved: &mut BTreeMap<String, f64>,
    rng: &mut DeterministicRng,
    family: &ProblemFamily,
) -> Result<f64, GenerationError> {
    match bound {
        Bound::Literal(value) => Ok(*value),
        Bound::Reference { parameter, offset } => {
            Ok(resolve_parameter(parameter, parameters, resolved, rng, family)? + offset)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{
        CanonicalSolution, ConceptId, ConstraintExpr, DifficultyRange, GeneratorId, GeneratorRef,
        ObjectiveId, ProblemFamilyId, ProblemFamilyStatus, ProvenanceKind, ProvenanceRef,
        ResponseType, SourceId,
    };

    fn minimal_family(
        parameters: BTreeMap<String, ParameterSpec>,
        constraints: Vec<ConstraintExpr>,
    ) -> ProblemFamily {
        ProblemFamily {
            id: ProblemFamilyId::new("problem.test_fixture").unwrap(),
            concept_id: ConceptId::new("concept.test").unwrap(),
            objective_ids: vec![ObjectiveId::new("objective.test").unwrap()],
            difficulty: DifficultyRange { min: 1, max: 1 },
            generator: GeneratorRef {
                id: GeneratorId::new("gen.test").unwrap(),
                version: 1,
            },
            parameters,
            constraints,
            prompt: "Prompt.".to_owned(),
            solution_structure: "Solution.".to_owned(),
            response_type: ResponseType::Numeric,
            canonical_solution: CanonicalSolution::Numeric { value: 1.0 },
            hints: Vec::new(),
            provenance_refs: vec![ProvenanceRef {
                source_id: SourceId::new("src.test").unwrap(),
                locator: None,
                kind: ProvenanceKind::Direct,
            }],
            status: ProblemFamilyStatus::Verified,
        }
    }

    fn fixed(value: f64) -> ParameterSpec {
        ParameterSpec {
            kind: ParameterType::Integer,
            value: Some(Bound::Literal(value)),
            min: None,
            max: None,
            description: None,
        }
    }

    fn bounded(kind: ParameterType, min: Bound, max: Bound) -> ParameterSpec {
        ParameterSpec {
            kind,
            value: None,
            min: Some(min),
            max: Some(max),
            description: None,
        }
    }

    #[test]
    fn fixed_value_parameter_resolves_without_consuming_rng_state() {
        let mut parameters = BTreeMap::new();
        parameters.insert("a".to_owned(), fixed(7.0));
        let family = minimal_family(parameters, Vec::new());

        let mut rng_before = DeterministicRng::new(1);
        let baseline = rng_before.next_u64();

        let mut rng = DeterministicRng::new(1);
        let resolved = resolve_parameters(&family, &mut rng).unwrap();
        assert_eq!(resolved["a"], 7.0);
        assert_eq!(
            rng.next_u64(),
            baseline,
            "fixed value must not draw from the RNG"
        );
    }

    #[test]
    fn reference_chain_resolves_in_dependency_order() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "coeff".to_owned(),
            bounded(
                ParameterType::Integer,
                Bound::Literal(2.0),
                Bound::Literal(2.0),
            ),
        );
        parameters.insert(
            "b".to_owned(),
            bounded(
                ParameterType::Integer,
                Bound::Literal(0.0),
                Bound::Reference {
                    parameter: "coeff".to_owned(),
                    offset: 0.0,
                },
            ),
        );
        parameters.insert(
            "c".to_owned(),
            bounded(
                ParameterType::Integer,
                Bound::Reference {
                    parameter: "b".to_owned(),
                    offset: 1.0,
                },
                Bound::Reference {
                    parameter: "b".to_owned(),
                    offset: 1.0,
                },
            ),
        );
        let family = minimal_family(parameters, Vec::new());

        let mut rng = DeterministicRng::new(1);
        let resolved = resolve_parameters(&family, &mut rng).unwrap();
        assert_eq!(resolved["coeff"], 2.0);
        assert_eq!(resolved["c"], resolved["b"] + 1.0);
    }

    #[test]
    fn underspecified_parameter_is_rejected() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "x".to_owned(),
            ParameterSpec {
                kind: ParameterType::Integer,
                value: None,
                min: None,
                max: None,
                description: None,
            },
        );
        let family = minimal_family(parameters, Vec::new());

        let mut rng = DeterministicRng::new(1);
        assert!(matches!(
            resolve_parameters(&family, &mut rng),
            Err(GenerationError::UnderspecifiedParameter { parameter, .. }) if parameter == "x"
        ));
    }

    #[test]
    fn unsatisfiable_constraint_fails_after_exactly_max_attempts() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "x".to_owned(),
            bounded(
                ParameterType::Integer,
                Bound::Literal(1.0),
                Bound::Literal(1.0),
            ),
        );
        let constraint = crate::knowledge::parse_constraint("test", "x > 1").unwrap();
        let family = minimal_family(parameters, vec![constraint]);

        let mut rng = DeterministicRng::new(1);
        assert!(matches!(
            resolve_parameters(&family, &mut rng),
            Err(GenerationError::ConstraintsUnsatisfiable { attempts, .. })
                if attempts == MAX_RESAMPLE_ATTEMPTS
        ));
    }

    #[test]
    fn satisfiable_narrow_constraint_eventually_succeeds() {
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "x".to_owned(),
            bounded(
                ParameterType::Integer,
                Bound::Literal(1.0),
                Bound::Literal(2.0),
            ),
        );
        let constraint = crate::knowledge::parse_constraint("test", "x == 1").unwrap();
        let family = minimal_family(parameters, vec![constraint]);

        let mut first_attempt = DeterministicRng::new(0);
        assert_eq!(first_attempt.sample_integer(1, 2), 2);

        let mut rng = DeterministicRng::new(0);
        let resolved = resolve_parameters(&family, &mut rng).unwrap();
        assert_eq!(resolved["x"], 1.0);
    }
}
