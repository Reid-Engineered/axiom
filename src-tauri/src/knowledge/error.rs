use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum KnowledgeError {
    InvalidIdentifier { value: String },
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { value } => {
                write!(f, "invalid Knowledge identifier: {value}")
            }
        }
    }
}

impl Error for KnowledgeError {}
