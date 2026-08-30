use tauri::State;

use super::{database_error, CommandResult, Database, Note};

pub fn get_recent_notes_handler(
    database: &Database,
    workspace_id: &str,
) -> CommandResult<Vec<Note>> {
    let connection = database.connection()?;
    let mut statement = connection
        .prepare(
            "SELECT id, workspace_id, concept_id, text, updated_at
             FROM notes WHERE workspace_id = ?1
             ORDER BY updated_at DESC",
        )
        .map_err(database_error)?;
    let notes = statement
        .query_map([workspace_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                concept_id: row.get(2)?,
                text: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok(notes)
}

#[tauri::command(rename = "getRecentNotes", rename_all = "camelCase")]
pub fn get_recent_notes(
    database: State<'_, Database>,
    workspace_id: String,
) -> CommandResult<Vec<Note>> {
    get_recent_notes_handler(&database, &workspace_id)
}
