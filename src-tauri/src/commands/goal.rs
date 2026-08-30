use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

use super::{database_error, now, CommandResult, Database, Goal, GoalInferredStructure};

pub(crate) fn load_goal(connection: &Connection, id: &str) -> CommandResult<Option<Goal>> {
    let base = connection
        .query_row(
            "SELECT id, workspace_id, text, state, inferred_deadline,
                    inferred_mastery_type, inferred_concept_scope, inferred_pacing,
                    previous_text, achieved_summary, created_at, updated_at
             FROM goals WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                ))
            },
        )
        .optional()
        .map_err(database_error)?;
    let Some((
        id,
        workspace_id,
        text,
        state,
        deadline,
        mastery_type,
        concept_scope,
        pacing,
        previous_text,
        achieved_summary,
        created_at,
        updated_at,
    )) = base
    else {
        return Ok(None);
    };

    let mut statement = connection
        .prepare("SELECT tool FROM goal_tools WHERE goal_id = ?1 ORDER BY position")
        .map_err(database_error)?;
    let tools = statement
        .query_map([&id], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    Ok(Some(Goal {
        id,
        workspace_id,
        text,
        state,
        inferred: GoalInferredStructure {
            deadline,
            mastery_type,
            concept_scope,
            pacing,
            tools: (!tools.is_empty()).then_some(tools),
        },
        previous_text,
        achieved_summary,
        created_at,
        updated_at,
    }))
}

pub fn get_goal_handler(database: &Database, id: &str) -> CommandResult<Goal> {
    let connection = database.connection()?;
    load_goal(&connection, id)?.ok_or_else(|| format!("Goal not found: {id}"))
}

pub fn get_goals_by_workspace_handler(
    database: &Database,
    workspace_id: &str,
) -> CommandResult<Vec<Goal>> {
    let connection = database.connection()?;
    let mut statement = connection
        .prepare("SELECT id FROM goals WHERE workspace_id = ?1 ORDER BY rowid")
        .map_err(database_error)?;
    let ids = statement
        .query_map([workspace_id], |row| row.get::<_, String>(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    ids.iter()
        .map(|id| load_goal(&connection, id).map(Option::unwrap))
        .collect()
}

pub fn update_goal_handler(database: &Database, id: &str, text: &str) -> CommandResult<Goal> {
    let connection = database.connection()?;
    let changed = connection
        .execute(
            "UPDATE goals
             SET previous_text = text, text = ?2, updated_at = ?3
             WHERE id = ?1",
            params![id, text.trim(), now()],
        )
        .map_err(database_error)?;
    if changed == 0 {
        return Err(format!("Goal not found: {id}"));
    }
    load_goal(&connection, id)?.ok_or_else(|| format!("Goal not found: {id}"))
}

pub fn revert_goal_handler(database: &Database, id: &str) -> CommandResult<Goal> {
    let connection = database.connection()?;
    let previous_text = connection
        .query_row(
            "SELECT previous_text FROM goals WHERE id = ?1",
            [id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(database_error)?;
    match previous_text {
        None => return Err(format!("Goal not found: {id}")),
        Some(None) => return Err(format!("Goal has no previous text: {id}")),
        Some(Some(_)) => {}
    }
    connection
        .execute(
            "UPDATE goals
             SET text = previous_text, previous_text = text, updated_at = ?2
             WHERE id = ?1",
            params![id, now()],
        )
        .map_err(database_error)?;
    load_goal(&connection, id)?.ok_or_else(|| format!("Goal not found: {id}"))
}

#[tauri::command(rename = "getGoal")]
pub fn get_goal(database: State<'_, Database>, id: String) -> CommandResult<Goal> {
    get_goal_handler(&database, &id)
}

#[tauri::command(rename = "getGoalsByWorkspace", rename_all = "camelCase")]
pub fn get_goals_by_workspace(
    database: State<'_, Database>,
    workspace_id: String,
) -> CommandResult<Vec<Goal>> {
    get_goals_by_workspace_handler(&database, &workspace_id)
}

#[tauri::command(rename = "updateGoal")]
pub fn update_goal(database: State<'_, Database>, id: String, text: String) -> CommandResult<Goal> {
    update_goal_handler(&database, &id, &text)
}

#[tauri::command(rename = "revertGoal")]
pub fn revert_goal(database: State<'_, Database>, id: String) -> CommandResult<Goal> {
    revert_goal_handler(&database, &id)
}
