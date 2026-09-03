use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MathVerifyError {
    UnparseableCanonicalSolution { expression: String, message: String },
}

impl fmt::Display for MathVerifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnparseableCanonicalSolution {
                expression,
                message,
            } => write!(
                formatter,
                "canonical solution expression {expression:?} could not be evaluated: {message}"
            ),
        }
    }
}

impl Error for MathVerifyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unparseable_canonical_solution_displays_expression_and_message() {
        let error = MathVerifyError::UnparseableCanonicalSolution {
            expression: "2*+pi".to_owned(),
            message: "parse error".to_owned(),
        };

        assert_eq!(
            error.to_string(),
            "canonical solution expression \"2*+pi\" could not be evaluated: parse error"
        );
    }
}
