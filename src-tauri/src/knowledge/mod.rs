// Several tasks in this plan produce pub(crate) items with no production caller until a
// later task (see docs/superpowers/plans/2026-08-30-knowledge-package-v1.md Task 3 Step 3
// for the full reasoning). This is removed in Task 12, once the loader wires everything
// together and every item has a real caller.
#![allow(dead_code)]

mod concept;
mod discover;
mod error;
mod example;
mod example_body;
mod frontmatter;
mod identifier;
mod objective;
mod package;
mod provenance;
mod raw;
mod types;
mod validate;

pub use error::KnowledgeError;
pub use identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};
pub use types::{
    Concept, Example, KnowledgePackage, Objective, ProvenanceKind, ProvenanceRef, Source,
    SourceLocator,
};
