use serde::{Deserialize, Serialize};

use super::manifest::ManifestError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(String);

impl ModuleId {
    pub fn new(value: impl Into<String>) -> Result<Self, ManifestError> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl CapabilityId {
    pub fn new(value: impl Into<String>) -> Result<Self, ManifestError> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_identifier(value: &str) -> Result<(), ManifestError> {
    let mut segment_count = 0;

    for segment in value.split('.') {
        segment_count += 1;
        let mut characters = segment.chars();
        let valid_first = characters
            .next()
            .is_some_and(|character| character.is_ascii_lowercase());
        let valid_rest = characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });

        if !valid_first || !valid_rest {
            return Err(ManifestError::InvalidIdentifier {
                value: value.to_owned(),
            });
        }
    }

    if segment_count < 2 {
        return Err(ManifestError::InvalidIdentifier {
            value: value.to_owned(),
        });
    }

    Ok(())
}
