use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

use super::{database_error, CommandResult, Concept, ConceptDiagnostic, Database};

fn edge_ids(connection: &Connection, concept_id: &str, kind: &str) -> CommandResult<Vec<String>> {
    let mut statement = connection
        .prepare(
            "SELECT target_concept_id FROM concept_edges
             WHERE source_concept_id = ?1 AND edge_kind = ?2
             ORDER BY position",
        )
        .map_err(database_error)?;
    let values = statement
        .query_map(params![concept_id, kind], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok(values)
}

pub(crate) fn load_concept(connection: &Connection, id: &str) -> CommandResult<Option<Concept>> {
    let base = connection
        .query_row(
            "SELECT id, workspace_id, name, chapter, mastery_state, was_mastery_state,
                    decayed_at, meaning, due_for_review_in_days, on_exam, display_formula,
                    explanation, learner_heuristic, heuristic_evidence, last_activity_at,
                    notes_count
             FROM concepts WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, bool>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, Option<String>>(14)?,
                    row.get::<_, i64>(15)?,
                ))
            },
        )
        .optional()
        .map_err(database_error)?;
    let Some((
        id,
        workspace_id,
        name,
        chapter,
        mastery_state,
        was_mastery_state,
        decayed_at,
        meaning,
        due_for_review_in_days,
        on_exam,
        display_formula,
        explanation,
        learner_heuristic,
        heuristic_evidence,
        last_activity_at,
        notes_count,
    )) = base
    else {
        return Ok(None);
    };

    let mut statement = connection
        .prepare(
            "SELECT description FROM concept_where_it_shows_up
             WHERE concept_id = ?1 ORDER BY position",
        )
        .map_err(database_error)?;
    let where_it_shows_up = statement
        .query_map([&id], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    let mut statement = connection
        .prepare(
            "SELECT id, expression, diagnostic_type, note, occurred_at
             FROM concept_diagnostics
             WHERE concept_id = ?1
             ORDER BY occurred_at DESC",
        )
        .map_err(database_error)?;
    let recent_diagnostics = statement
        .query_map([&id], |row| {
            Ok(ConceptDiagnostic {
                id: row.get(0)?,
                expression: row.get(1)?,
                diagnostic_type: row.get(2)?,
                note: row.get(3)?,
                occurred_at: row.get(4)?,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    Ok(Some(Concept {
        blocks_concept_ids: edge_ids(connection, &id, "blocks")?,
        prerequisite_concept_ids: edge_ids(connection, &id, "prerequisite")?,
        related_concept_ids: edge_ids(connection, &id, "related")?,
        leads_to_concept_ids: edge_ids(connection, &id, "leadsTo")?,
        id,
        workspace_id,
        name,
        chapter,
        mastery_state,
        was_mastery_state,
        decayed_at,
        meaning,
        due_for_review_in_days,
        on_exam,
        display_formula,
        explanation,
        learner_heuristic,
        heuristic_evidence,
        where_it_shows_up: (!where_it_shows_up.is_empty()).then_some(where_it_shows_up),
        recent_diagnostics: (!recent_diagnostics.is_empty()).then_some(recent_diagnostics),
        last_activity_at,
        notes_count,
    }))
}

pub fn get_concepts_by_workspace_handler(
    database: &Database,
    workspace_id: &str,
) -> CommandResult<Vec<Concept>> {
    let connection = database.connection()?;
    let mut statement = connection
        .prepare("SELECT id FROM concepts WHERE workspace_id = ?1 ORDER BY rowid")
        .map_err(database_error)?;
    let ids = statement
        .query_map([workspace_id], |row| row.get::<_, String>(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    ids.iter()
        .map(|id| load_concept(&connection, id).map(Option::unwrap))
        .collect()
}

pub fn get_concept_handler(database: &Database, id: &str) -> CommandResult<Concept> {
    let connection = database.connection()?;
    load_concept(&connection, id)?.ok_or_else(|| format!("Concept not found: {id}"))
}

pub fn search_concepts_handler(
    database: &Database,
    workspace_id: &str,
    query: &str,
) -> CommandResult<Vec<Concept>> {
    let concepts = get_concepts_by_workspace_handler(database, workspace_id)?;
    let normalized_query = query.trim().to_lowercase();
    if normalized_query.is_empty() {
        return Ok(concepts);
    }
    Ok(concepts
        .into_iter()
        .filter(|concept| {
            [&concept.name, &concept.chapter]
                .into_iter()
                .any(|value| value.to_lowercase().contains(&normalized_query))
                || concept
                    .explanation
                    .as_ref()
                    .is_some_and(|value| value.to_lowercase().contains(&normalized_query))
                || concept.where_it_shows_up.as_ref().is_some_and(|values| {
                    values
                        .iter()
                        .any(|value| value.to_lowercase().contains(&normalized_query))
                })
        })
        .collect())
}

#[tauri::command(rename = "getConceptsByWorkspace", rename_all = "camelCase")]
pub fn get_concepts_by_workspace(
    database: State<'_, Database>,
    workspace_id: String,
) -> CommandResult<Vec<Concept>> {
    get_concepts_by_workspace_handler(&database, &workspace_id)
}

#[tauri::command(rename = "getConcept")]
pub fn get_concept(database: State<'_, Database>, id: String) -> CommandResult<Concept> {
    get_concept_handler(&database, &id)
}

#[tauri::command(rename = "searchConcepts", rename_all = "camelCase")]
pub fn search_concepts(
    database: State<'_, Database>,
    workspace_id: String,
    query: String,
) -> CommandResult<Vec<Concept>> {
    search_concepts_handler(&database, &workspace_id, &query)
}
