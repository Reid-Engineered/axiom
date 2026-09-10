use std::sync::Arc;

use rusqlite::{params, Connection, OptionalExtension};
use tauri::{async_runtime::RwLock, State};

use crate::modules::{
    CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement, ModuleId,
    ModuleInstallation, ModuleRegistry,
};
use crate::practice::{
    AttemptStatus, DescribeRequest, DescribeResponse, StartRequest, StartResponse,
};

use super::{
    database_error, new_id, now, CommandResult, Database, Session, SessionIntent,
    StartSessionInput, TutorExchange,
};

pub(crate) fn load_session(connection: &Connection, id: &str) -> CommandResult<Option<Session>> {
    let session = connection
        .query_row(
            "SELECT id, workspace_id, concept_id, status, intent_activity, intent_detail,
                    intent_target_minutes, resume_summary, thumbnail_url, elapsed_minutes,
                    problem_index, problem_count, open_question, started_at, paused_at,
                    current_attempt_id, last_activity_at
             FROM sessions WHERE id = ?1",
            [id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    concept_id: row.get(2)?,
                    current_attempt_id: row.get(15)?,
                    last_activity_at: row.get(16)?,
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
             ORDER BY last_activity_at IS NULL, last_activity_at DESC, rowid DESC LIMIT 1",
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

pub async fn start_session_handler(
    database: &Database,
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: StartSessionInput,
) -> CommandResult<Session> {
    let (concept_name, knowledge_concept_id, existing) = {
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
        let (concept_name, knowledge_concept_id) = connection
            .query_row(
                "SELECT name, knowledge_concept_id FROM concepts
                 WHERE id = ?1 AND workspace_id = ?2",
                params![input.concept_id, input.workspace_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()
            .map_err(database_error)?
            .ok_or_else(|| format!("Concept not found in workspace: {}", input.concept_id))?;
        let existing_id = connection
            .query_row(
                "SELECT id FROM sessions
                 WHERE workspace_id = ?1 AND concept_id = ?2 AND status <> 'completed'
                 ORDER BY last_activity_at IS NULL, last_activity_at DESC, rowid DESC LIMIT 1",
                params![input.workspace_id, input.concept_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(database_error)?;
        let existing = existing_id
            .map(|id| load_session(&connection, &id).map(Option::unwrap))
            .transpose()?;
        (concept_name, knowledge_concept_id, existing)
    };

    if let Some(existing) = existing {
        let needs_attempt = match existing.current_attempt_id.as_deref() {
            None => true,
            Some(attempt_id) => {
                attempt_is_solved(registry, installation, &input.workspace_id, attempt_id).await
            }
        };
        let new_attempt_id = if needs_attempt {
            match knowledge_concept_id {
                Some(concept_id) => {
                    start_practice_attempt(registry, installation, &input.workspace_id, concept_id)
                        .await
                }
                None => None,
            }
        } else {
            existing.current_attempt_id
        };
        let activity_at = now();
        let mut connection = database.connection()?;
        let transaction = connection.transaction().map_err(database_error)?;
        transaction
            .execute(
                "UPDATE sessions SET status = 'active', paused_at = NULL,
                    current_attempt_id = ?2,
                    problem_index = CASE
                        WHEN ?3 AND ?2 IS NOT NULL THEN COALESCE(problem_index, 0) + 1
                        ELSE problem_index
                    END,
                    last_activity_at = ?4
                 WHERE id = ?1",
                params![existing.id, new_attempt_id, needs_attempt, activity_at],
            )
            .map_err(database_error)?;
        touch_workspace(&transaction, &input.workspace_id, &activity_at)?;
        transaction.commit().map_err(database_error)?;
        return load_session(&connection, &existing.id)?
            .ok_or_else(|| format!("Session not found: {}", existing.id));
    }
    let current_attempt_id = match knowledge_concept_id {
        Some(concept_id) => {
            start_practice_attempt(registry, installation, &input.workspace_id, concept_id).await
        }
        None => None,
    };

    // A bound attempt is problem 1. Left NULL when nothing was bound, so the counter never
    // claims a problem that does not exist. `mockBackend.ts` already does this; the real
    // backend did not, and only the UI's `?? 1` and `nextProblem`'s COALESCE hid the gap.
    let problem_index = current_attempt_id.as_ref().map(|_| 1);
    let id = new_id("session");
    let activity_at = now();
    let mut connection = database.connection()?;
    let transaction = connection.transaction().map_err(database_error)?;
    transaction
        .execute(
            "INSERT INTO sessions (
                id, workspace_id, concept_id, status, intent_activity, intent_detail,
                intent_target_minutes, resume_summary, elapsed_minutes, started_at,
                current_attempt_id, problem_index, last_activity_at
            ) VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, ?7, 0, ?8, ?9, ?10, ?11)",
            params![
                id,
                input.workspace_id,
                input.concept_id,
                input.intent.activity,
                input.intent.detail,
                input.intent.target_minutes,
                format!("Ready to continue with {concept_name}."),
                now(),
                current_attempt_id,
                problem_index,
                activity_at,
            ],
        )
        .map_err(database_error)?;
    touch_workspace(&transaction, &input.workspace_id, &activity_at)?;
    transaction.commit().map_err(database_error)?;
    load_session(&connection, &id)?.ok_or_else(|| format!("Session not found: {id}"))
}

async fn attempt_is_solved(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    workspace_id: &str,
    attempt_id: &str,
) -> bool {
    let requirement = CapabilityRequirement {
        id: CapabilityId::new("practice.describe").expect("static capability id is valid"),
        min_version: 1,
    };
    let handle = {
        let registry = registry.read().await;
        match registry.resolve(installation, &requirement) {
            Ok(handle) => handle,
            Err(_) => return false,
        }
    };
    let call = CapabilityCall {
        envelope: CallEnvelope {
            workspace_id: workspace_id.to_owned(),
            capability_id: requirement.id.clone(),
            version: 1,
            calling_module_id: ModuleId::new("core.session").expect("static module id is valid"),
        },
        input: DescribeRequest {
            workspace_id: workspace_id.to_owned(),
            attempt_id: attempt_id.to_owned(),
        },
    };
    let response: Result<DescribeResponse, _> = {
        let registry = registry.read().await;
        registry.invoke(&handle, installation, call).await
    };
    matches!(response, Ok(response) if response.status == AttemptStatus::Solved)
}

fn touch_workspace(
    connection: &Connection,
    workspace_id: &str,
    activity_at: &str,
) -> CommandResult<()> {
    connection
        .execute(
            "UPDATE workspaces SET last_activity_at = ?2 WHERE id = ?1",
            params![workspace_id, activity_at],
        )
        .map_err(database_error)?;
    Ok(())
}

pub(crate) fn touch_session_for_attempt(
    database: &Database,
    attempt_id: &str,
) -> CommandResult<()> {
    let activity_at = now();
    let mut connection = database.connection()?;
    let transaction = connection.transaction().map_err(database_error)?;
    let workspace_id = transaction
        .query_row(
            "SELECT workspace_id FROM sessions WHERE current_attempt_id = ?1",
            [attempt_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(database_error)?;
    if let Some(workspace_id) = workspace_id {
        transaction
            .execute(
                "UPDATE sessions SET last_activity_at = ?2 WHERE current_attempt_id = ?1",
                params![attempt_id, activity_at],
            )
            .map_err(database_error)?;
        touch_workspace(&transaction, &workspace_id, &activity_at)?;
    }
    transaction.commit().map_err(database_error)?;
    Ok(())
}

async fn start_practice_attempt(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    workspace_id: &str,
    concept_id: String,
) -> Option<String> {
    let requirement = CapabilityRequirement {
        id: CapabilityId::new("practice.start").expect("static capability id is valid"),
        min_version: 1,
    };
    let handle = {
        let registry = registry.read().await;
        registry.resolve(installation, &requirement).ok()?
    };
    let call = CapabilityCall {
        envelope: CallEnvelope {
            workspace_id: workspace_id.to_owned(),
            capability_id: requirement.id.clone(),
            version: 1,
            calling_module_id: ModuleId::new("core.session").expect("static module id is valid"),
        },
        input: StartRequest {
            workspace_id: workspace_id.to_owned(),
            concept_id,
        },
    };
    let result: Result<StartResponse, _> = {
        let registry = registry.read().await;
        registry.invoke(&handle, installation, call).await
    };
    match result {
        Ok(response) => response.attempt_id,
        Err(error) => {
            eprintln!("practice.start failed during session creation: {error}");
            None
        }
    }
}

pub async fn next_problem_handler(
    database: &Database,
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    session_id: &str,
) -> CommandResult<Session> {
    let (workspace_id, knowledge_concept_id) = {
        let connection = database.connection()?;
        ensure_mutable_session(&connection, session_id)?;
        connection
            .query_row(
                "SELECT sessions.workspace_id, concepts.knowledge_concept_id
                 FROM sessions
                 JOIN concepts ON concepts.id = sessions.concept_id
                 WHERE sessions.id = ?1",
                [session_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()
            .map_err(database_error)?
            .ok_or_else(|| format!("Session not found: {session_id}"))?
    };
    let attempt_id = match knowledge_concept_id {
        Some(concept_id) => {
            start_practice_attempt(registry, installation, &workspace_id, concept_id).await
        }
        None => None,
    };

    let activity_at = now();
    let mut connection = database.connection()?;
    let transaction = connection.transaction().map_err(database_error)?;
    if attempt_id.is_some() {
        transaction
            .execute(
                "UPDATE sessions
                 SET current_attempt_id = ?2, problem_index = COALESCE(problem_index, 1) + 1,
                     last_activity_at = ?3
                 WHERE id = ?1",
                params![session_id, attempt_id, activity_at],
            )
            .map_err(database_error)?;
    } else {
        transaction
            .execute(
                "UPDATE sessions SET last_activity_at = ?2 WHERE id = ?1",
                params![session_id, activity_at],
            )
            .map_err(database_error)?;
    }
    touch_workspace(&transaction, &workspace_id, &activity_at)?;
    transaction.commit().map_err(database_error)?;
    load_session(&connection, session_id)?.ok_or_else(|| format!("Session not found: {session_id}"))
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
    let mut connection = database.connection()?;
    ensure_mutable_session(&connection, id)?;
    let activity_at = now();
    let transaction = connection.transaction().map_err(database_error)?;
    let workspace_id: String = transaction
        .query_row(
            "SELECT workspace_id FROM sessions WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    transaction
        .execute(
            "UPDATE sessions SET status = 'paused', paused_at = ?2, last_activity_at = ?2 WHERE id = ?1",
            params![id, activity_at],
        )
        .map_err(database_error)?;
    touch_workspace(&transaction, &workspace_id, &activity_at)?;
    transaction.commit().map_err(database_error)?;
    load_session(&connection, id)?.ok_or_else(|| format!("Session not found: {id}"))
}

pub fn resume_session_handler(database: &Database, id: &str) -> CommandResult<Session> {
    let mut connection = database.connection()?;
    ensure_mutable_session(&connection, id)?;
    let activity_at = now();
    let transaction = connection.transaction().map_err(database_error)?;
    let workspace_id: String = transaction
        .query_row(
            "SELECT workspace_id FROM sessions WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    transaction
        .execute(
            "UPDATE sessions SET status = 'active', paused_at = NULL, last_activity_at = ?2 WHERE id = ?1",
            params![id, activity_at],
        )
        .map_err(database_error)?;
    touch_workspace(&transaction, &workspace_id, &activity_at)?;
    transaction.commit().map_err(database_error)?;
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
pub async fn start_session(
    database: State<'_, Database>,
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: StartSessionInput,
) -> CommandResult<Session> {
    start_session_handler(&database, &registry, &installation, input).await
}

#[tauri::command(rename = "nextProblem")]
pub async fn next_problem(
    database: State<'_, Database>,
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    session_id: String,
) -> CommandResult<Session> {
    next_problem_handler(&database, &registry, &installation, &session_id).await
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
