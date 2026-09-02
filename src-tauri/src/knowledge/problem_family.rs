use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::constraint::{parse_constraint, ConstraintExpr, Term};
use super::error::KnowledgeError;
use super::frontmatter::split_frontmatter;
use super::identifier::{ConceptId, ObjectiveId, ProblemFamilyId};
use super::problem_family_body::parse_problem_family_body;
use super::provenance::convert_provenance_refs;
use super::raw::{RawBound, RawCanonicalSolution, RawParameterSpec, RawProblemFamilyFrontmatter};
use super::types::{
    Bound, CanonicalSolution, DifficultyRange, GeneratorRef, Hint, ParameterSpec, ParameterType,
    ProblemFamily, ProblemFamilyStatus, ResponseType,
};

pub(crate) fn parse_problem_family_file(
    path: &Path,
    raw: &str,
) -> Result<ProblemFamily, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let frontmatter: RawProblemFamilyFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;
    let id = ProblemFamilyId::new(frontmatter.id)?;
    let entity_id = id.as_str().to_owned();
    let concept_id = ConceptId::new(frontmatter.concept_id)?;
    let objective_ids = frontmatter
        .objective_ids
        .into_iter()
        .map(ObjectiveId::new)
        .collect::<Result<_, _>>()?;
    if frontmatter.difficulty.min > frontmatter.difficulty.max {
        return Err(KnowledgeError::InvalidDifficultyRange {
            entity_id,
            min: frontmatter.difficulty.min,
            max: frontmatter.difficulty.max,
        });
    }
    crate::modules::identifier::validate_identifier(&frontmatter.generator.id).map_err(|_| {
        KnowledgeError::InvalidIdentifier {
            value: frontmatter.generator.id.clone(),
        }
    })?;
    let difficulty = DifficultyRange {
        min: frontmatter.difficulty.min,
        max: frontmatter.difficulty.max,
    };
    let generator = GeneratorRef {
        id: frontmatter.generator.id,
        version: frontmatter.generator.version,
    };
    let parameters = convert_parameters(&entity_id, frontmatter.parameters)?;
    validate_parameter_references(&entity_id, &parameters)?;
    let constraints = frontmatter
        .constraints
        .iter()
        .map(|text| parse_constraint(&entity_id, text))
        .collect::<Result<Vec<_>, _>>()?;
    for constraint in &constraints {
        validate_constraint_parameters(&entity_id, constraint, &parameters)?;
    }
    let response_type = match frontmatter.response_type.as_str() {
        "symbolic-expression" => ResponseType::SymbolicExpression,
        "numeric" => ResponseType::Numeric,
        other => {
            return Err(KnowledgeError::UnknownResponseType {
                entity_id,
                value: other.to_owned(),
            })
        }
    };
    let canonical_solution =
        convert_canonical_solution(&entity_id, response_type, frontmatter.canonical_solution)?;
    let status = match frontmatter.status.as_str() {
        "verified" => ProblemFamilyStatus::Verified,
        "needs-review" => ProblemFamilyStatus::NeedsReview,
        other => {
            return Err(KnowledgeError::UnknownProblemFamilyStatus {
                entity_id,
                value: other.to_owned(),
            })
        }
    };
    let provenance_refs = convert_provenance_refs(&entity_id, frontmatter.provenance_refs)?;
    let parsed_body = parse_problem_family_body(&entity_id, &body)?;
    if frontmatter.hints.len() != parsed_body.hint_texts.len() {
        return Err(KnowledgeError::ProblemFamilyHintCountMismatch {
            entity_id,
            frontmatter_count: frontmatter.hints.len(),
            body_count: parsed_body.hint_texts.len(),
        });
    }
    let mut seen_levels = HashSet::new();
    let hints = frontmatter
        .hints
        .into_iter()
        .zip(parsed_body.hint_texts)
        .map(|(raw_hint, text)| {
            if !seen_levels.insert(raw_hint.level) {
                return Err(KnowledgeError::DuplicateHintLevel {
                    entity_id: entity_id.clone(),
                    level: raw_hint.level,
                });
            }
            Ok(Hint {
                level: raw_hint.level,
                text,
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(ProblemFamily {
        id,
        concept_id,
        objective_ids,
        difficulty,
        generator,
        parameters,
        constraints,
        prompt: parsed_body.prompt,
        solution_structure: parsed_body.solution_structure,
        response_type,
        canonical_solution,
        hints,
        provenance_refs,
        status,
    })
}

fn convert_parameters(
    entity_id: &str,
    raw: std::collections::BTreeMap<String, RawParameterSpec>,
) -> Result<std::collections::BTreeMap<String, ParameterSpec>, KnowledgeError> {
    raw.into_iter()
        .map(|(name, raw_spec)| {
            let kind = match raw_spec.kind.as_str() {
                "integer" => ParameterType::Integer,
                "float" => ParameterType::Float,
                other => {
                    return Err(KnowledgeError::UnknownParameterType {
                        entity_id: entity_id.to_owned(),
                        value: other.to_owned(),
                    })
                }
            };
            let value = raw_spec.value.map(convert_bound);
            let min = raw_spec.min.map(convert_bound);
            let max = raw_spec.max.map(convert_bound);
            if value.is_some() && (min.is_some() || max.is_some()) {
                return Err(KnowledgeError::ParameterValueAndBoundsConflict {
                    entity_id: entity_id.to_owned(),
                    parameter: name,
                });
            }
            Ok((
                name,
                ParameterSpec {
                    kind,
                    value,
                    min,
                    max,
                    description: raw_spec.description,
                },
            ))
        })
        .collect()
}

fn convert_bound(raw: RawBound) -> Bound {
    match raw {
        RawBound::Literal(value) => Bound::Literal(value),
        RawBound::Reference { parameter, offset } => Bound::Reference { parameter, offset },
    }
}

fn validate_parameter_references(
    entity_id: &str,
    parameters: &std::collections::BTreeMap<String, ParameterSpec>,
) -> Result<(), KnowledgeError> {
    for (name, spec) in parameters {
        for bound in [&spec.value, &spec.min, &spec.max].into_iter().flatten() {
            if let Bound::Reference { parameter, .. } = bound {
                if !parameters.contains_key(parameter) {
                    return Err(KnowledgeError::DanglingParameterReference {
                        entity_id: entity_id.to_owned(),
                        parameter: name.clone(),
                        target: parameter.clone(),
                    });
                }
            }
        }
    }
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    fn visit(
        name: &str,
        parameters: &std::collections::BTreeMap<String, ParameterSpec>,
        colors: &mut HashMap<String, Color>,
        path: &mut Vec<String>,
        entity_id: &str,
    ) -> Result<(), KnowledgeError> {
        match colors.get(name) {
            Some(Color::Black) => return Ok(()),
            Some(Color::Gray) => {
                let start = path.iter().position(|item| item == name).unwrap_or(0);
                let mut cycle = path[start..].to_vec();
                cycle.push(name.to_owned());
                return Err(KnowledgeError::ParameterReferenceCycle {
                    entity_id: entity_id.to_owned(),
                    cycle,
                });
            }
            _ => {}
        }
        colors.insert(name.to_owned(), Color::Gray);
        path.push(name.to_owned());
        if let Some(spec) = parameters.get(name) {
            for bound in [&spec.value, &spec.min, &spec.max].into_iter().flatten() {
                if let Bound::Reference { parameter, .. } = bound {
                    visit(parameter, parameters, colors, path, entity_id)?;
                }
            }
        }
        path.pop();
        colors.insert(name.to_owned(), Color::Black);
        Ok(())
    }
    let mut colors = parameters
        .keys()
        .map(|key| (key.clone(), Color::White))
        .collect::<HashMap<_, _>>();
    let mut path = Vec::new();
    for name in parameters.keys() {
        if colors.get(name) == Some(&Color::White) {
            visit(name, parameters, &mut colors, &mut path, entity_id)?;
        }
    }
    Ok(())
}

fn validate_constraint_parameters(
    entity_id: &str,
    constraint: &ConstraintExpr,
    parameters: &std::collections::BTreeMap<String, ParameterSpec>,
) -> Result<(), KnowledgeError> {
    fn check(
        term: &Term,
        parameters: &std::collections::BTreeMap<String, ParameterSpec>,
        entity_id: &str,
    ) -> Result<(), KnowledgeError> {
        match term {
            Term::Param(name) if !parameters.contains_key(name) => {
                Err(KnowledgeError::ConstraintUnknownParameter {
                    entity_id: entity_id.to_owned(),
                    parameter: name.clone(),
                })
            }
            Term::BinaryOp { left, right, .. } => {
                check(left, parameters, entity_id)?;
                check(right, parameters, entity_id)
            }
            _ => Ok(()),
        }
    }
    match constraint {
        ConstraintExpr::Comparison { left, right, .. } => {
            check(left, parameters, entity_id)?;
            check(right, parameters, entity_id)
        }
        ConstraintExpr::All(expressions) => {
            for expression in expressions {
                validate_constraint_parameters(entity_id, expression, parameters)?;
            }
            Ok(())
        }
    }
}

fn convert_canonical_solution(
    entity_id: &str,
    response_type: ResponseType,
    raw: RawCanonicalSolution,
) -> Result<CanonicalSolution, KnowledgeError> {
    match (response_type, raw.expression, raw.value) {
        (ResponseType::SymbolicExpression, Some(expression), None) => {
            Ok(CanonicalSolution::Symbolic { expression })
        }
        (ResponseType::Numeric, None, Some(value)) => Ok(CanonicalSolution::Numeric { value }),
        _ => Err(KnowledgeError::ResponseTypeSolutionMismatch {
            entity_id: entity_id.to_owned(),
            response_type: match response_type {
                ResponseType::SymbolicExpression => "symbolic-expression",
                ResponseType::Numeric => "numeric",
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"+++
id = "problem.a"
concept_id = "shell.a"
difficulty = { min = 1, max = 2 }
generator = { id = "gen.a", version = 1 }
constraints = ["b <= coeff"]
response_type = "numeric"
status = "verified"

[parameters.coeff]
type = "integer"
min = 2
max = 6

[parameters.b]
type = "integer"
min = 1
max = { parameter = "coeff" }

[canonical_solution]
value = 1.0

[[provenance_refs]]
source_id = "src.a"
kind = "direct"
+++

## Prompt

P.

## Solution

S.
"#;

    #[test]
    fn valid_family_parses_constraints() {
        let family = parse_problem_family_file(Path::new("problems/problem.a.md"), VALID).unwrap();
        assert_eq!(family.constraints.len(), 1);
    }

    #[test]
    fn rejects_dangling_and_cyclic_parameter_references() {
        let dangling = VALID.replace("parameter = \"coeff\"", "parameter = \"missing\"");
        assert!(matches!(
            parse_problem_family_file(Path::new("x.md"), &dangling),
            Err(KnowledgeError::DanglingParameterReference { .. })
        ));
        let cyclic = VALID.replace("max = 6", "max = { parameter = \"b\" }");
        assert!(matches!(
            parse_problem_family_file(Path::new("x.md"), &cyclic),
            Err(KnowledgeError::ParameterReferenceCycle { .. })
        ));
    }

    #[test]
    fn rejects_unknown_constraint_parameter_and_solution_mismatch() {
        let unknown = VALID.replace("b <= coeff", "z <= coeff");
        assert!(matches!(
            parse_problem_family_file(Path::new("x.md"), &unknown),
            Err(KnowledgeError::ConstraintUnknownParameter { .. })
        ));
        let mismatch = VALID.replace("value = 1.0", "expression = \"x\"");
        assert!(matches!(
            parse_problem_family_file(Path::new("x.md"), &mismatch),
            Err(KnowledgeError::ResponseTypeSolutionMismatch { .. })
        ));
    }
}
