mod concept;
mod discover;
mod error;
mod example;
mod example_body;
mod frontmatter;
mod identifier;
mod loader;
mod objective;
mod package;
mod provenance;
mod raw;
mod relationships;
mod types;
mod validate;

pub use error::KnowledgeError;
pub use identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};
pub use loader::load_knowledge_package;
pub use relationships::related_concepts;
pub use types::{
    Concept, Example, KnowledgePackage, Objective, ProvenanceKind, ProvenanceRef, Source,
    SourceLocator,
};

#[cfg(test)]
mod tests;
