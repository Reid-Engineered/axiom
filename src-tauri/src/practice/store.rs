use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, params_from_iter, Connection, OptionalExtension};

use crate::knowledge::ProblemInstance;

use super::error::PracticeError;
use super::types::AttemptStatus;

pub struct PracticeStore(Mutex<Connection>);

#[derive(Debug, Clone, PartialEq)]
pub struct AttemptRow {
    pub id: String,
    pub workspace_id: String,
    pub family_id: String,
    pub seed: u64,
    pub instance: ProblemInstance,
    pub hints_revealed: u32,
    pub status: AttemptStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl PracticeStore {
    pub fn new(connection: Connection) -> Self {
        Self(Mutex::new(connection))
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>, PracticeError> {
        self.0
            .lock()
            .map_err(|_| PracticeError::Storage("connection lock is poisoned".to_owned()))
    }

    pub fn insert_attempt(
        &self,
        id: &str,
        workspace_id: &str,
        family_id: &str,
        seed: u64,
        instance: &ProblemInstance,
    ) -> Result<(), PracticeError> {
        let instance_json = serde_json::to_string(instance)
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        let now = now();
        self.connection()?
            .execute(
                "INSERT INTO practice_attempts
                    (id, workspace_id, family_id, seed, instance_json, hints_revealed,
                     status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, 'open', ?6, ?6)",
                params![id, workspace_id, family_id, seed as i64, instance_json, now],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        Ok(())
    }

    pub fn load_attempt(
        &self,
        attempt_id: &str,
        workspace_id: &str,
    ) -> Result<AttemptRow, PracticeError> {
        self.connection()?
            .query_row(
                "SELECT id, workspace_id, family_id, seed, instance_json, hints_revealed,
                        status, created_at, updated_at
                 FROM practice_attempts WHERE id = ?1 AND workspace_id = ?2",
                params![attempt_id, workspace_id],
                map_attempt_row,
            )
            .optional()
            .map_err(|error| PracticeError::Storage(error.to_string()))?
            .ok_or_else(|| PracticeError::AttemptNotFound {
                attempt_id: attempt_id.to_owned(),
            })
    }

    pub fn most_recent_candidate_family(
        &self,
        workspace_id: &str,
        family_ids: &[String],
    ) -> Result<Option<String>, PracticeError> {
        if family_ids.is_empty() {
            return Ok(None);
        }

        let placeholders = (2..family_ids.len() + 2)
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT family_id FROM practice_attempts
             WHERE workspace_id = ?1 AND family_id IN ({placeholders})
             ORDER BY created_at DESC, rowid DESC LIMIT 1"
        );
        self.connection()?
            .query_row(
                &sql,
                params_from_iter(
                    std::iter::once(workspace_id).chain(family_ids.iter().map(String::as_str)),
                ),
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| PracticeError::Storage(error.to_string()))
    }

    pub fn increment_hints_revealed(&self, attempt_id: &str) -> Result<u32, PracticeError> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE practice_attempts SET hints_revealed = hints_revealed + 1, updated_at = ?2
                 WHERE id = ?1",
                params![attempt_id, now()],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        connection
            .query_row(
                "SELECT hints_revealed FROM practice_attempts WHERE id = ?1",
                params![attempt_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value as u32)
            .map_err(|error| PracticeError::Storage(error.to_string()))
    }

    pub fn record_submission(
        &self,
        attempt_id: &str,
        response_json: &str,
        correct: bool,
    ) -> Result<(), PracticeError> {
        self.connection()?
            .execute(
                "INSERT INTO practice_submissions (id, attempt_id, response_json, correct, submitted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![new_id(), attempt_id, response_json, correct as i64, now()],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        Ok(())
    }

    pub fn mark_solved(&self, attempt_id: &str) -> Result<(), PracticeError> {
        self.connection()?
            .execute(
                "UPDATE practice_attempts SET status = 'solved', updated_at = ?2 WHERE id = ?1",
                params![attempt_id, now()],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        Ok(())
    }

    pub fn count_submissions(&self, attempt_id: &str) -> Result<u32, PracticeError> {
        self.connection()?
            .query_row(
                "SELECT COUNT(*) FROM practice_submissions WHERE attempt_id = ?1",
                params![attempt_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value as u32)
            .map_err(|error| PracticeError::Storage(error.to_string()))
    }
}

fn map_attempt_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttemptRow> {
    let instance_json: String = row.get(4)?;
    let instance: ProblemInstance = serde_json::from_str(&instance_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let status_text: String = row.get(6)?;
    let status = match status_text.as_str() {
        "solved" => AttemptStatus::Solved,
        _ => AttemptStatus::Open,
    };
    Ok(AttemptRow {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        family_id: row.get(2)?,
        seed: row.get::<_, i64>(3)? as u64,
        instance,
        hints_revealed: row.get::<_, i64>(5)? as u32,
        status,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn new_id() -> String {
    format!("submission-{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
impl PracticeStore {
    pub(crate) fn connection_for_test(&self) -> MutexGuard<'_, Connection> {
        self.connection().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_instance() -> ProblemInstance {
        use std::collections::BTreeMap;
        ProblemInstance {
            family_id: crate::knowledge::ProblemFamilyId::new("problem.shell_y_poly").unwrap(),
            seed: 7,
            resolved_parameters: BTreeMap::from([("coeff".to_owned(), 4.0)]),
            prompt: "Find the volume.".to_owned(),
            canonical_solution: crate::knowledge::ResolvedSolution::Numeric(12.0),
            hints: vec!["Identify the radius.".to_owned()],
        }
    }

    fn store() -> PracticeStore {
        PracticeStore::new(crate::db::open_in_memory().unwrap())
    }

    fn seed_workspace(store: &PracticeStore) {
        let mut connection = store.connection().unwrap();
        let transaction = connection.transaction().unwrap();
        transaction
            .execute(
                "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
                 VALUES ('ws-1', 'Test', 'goal-1', 0.0, 0)",
                [],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO goals (id, workspace_id, text, state, created_at, updated_at)
                 VALUES ('goal-1', 'ws-1', 'Test goal', 'Guiding', ?1, ?1)",
                ["2026-09-04T12:00:00Z"],
            )
            .unwrap();
        transaction.commit().unwrap();
    }

    #[test]
    fn insert_then_load_round_trips_the_full_instance() {
        let store = store();
        seed_workspace(&store);
        let instance = sample_instance();
        store
            .insert_attempt("attempt-1", "ws-1", "problem.shell_y_poly", 7, &instance)
            .unwrap();

        let row = store.load_attempt("attempt-1", "ws-1").unwrap();

        assert_eq!(row.instance, instance);
        assert_eq!(row.hints_revealed, 0);
        assert_eq!(row.status, AttemptStatus::Open);
    }

    #[test]
    fn load_attempt_from_a_different_workspace_is_not_found() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt(
                "attempt-1",
                "ws-1",
                "problem.shell_y_poly",
                7,
                &sample_instance(),
            )
            .unwrap();

        let result = store.load_attempt("attempt-1", "ws-other");

        assert!(matches!(result, Err(PracticeError::AttemptNotFound { .. })));
    }

    #[test]
    fn increment_hints_revealed_persists_across_loads() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt(
                "attempt-1",
                "ws-1",
                "problem.shell_y_poly",
                7,
                &sample_instance(),
            )
            .unwrap();

        assert_eq!(store.increment_hints_revealed("attempt-1").unwrap(), 1);
        assert_eq!(store.increment_hints_revealed("attempt-1").unwrap(), 2);
        assert_eq!(
            store
                .load_attempt("attempt-1", "ws-1")
                .unwrap()
                .hints_revealed,
            2
        );
    }

    #[test]
    fn record_submission_then_count_reflects_it() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt(
                "attempt-1",
                "ws-1",
                "problem.shell_y_poly",
                7,
                &sample_instance(),
            )
            .unwrap();

        store
            .record_submission("attempt-1", "{\"value\":1.0}", false)
            .unwrap();
        store
            .record_submission("attempt-1", "{\"value\":4.0}", true)
            .unwrap();

        assert_eq!(store.count_submissions("attempt-1").unwrap(), 2);
    }

    #[test]
    fn mark_solved_updates_status() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt(
                "attempt-1",
                "ws-1",
                "problem.shell_y_poly",
                7,
                &sample_instance(),
            )
            .unwrap();

        store.mark_solved("attempt-1").unwrap();

        assert_eq!(
            store.load_attempt("attempt-1", "ws-1").unwrap().status,
            AttemptStatus::Solved
        );
    }

    #[test]
    fn most_recent_candidate_family_filters_by_workspace_and_candidate_list() {
        let store = store();
        seed_workspace(&store);
        for (attempt_id, family_id) in [
            ("attempt-1", "problem.family_a"),
            ("attempt-2", "problem.family_b"),
            ("attempt-3", "problem.outside_candidates"),
        ] {
            store
                .insert_attempt(attempt_id, "ws-1", family_id, 7, &sample_instance())
                .unwrap();
        }

        let candidates = vec!["problem.family_a".to_owned(), "problem.family_b".to_owned()];
        assert_eq!(
            store
                .most_recent_candidate_family("ws-1", &candidates)
                .unwrap(),
            Some("problem.family_b".to_owned())
        );
        assert_eq!(
            store
                .most_recent_candidate_family("ws-other", &candidates)
                .unwrap(),
            None
        );
        assert_eq!(
            store.most_recent_candidate_family("ws-1", &[]).unwrap(),
            None
        );
    }

    #[test]
    fn attempt_surviving_a_fresh_connection_to_the_same_file_reads_back_identically() {
        let dir =
            std::env::temp_dir().join(format!("axiom-practice-test-{}", uuid::Uuid::new_v4()));
        let db_path = dir.join("axiom.sqlite3");
        std::fs::create_dir_all(&dir).unwrap();

        {
            let store = PracticeStore::new(crate::db::open(&db_path).unwrap());
            seed_workspace(&store);
            store
                .insert_attempt(
                    "attempt-1",
                    "ws-1",
                    "problem.shell_y_poly",
                    7,
                    &sample_instance(),
                )
                .unwrap();
        }

        // Scoped so the connection — and the WAL and SHM files it holds open — is closed
        // before the directory is removed: Windows refuses to delete a file still open by
        // this process. Asserting after cleanup keeps a failure from leaking the temp dir.
        let instance = {
            let reopened = PracticeStore::new(crate::db::open(&db_path).unwrap());
            reopened.load_attempt("attempt-1", "ws-1").unwrap().instance
        };

        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(instance, sample_instance());
    }
}
