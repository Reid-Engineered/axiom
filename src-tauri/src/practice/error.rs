use std::error::Error;
use std::fmt;

use crate::generation::GenerationError;

#[derive(Debug)]
pub enum PracticeError {
    FamilyNotFound { family_id: String },
    AttemptNotFound { attempt_id: String },
    NoMoreHints { attempt_id: String },
    AlreadySolved { attempt_id: String },
    ResponseTypeMismatch { attempt_id: String },
    GenerationFailed(GenerationError),
    VerificationFailed(String),
    Storage(String),
}

impl fmt::Display for PracticeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FamilyNotFound { family_id } => {
                write!(formatter, "no problem family {family_id:?} exists")
            }
            Self::AttemptNotFound { attempt_id } => {
                write!(
                    formatter,
                    "no attempt {attempt_id:?} exists in this workspace"
                )
            }
            Self::NoMoreHints { attempt_id } => write!(
                formatter,
                "attempt {attempt_id:?} has no more hints to reveal"
            ),
            Self::AlreadySolved { attempt_id } => {
                write!(formatter, "attempt {attempt_id:?} is already solved")
            }
            Self::ResponseTypeMismatch { attempt_id } => write!(
                formatter,
                "response shape for attempt {attempt_id:?} does not match its response_type"
            ),
            Self::GenerationFailed(error) => write!(formatter, "generation failed: {error}"),
            Self::VerificationFailed(message) => {
                write!(formatter, "verification failed: {message}")
            }
            Self::Storage(message) => write!(formatter, "storage error: {message}"),
        }
    }
}

impl Error for PracticeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GenerationFailed(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GenerationError> for PracticeError {
    fn from(error: GenerationError) -> Self {
        Self::GenerationFailed(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::GeneratorId;

    #[test]
    fn family_not_found_displays_the_family_id() {
        let error = PracticeError::FamilyNotFound {
            family_id: "problem.nonexistent".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "no problem family \"problem.nonexistent\" exists"
        );
    }

    #[test]
    fn attempt_not_found_displays_the_attempt_id() {
        let error = PracticeError::AttemptNotFound {
            attempt_id: "attempt-1".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "no attempt \"attempt-1\" exists in this workspace"
        );
    }

    #[test]
    fn generation_failed_wraps_and_displays_the_underlying_error() {
        let underlying = GenerationError::UnknownGenerator {
            id: GeneratorId::new("gen.nonexistent").unwrap(),
        };
        let error: PracticeError = underlying.into();
        assert_eq!(
            error.to_string(),
            "generation failed: no generator is registered for gen.nonexistent"
        );
    }
}
