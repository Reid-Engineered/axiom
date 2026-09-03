use std::error::Error;
use std::fmt;

use crate::knowledge::{GeneratorId, ProblemFamilyId};

#[derive(Debug, PartialEq)]
pub enum GenerationError {
    UnknownGenerator {
        id: GeneratorId,
    },
    UnderspecifiedParameter {
        family_id: ProblemFamilyId,
        parameter: String,
    },
    InvalidParameterBounds {
        family_id: ProblemFamilyId,
        parameter: String,
        min: f64,
        max: f64,
    },
    ConstraintsUnsatisfiable {
        family_id: ProblemFamilyId,
        attempts: u32,
    },
}

impl fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownGenerator { id } => {
                write!(formatter, "no generator is registered for {id}")
            }
            Self::UnderspecifiedParameter {
                family_id,
                parameter,
            } => write!(
                formatter,
                "{family_id}: parameter {parameter:?} has neither a fixed value nor both a \
                 min and max bound, so it cannot be sampled"
            ),
            Self::InvalidParameterBounds {
                family_id,
                parameter,
                min,
                max,
            } => write!(
                formatter,
                "{family_id}: parameter {parameter:?} has invalid resolved bounds: \
                 min {min} is greater than max {max}"
            ),
            Self::ConstraintsUnsatisfiable {
                family_id,
                attempts,
            } => write!(
                formatter,
                "{family_id}: no combination of sampled parameters satisfied every declared \
                 constraint after {attempts} attempts"
            ),
        }
    }
}

impl Error for GenerationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_generator_displays_the_generator_id() {
        let error = GenerationError::UnknownGenerator {
            id: GeneratorId::new("gen.nonexistent").unwrap(),
        };
        assert_eq!(
            error.to_string(),
            "no generator is registered for gen.nonexistent"
        );
    }

    #[test]
    fn underspecified_parameter_displays_family_and_parameter() {
        let error = GenerationError::UnderspecifiedParameter {
            family_id: ProblemFamilyId::new("problem.test").unwrap(),
            parameter: "x".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "problem.test: parameter \"x\" has neither a fixed value nor both a min and max \
             bound, so it cannot be sampled"
        );
    }

    #[test]
    fn constraints_unsatisfiable_displays_family_and_attempt_count() {
        let error = GenerationError::ConstraintsUnsatisfiable {
            family_id: ProblemFamilyId::new("problem.test").unwrap(),
            attempts: 1000,
        };
        assert_eq!(
            error.to_string(),
            "problem.test: no combination of sampled parameters satisfied every declared \
             constraint after 1000 attempts"
        );
    }

    #[test]
    fn invalid_parameter_bounds_displays_family_parameter_and_bounds() {
        let error = GenerationError::InvalidParameterBounds {
            family_id: ProblemFamilyId::new("problem.test").unwrap(),
            parameter: "x".to_owned(),
            min: 6.0,
            max: 5.0,
        };
        assert_eq!(
            error.to_string(),
            "problem.test: parameter \"x\" has invalid resolved bounds: min 6 is greater than max 5"
        );
    }
}
