use rusqlite::{params, Connection, Error, ErrorCode};

use super::{migrate, open_in_memory, LATEST_SCHEMA_VERSION};

const WORKSPACE_ID: &str = "workspace-calculus";
const GUIDING_GOAL_ID: &str = "goal-guiding";

fn insert_workspace_and_guiding_goal(connection: &mut Connection) {
    let transaction = connection.transaction().unwrap();
    transaction
        .execute(
            "INSERT INTO workspaces (
                id, name, guiding_goal_id, progress, paused
            ) VALUES (?1, 'Calculus II', ?2, 0.25, 0)",
            params![WORKSPACE_ID, GUIDING_GOAL_ID],
        )
        .unwrap();
    transaction
        .execute(
            "INSERT INTO goals (
                id, workspace_id, text, state, created_at, updated_at
            ) VALUES (?1, ?2, 'Prepare for the final', 'Guiding', ?3, ?3)",
            params![GUIDING_GOAL_ID, WORKSPACE_ID, "2026-08-29T12:00:00Z"],
        )
        .unwrap();
    transaction.commit().unwrap();
}

fn insert_concept(connection: &Connection, id: &str, name: &str) {
    connection
        .execute(
            "INSERT INTO concepts (
                id, workspace_id, name, chapter, mastery_state, meaning, on_exam, notes_count
            ) VALUES (?1, ?2, ?3, '7 · Applications of Integration', 'Developing',
                'Needs another worked example', 1, 0)",
            params![id, WORKSPACE_ID, name],
        )
        .unwrap();
}

fn notes_count(connection: &Connection, concept_id: &str) -> i64 {
    connection
        .query_row(
            "SELECT notes_count FROM concepts WHERE id = ?1",
            [concept_id],
            |row| row.get(0),
        )
        .unwrap()
}

#[test]
fn migration_applies_the_current_schema_once() {
    let mut connection = open_in_memory().unwrap();

    migrate(&mut connection).unwrap();

    let version: i64 = connection
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();
    let migration_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();
    let domain_table_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema
             WHERE type = 'table'
               AND name NOT LIKE 'sqlite_%'
               AND name <> 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(version, LATEST_SCHEMA_VERSION);
    assert_eq!(migration_count, LATEST_SCHEMA_VERSION);
    assert_eq!(domain_table_count, 26);
}

#[test]
fn workspace_allows_only_one_guiding_goal() {
    let mut connection = open_in_memory().unwrap();
    insert_workspace_and_guiding_goal(&mut connection);

    let result = connection.execute(
        "INSERT INTO goals (
            id, workspace_id, text, state, created_at, updated_at
        ) VALUES ('goal-second', ?1, 'Build fluency', 'Guiding', ?2, ?2)",
        params![WORKSPACE_ID, "2026-08-29T13:00:00Z"],
    );

    assert!(matches!(
        result,
        Err(Error::SqliteFailure(error, _)) if error.code == ErrorCode::ConstraintViolation
    ));

    let guiding_goal_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM goals WHERE workspace_id = ?1 AND state = 'Guiding'",
            [WORKSPACE_ID],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(guiding_goal_count, 1);
}

#[test]
fn note_triggers_keep_concept_counts_synchronized() {
    let mut connection = open_in_memory().unwrap();
    insert_workspace_and_guiding_goal(&mut connection);
    insert_concept(&connection, "concept-shells", "Shell method");
    insert_concept(&connection, "concept-washers", "Washer method");

    connection
        .execute(
            "INSERT INTO notes (id, workspace_id, concept_id, text, updated_at)
             VALUES ('note-one', ?1, 'concept-shells', 'Radius comes from the axis.', ?2)",
            params![WORKSPACE_ID, "2026-08-29T14:00:00Z"],
        )
        .unwrap();
    assert_eq!(notes_count(&connection, "concept-shells"), 1);
    assert_eq!(notes_count(&connection, "concept-washers"), 0);

    connection
        .execute(
            "UPDATE notes SET concept_id = 'concept-washers' WHERE id = 'note-one'",
            [],
        )
        .unwrap();
    assert_eq!(notes_count(&connection, "concept-shells"), 0);
    assert_eq!(notes_count(&connection, "concept-washers"), 1);

    connection
        .execute("DELETE FROM notes WHERE id = 'note-one'", [])
        .unwrap();
    assert_eq!(notes_count(&connection, "concept-washers"), 0);
}

#[test]
fn practice_tables_exist_after_migration() {
    let connection = crate::db::open_in_memory().unwrap();

    let attempts_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('practice_attempts')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(attempts_columns, 9);

    let submissions_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('practice_submissions')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(submissions_columns, 5);
}
