use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

use super::{
    database_error, new_id, now, CommandResult, CreateWorkspaceInput, Database,
    OfflineKindAvailability, OfflinePartialAvailability, Workspace, WorkspaceActivityEvent,
};

const OFFLINE_KINDS: [&str; 4] = [
    "textbookAndLectureNotes",
    "problemBanks",
    "visualAssetsAndModuleData",
    "courseVideos",
];

pub(crate) fn load_workspace(
    connection: &Connection,
    id: &str,
) -> CommandResult<Option<Workspace>> {
    let base = connection
        .query_row(
            "SELECT id, name, guiding_goal_id, progress, last_concept_name,
                    last_activity_at, paused
             FROM workspaces WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, bool>(6)?,
                ))
            },
        )
        .optional()
        .map_err(database_error)?;

    let Some((id, name, guiding_goal_id, progress, last_concept_name, last_activity_at, paused)) =
        base
    else {
        return Ok(None);
    };

    let mut statement = connection
        .prepare(
            "SELECT kind, enabled, size_bytes, partial_available_count,
                    partial_total_count, partial_limit_reason
             FROM workspace_offline_availability
             WHERE workspace_id = ?1
             ORDER BY CASE kind
                 WHEN 'textbookAndLectureNotes' THEN 0
                 WHEN 'problemBanks' THEN 1
                 WHEN 'visualAssetsAndModuleData' THEN 2
                 ELSE 3
             END",
        )
        .map_err(database_error)?;
    let offline_availability = statement
        .query_map([&id], |row| {
            let available_count = row.get::<_, Option<i64>>(3)?;
            let total_count = row.get::<_, Option<i64>>(4)?;
            let limit_reason = row.get::<_, Option<String>>(5)?;
            let partial = match (available_count, total_count, limit_reason) {
                (Some(available_count), Some(total_count), Some(limit_reason)) => {
                    Some(OfflinePartialAvailability {
                        available_count,
                        total_count,
                        limit_reason,
                    })
                }
                _ => None,
            };
            Ok(OfflineKindAvailability {
                kind: row.get(0)?,
                enabled: row.get(1)?,
                size_bytes: row.get(2)?,
                partial,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    let mut statement = connection
        .prepare(
            "SELECT module_id FROM workspace_modules
             WHERE workspace_id = ?1 AND enabled = 1
             ORDER BY module_id",
        )
        .map_err(database_error)?;
    let enabled_module_ids = statement
        .query_map([&id], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    Ok(Some(Workspace {
        id,
        name,
        guiding_goal_id,
        progress,
        last_concept_name,
        last_activity_at,
        paused,
        offline_availability,
        enabled_module_ids,
    }))
}

pub fn get_workspaces_handler(database: &Database) -> CommandResult<Vec<Workspace>> {
    let connection = database.connection()?;
    let mut statement = connection
        .prepare("SELECT id FROM workspaces ORDER BY rowid")
        .map_err(database_error)?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    ids.iter()
        .map(|id| load_workspace(&connection, id).map(Option::unwrap))
        .collect()
}

pub fn get_workspace_handler(database: &Database, id: &str) -> CommandResult<Workspace> {
    let connection = database.connection()?;
    load_workspace(&connection, id)?.ok_or_else(|| format!("Workspace not found: {id}"))
}

pub fn get_recent_activity_handler(
    database: &Database,
    workspace_id: &str,
) -> CommandResult<Vec<WorkspaceActivityEvent>> {
    let connection = database.connection()?;
    let mut statement = connection
        .prepare(
            "SELECT id, workspace_id, occurred_at, summary
             FROM workspace_activity_events
             WHERE workspace_id = ?1
             ORDER BY occurred_at ASC
             LIMIT 3",
        )
        .map_err(database_error)?;
    let events = statement
        .query_map([workspace_id], |row| {
            Ok(WorkspaceActivityEvent {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                occurred_at: row.get(2)?,
                summary: row.get(3)?,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok(events)
}

pub fn create_workspace_handler(
    database: &Database,
    input: CreateWorkspaceInput,
) -> CommandResult<Workspace> {
    let workspace_id = new_id("workspace");
    let goal_id = new_id("goal");
    let created_at = now();
    let subject = input.subject.trim();
    let goal_text = input.goal_text.trim();
    let mut connection = database.connection()?;
    let transaction = connection.transaction().map_err(database_error)?;

    transaction
        .execute(
            "INSERT INTO workspaces (
                id, name, guiding_goal_id, progress, paused
            ) VALUES (?1, ?2, ?3, 0, 0)",
            params![workspace_id, subject, goal_id],
        )
        .map_err(database_error)?;
    transaction
        .execute(
            "INSERT INTO goals (
                id, workspace_id, text, state, created_at, updated_at
            ) VALUES (?1, ?2, ?3, 'Guiding', ?4, ?4)",
            params![goal_id, workspace_id, goal_text, created_at],
        )
        .map_err(database_error)?;
    for kind in OFFLINE_KINDS {
        transaction
            .execute(
                "INSERT INTO workspace_offline_availability (
                    workspace_id, kind, enabled, size_bytes
                ) VALUES (?1, ?2, 0, 0)",
                params![workspace_id, kind],
            )
            .map_err(database_error)?;
    }
    transaction.commit().map_err(database_error)?;

    load_workspace(&connection, &workspace_id)?
        .ok_or_else(|| format!("Workspace not found: {workspace_id}"))
}

pub fn set_workspace_offline_availability_handler(
    database: &Database,
    id: &str,
    kind: &str,
    enabled: bool,
) -> CommandResult<Workspace> {
    let connection = database.connection()?;
    if load_workspace(&connection, id)?.is_none() {
        return Err(format!("Workspace not found: {id}"));
    }
    let changed = connection
        .execute(
            "UPDATE workspace_offline_availability SET enabled = ?3
             WHERE workspace_id = ?1 AND kind = ?2",
            params![id, kind, enabled],
        )
        .map_err(database_error)?;
    if changed == 0 {
        return Err(format!("Offline content kind not found: {kind}"));
    }
    load_workspace(&connection, id)?.ok_or_else(|| format!("Workspace not found: {id}"))
}

#[tauri::command(rename = "getWorkspaces")]
pub fn get_workspaces(database: State<'_, Database>) -> CommandResult<Vec<Workspace>> {
    get_workspaces_handler(&database)
}

#[tauri::command(rename = "getWorkspace")]
pub fn get_workspace(database: State<'_, Database>, id: String) -> CommandResult<Workspace> {
    get_workspace_handler(&database, &id)
}

#[tauri::command(rename = "getRecentActivity", rename_all = "camelCase")]
pub fn get_recent_activity(
    database: State<'_, Database>,
    workspace_id: String,
) -> CommandResult<Vec<WorkspaceActivityEvent>> {
    get_recent_activity_handler(&database, &workspace_id)
}

#[tauri::command(rename = "createWorkspace")]
pub fn create_workspace(
    database: State<'_, Database>,
    input: CreateWorkspaceInput,
) -> CommandResult<Workspace> {
    create_workspace_handler(&database, input)
}

#[tauri::command(rename = "setWorkspaceOfflineAvailability", rename_all = "camelCase")]
pub fn set_workspace_offline_availability(
    database: State<'_, Database>,
    id: String,
    kind: String,
    enabled: bool,
) -> CommandResult<Workspace> {
    set_workspace_offline_availability_handler(&database, &id, &kind, enabled)
}
