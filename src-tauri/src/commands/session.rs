use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

use super::{
    database_error, new_id, now, CommandResult, Database, Session, SessionIntent,
    StartSessionInput, TutorExchange,
};

pub(crate) fn load_session(connection: &Connection, id: &str) -> CommandResult<Option<Session>> {
    let session = connection
        .query_row(
            "SELECT id, workspace_id, concept_id, status, intent_activity, intent_detail,
                    intent_target_minutes, resume_summary, thumbnail_url, elapsed_minutes,
                    problem_index, problem_count, open_question, started_at, paused_at
             FROM sessions WHERE id = ?1",
            [id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    concept_id: row.get(2)?,
                    status: row.get(3)?,
                    intent: SessionIntent {
                        activity: row.get(4)?,
                        detail: row.get(5)?,
                        target_minutes: row.get(6)?,
                    },
                    resume_summary: row.get(7)?,
                    thumbnail_url: row.get(8)?,
                    elapsed_minutes: row.get(9)?,
                    problem_index: row.get(10)?,
                    problem_count: row.get(11)?,
                    open_question: row.get(12)?,
                    started_at: row.get(13)?,
                    paused_at: row.get(14)?,
                    exchanges: Vec::new(),
                    settled_conclusions: Vec::new(),
                })
            },
        )
        .optional()
        .map_err(database_error)?;
    let Some(mut session) = session else {
        return Ok(None);
    };

    let mut statement = connection
        .prepare(
            "SELECT id, question, answer, occurred_at, pinned_to_visualization
             FROM tutor_exchanges WHERE session_id = ?1 ORDER BY position",
        )
        .map_err(database_error)?;
    session.exchanges = statement
        .query_map([&session.id], |row| {
            Ok(TutorExchange {
                id: row.get(0)?,
                question: row.get(1)?,
                answer: row.get(2)?,
                occurred_at: row.get(3)?,
                pinned_to_visualization: row.get(4)?,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    let mut statement = connection
        .prepare(
            "SELECT conclusion FROM session_settled_conclusions
             WHERE session_id = ?1 ORDER BY position",
        )
        .map_err(database_error)?;
    session.settled_conclusions = statement
        .query_map([&session.id], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok(Some(session))
}

pub fn get_active_session_by_workspace_handler(
    database: &Database,
    workspace_id: &str,
) -> CommandResult<Option<Session>> {
    let connection = database.connection()?;
    let id = connection
        .query_row(
            "SELECT id FROM sessions
             WHERE workspace_id = ?1 AND status <> 'completed'
             ORDER BY rowid LIMIT 1",
            [workspace_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(database_error)?;
    id.map(|id| load_session(&connection, &id).map(Option::unwrap))
        .transpose()
}

pub fn get_session_handler(database: &Database, id: &str) -> CommandResult<Session> {
    let connection = database.connection()?;
    load_session(&connection, id)?.ok_or_else(|| format!("Session not found: {id}"))
}

pub fn start_session_handler(
    database: &Database,
    input: StartSessionInput,
) -> CommandResult<Session> {
    let connection = database.connection()?;
    let workspace_exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM workspaces WHERE id = ?1)",
            [&input.workspace_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(database_error)?;
    if !workspace_exists {
        return Err(format!("Workspace not found: {}", input.workspace_id));
    }
    let concept_name = connection
        .query_row(
            "SELECT name FROM concepts WHERE id = ?1 AND workspace_id = ?2",
            params![input.concept_id, input.workspace_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| format!("Concept not found in workspace: {}", input.concept_id))?;
    let id = new_id("session");
    connection
        .execute(
            "INSERT INTO sessions (
                id, workspace_id, concept_id, status, intent_activity, intent_detail,
                intent_target_minutes, resume_summary, elapsed_minutes, started_at
            ) VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, ?7, 0, ?8)",
            params![
                id,
                input.workspace_id,
                input.concept_id,
                input.intent.activity,
                input.intent.detail,
                input.intent.target_minutes,
                format!("Ready to continue with {concept_name}."),
                now(),
            ],
        )
        .map_err(database_error)?;
    load_session(&connection, &id)?.ok_or_else(|| format!("Session not found: {id}"))
}

fn ensure_mutable_session(connection: &Connection, id: &str) -> CommandResult<()> {
    let status = connection
        .query_row("SELECT status FROM sessions WHERE id = ?1", [id], |row| {
            row.get::<_, String>(0)
        })
        .optional()
        .map_err(database_error)?;
    match status.as_deref() {
        None => Err(format!("Session not found: {id}")),
        Some("completed") => Err(format!("Session is completed: {id}")),
        Some(_) => Ok(()),
    }
}

pub fn pause_session_handler(database: &Database, id: &str) -> CommandResult<Session> {
    let connection = database.connection()?;
    ensure_mutable_session(&connection, id)?;
    connection
        .execute(
            "UPDATE sessions SET status = 'paused', paused_at = ?2 WHERE id = ?1",
            params![id, now()],
        )
        .map_err(database_error)?;
    load_session(&connection, id)?.ok_or_else(|| format!("Session not found: {id}"))
}

pub fn resume_session_handler(database: &Database, id: &str) -> CommandResult<Session> {
    let connection = database.connection()?;
    ensure_mutable_session(&connection, id)?;
    connection
        .execute(
            "UPDATE sessions SET status = 'active', paused_at = NULL WHERE id = ?1",
            [id],
        )
        .map_err(database_error)?;
    load_session(&connection, id)?.ok_or_else(|| format!("Session not found: {id}"))
}

pub fn add_tutor_exchange_handler(
    database: &Database,
    session_id: &str,
    question: &str,
) -> CommandResult<Session> {
    let mut connection = database.connection()?;
    ensure_mutable_session(&connection, session_id)?;
    let transaction = connection.transaction().map_err(database_error)?;
    let position = transaction
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1
             FROM tutor_exchanges WHERE session_id = ?1",
            [session_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(database_error)?;
    transaction
        .execute(
            "INSERT INTO tutor_exchanges (
                id, session_id, position, question, answer, occurred_at,
                pinned_to_visualization
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
            params![
                new_id("exchange"),
                session_id,
                position,
                question.trim(),
                "Start with what the current representation makes visible, then test one step against the goal.",
                now(),
            ],
        )
        .map_err(database_error)?;
    transaction.commit().map_err(database_error)?;
    load_session(&connection, session_id)?.ok_or_else(|| format!("Session not found: {session_id}"))
}

pub fn end_session_handler(database: &Database, id: &str) -> CommandResult<Session> {
    let connection = database.connection()?;
    if load_session(&connection, id)?.is_none() {
        return Err(format!("Session not found: {id}"));
    }
    connection
        .execute(
            "UPDATE sessions SET status = 'completed', paused_at = NULL WHERE id = ?1",
            [id],
        )
        .map_err(database_error)?;
    load_session(&connection, id)?.ok_or_else(|| format!("Session not found: {id}"))
}

#[tauri::command(rename = "getActiveSessionByWorkspace", rename_all = "camelCase")]
pub fn get_active_session_by_workspace(
    database: State<'_, Database>,
    workspace_id: String,
) -> CommandResult<Option<Session>> {
    get_active_session_by_workspace_handler(&database, &workspace_id)
}

#[tauri::command(rename = "getSession")]
pub fn get_session(database: State<'_, Database>, id: String) -> CommandResult<Session> {
    get_session_handler(&database, &id)
}

#[tauri::command(rename = "startSession")]
pub fn start_session(
    database: State<'_, Database>,
    input: StartSessionInput,
) -> CommandResult<Session> {
    start_session_handler(&database, input)
}

#[tauri::command(rename = "pauseSession")]
pub fn pause_session(database: State<'_, Database>, id: String) -> CommandResult<Session> {
    pause_session_handler(&database, &id)
}

#[tauri::command(rename = "resumeSession")]
pub fn resume_session(database: State<'_, Database>, id: String) -> CommandResult<Session> {
    resume_session_handler(&database, &id)
}

#[tauri::command(rename = "addTutorExchange", rename_all = "camelCase")]
pub fn add_tutor_exchange(
    database: State<'_, Database>,
    session_id: String,
    question: String,
) -> CommandResult<Session> {
    add_tutor_exchange_handler(&database, &session_id, &question)
}

#[tauri::command(rename = "endSession")]
pub fn end_session(database: State<'_, Database>, id: String) -> CommandResult<Session> {
    end_session_handler(&database, &id)
}
