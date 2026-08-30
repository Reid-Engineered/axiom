use super::error::KnowledgeError;
use super::identifier::SourceId;
use super::raw::{RawProvenanceRef, RawSourceLocator};
use super::types::{ProvenanceKind, ProvenanceRef, SourceLocator};

pub(crate) fn convert_provenance_refs(
    entity_id: &str,
    raw_refs: Vec<RawProvenanceRef>,
) -> Result<Vec<ProvenanceRef>, KnowledgeError> {
    if raw_refs.is_empty() {
        return Err(KnowledgeError::MissingProvenance {
            entity_id: entity_id.to_owned(),
        });
    }

    let refs: Vec<ProvenanceRef> = raw_refs
        .into_iter()
        .map(|raw| convert_one(entity_id, raw))
        .collect::<Result<_, _>>()?;

    for i in 0..refs.len() {
        for j in (i + 1)..refs.len() {
            if refs[i].source_id == refs[j].source_id
                && refs[i].locator == refs[j].locator
                && refs[i].kind == refs[j].kind
            {
                return Err(KnowledgeError::DuplicateProvenanceRef {
                    entity_id: entity_id.to_owned(),
                    source_id: refs[i].source_id.as_str().to_owned(),
                });
            }
        }
    }

    Ok(refs)
}

fn convert_one(entity_id: &str, raw: RawProvenanceRef) -> Result<ProvenanceRef, KnowledgeError> {
    let source_id = SourceId::new(raw.source_id)?;
    let kind = match raw.kind.as_str() {
        "direct" => ProvenanceKind::Direct,
        "derived" => ProvenanceKind::Derived,
        other => {
            return Err(KnowledgeError::UnknownProvenanceKind {
                entity_id: entity_id.to_owned(),
                value: other.to_owned(),
            })
        }
    };
    let locator = match raw.locator {
        None => None,
        Some(RawSourceLocator {
            section,
            pages,
            label,
        }) => {
            if section.is_none() && pages.is_none() && label.is_none() {
                return Err(KnowledgeError::EmptySourceLocator {
                    entity_id: entity_id.to_owned(),
                });
            }
            Some(SourceLocator {
                section,
                pages,
                label,
            })
        }
    };

    Ok(ProvenanceRef {
        source_id,
        locator,
        kind,
    })
}
