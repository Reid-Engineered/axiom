use rusqlite::{Connection, OptionalExtension};
use tauri::State;

use super::{database_error, ChapterSegment, CommandResult, Database, Material, MaterialResult};

pub(crate) fn load_material(
    connection: &Connection,
    workspace_id: &str,
) -> CommandResult<Option<Material>> {
    let material = connection
        .query_row(
            "SELECT id, workspace_id, title, edition, total_pages, total_chapters,
                    highlights_count, notes_count
             FROM materials WHERE workspace_id = ?1",
            [workspace_id],
            |row| {
                Ok(Material {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    title: row.get(2)?,
                    edition: row.get(3)?,
                    total_pages: row.get(4)?,
                    total_chapters: row.get(5)?,
                    highlights_count: row.get(6)?,
                    notes_count: row.get(7)?,
                    segments: Vec::new(),
                    most_marked_sections: Vec::new(),
                })
            },
        )
        .optional()
        .map_err(database_error)?;
    let Some(mut material) = material else {
        return Ok(None);
    };

    let mut statement = connection
        .prepare(
            "SELECT label, status, detail FROM material_chapter_segments
             WHERE material_id = ?1 ORDER BY position",
        )
        .map_err(database_error)?;
    material.segments = statement
        .query_map([&material.id], |row| {
            Ok(ChapterSegment {
                label: row.get(0)?,
                status: row.get(1)?,
                detail: row.get(2)?,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    let mut statement = connection
        .prepare(
            "SELECT section FROM material_most_marked_sections
             WHERE material_id = ?1 ORDER BY position",
        )
        .map_err(database_error)?;
    material.most_marked_sections = statement
        .query_map([&material.id], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok(Some(material))
}

pub fn get_material_handler(database: &Database, workspace_id: &str) -> CommandResult<Material> {
    let connection = database.connection()?;
    load_material(&connection, workspace_id)?
        .ok_or_else(|| format!("Material not found for workspace: {workspace_id}"))
}

pub fn search_material_handler(
    database: &Database,
    workspace_id: &str,
    query: &str,
) -> CommandResult<Vec<MaterialResult>> {
    let connection = database.connection()?;
    let material = load_material(&connection, workspace_id)?
        .ok_or_else(|| format!("Material not found for workspace: {workspace_id}"))?;
    let mut statement = connection
        .prepare(
            "SELECT id, kind, page, title, reason, concept_id, in_syllabus,
                    highlighted_at, exercise_total, exercise_attempted
             FROM material_results
             WHERE material_id = ?1 AND in_syllabus = 1
             ORDER BY rowid",
        )
        .map_err(database_error)?;
    let results = statement
        .query_map([material.id], |row| {
            Ok(MaterialResult {
                id: row.get(0)?,
                kind: row.get(1)?,
                page: row.get(2)?,
                title: row.get(3)?,
                reason: row.get(4)?,
                concept_id: row.get(5)?,
                in_syllabus: row.get(6)?,
                highlighted_at: row.get(7)?,
                exercise_total: row.get(8)?,
                exercise_attempted: row.get(9)?,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    let terms = query
        .trim()
        .to_lowercase()
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    Ok(results
        .into_iter()
        .filter(|result| {
            let searchable = format!("{} {}", result.title, result.reason).to_lowercase();
            terms.iter().all(|term| searchable.contains(term))
        })
        .collect())
}

#[tauri::command(rename = "getMaterial", rename_all = "camelCase")]
pub fn get_material(
    database: State<'_, Database>,
    workspace_id: String,
) -> CommandResult<Material> {
    get_material_handler(&database, &workspace_id)
}

#[tauri::command(rename = "searchMaterial", rename_all = "camelCase")]
pub fn search_material(
    database: State<'_, Database>,
    workspace_id: String,
    query: String,
) -> CommandResult<Vec<MaterialResult>> {
    search_material_handler(&database, &workspace_id, &query)
}
