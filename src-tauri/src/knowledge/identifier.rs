use serde::{Deserialize, Serialize};

use crate::modules::identifier::validate_identifier;

use super::error::KnowledgeError;

macro_rules! knowledge_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeError> {
                let value = value.into();
                validate_identifier(&value).map_err(|_| KnowledgeError::InvalidIdentifier {
                    value: value.clone(),
                })?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

knowledge_id!(KnowledgePackageId);
knowledge_id!(ConceptId);
knowledge_id!(ObjectiveId);
knowledge_id!(ExampleId);
knowledge_id!(SourceId);
knowledge_id!(ProblemFamilyId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identifiers_are_accepted() {
        for valid in [
            "shell.method_vertical_axis",
            "org.axiom.calculus_shells",
            "a.b",
            "a0.b_1",
        ] {
            assert!(ConceptId::new(valid).is_ok(), "{valid} should be valid");
            assert!(SourceId::new(valid).is_ok(), "{valid} should be valid");
        }
    }

    #[test]
    fn invalid_identifiers_are_rejected() {
        for invalid in [
            "Shell.Method",
            "shell",
            "shell..method",
            ".shell",
            "shell.",
            "shell.Method",
            "shell-method.vertical",
        ] {
            assert!(matches!(
                ConceptId::new(invalid),
                Err(KnowledgeError::InvalidIdentifier { .. })
            ));
        }
    }

    #[test]
    fn distinct_entity_kinds_may_share_a_lexical_value() {
        assert!(ConceptId::new("shell.basic").is_ok());
        assert!(ObjectiveId::new("shell.basic").is_ok());
    }
}
