use rusqlite::params;

use super::{concept, goal, material, module, note, session, workspace};
use super::{CreateWorkspaceInput, Database, SessionIntent, StartSessionInput, Workspace};

fn database() -> Database {
    Database::open_in_memory().unwrap()
}

fn create_workspace(database: &Database) -> Workspace {
    workspace::create_workspace_handler(
        database,
        CreateWorkspaceInput {
            subject: "  Calculus II  ".to_owned(),
            goal_text: "  Prepare for the final  ".to_owned(),
        },
    )
    .unwrap()
}

fn insert_concept(database: &Database, workspace_id: &str, id: &str, name: &str) {
    let connection = database.connection().unwrap();
    connection
        .execute(
            "INSERT INTO concepts (
                id, workspace_id, name, chapter, mastery_state, meaning, on_exam, notes_count
            ) VALUES (?1, ?2, ?3, '7 · Applications of Integration', 'Developing',
                'Needs another worked example', 1, 0)",
            params![id, workspace_id, name],
        )
        .unwrap();
}

#[test]
fn workspace_handlers_create_read_toggle_and_bound_activity() {
    let database = database();
    let created = create_workspace(&database);

    assert_eq!(created.name, "Calculus II");
    assert_eq!(created.offline_availability.len(), 4);
    assert_eq!(
        workspace::get_workspaces_handler(&database).unwrap().len(),
        1
    );
    assert_eq!(
        workspace::get_workspace_handler(&database, &created.id)
            .unwrap()
            .guiding_goal_id,
        created.guiding_goal_id
    );

    let updated = workspace::set_workspace_offline_availability_handler(
        &database,
        &created.id,
        "problemBanks",
        true,
    )
    .unwrap();
    assert!(
        updated
            .offline_availability
            .iter()
            .find(|item| item.kind == "problemBanks")
            .unwrap()
            .enabled
    );

    {
        let connection = database.connection().unwrap();
        for day in 1..=4 {
            connection
                .execute(
                    "INSERT INTO workspace_activity_events (
                        id, workspace_id, occurred_at, summary
                    ) VALUES (?1, ?2, ?3, ?4)",
                    params![
                        format!("event-{day}"),
                        created.id,
                        format!("2026-08-0{day}T12:00:00Z"),
                        format!("Activity {day}"),
                    ],
                )
                .unwrap();
        }
    }
    let activity = workspace::get_recent_activity_handler(&database, &created.id).unwrap();
    assert_eq!(activity.len(), 3);
    assert_eq!(activity[0].id, "event-1");
    assert_eq!(activity[2].id, "event-3");
}

#[test]
fn goal_handlers_preserve_previous_text_and_revert() {
    let database = database();
    let workspace = create_workspace(&database);

    let goal = goal::get_goal_handler(&database, &workspace.guiding_goal_id).unwrap();
    assert_eq!(goal.text, "Prepare for the final");
    assert_eq!(
        goal::get_goals_by_workspace_handler(&database, &workspace.id)
            .unwrap()
            .len(),
        1
    );

    let updated = goal::update_goal_handler(
        &database,
        &workspace.guiding_goal_id,
        "  Build shell-method fluency  ",
    )
    .unwrap();
    assert_eq!(updated.text, "Build shell-method fluency");
    assert_eq!(
        updated.previous_text.as_deref(),
        Some("Prepare for the final")
    );

    let reverted = goal::revert_goal_handler(&database, &workspace.guiding_goal_id).unwrap();
    assert_eq!(reverted.text, "Prepare for the final");
    assert_eq!(
        reverted.previous_text.as_deref(),
        Some("Build shell-method fluency")
    );
}

#[test]
fn concept_handlers_reconstruct_relations_and_search_evidence() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    insert_concept(&database, &workspace.id, "concept-washers", "Washer method");
    {
        let connection = database.connection().unwrap();
        connection
            .execute(
                "INSERT INTO concept_edges (
                    source_concept_id, edge_kind, position, target_concept_id
                ) VALUES ('concept-shells', 'related', 0, 'concept-washers')",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO concept_where_it_shows_up (
                    concept_id, position, description
                ) VALUES ('concept-shells', 0, 'Choosing a radius from a shifted axis')",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO concept_diagnostics (
                    id, concept_id, expression, diagnostic_type, note, occurred_at
                ) VALUES ('diagnostic-1', 'concept-shells', '2πrh', 'positive',
                    'Radius identified correctly', '2026-08-29T12:00:00Z')",
                [],
            )
            .unwrap();
    }

    let shell = concept::get_concept_handler(&database, "concept-shells").unwrap();
    assert_eq!(shell.related_concept_ids, vec!["concept-washers"]);
    assert_eq!(shell.recent_diagnostics.unwrap().len(), 1);
    assert_eq!(
        concept::get_concepts_by_workspace_handler(&database, &workspace.id)
            .unwrap()
            .len(),
        2
    );
    let search =
        concept::search_concepts_handler(&database, &workspace.id, "shifted axis").unwrap();
    assert_eq!(search.len(), 1);
    assert_eq!(search[0].id, "concept-shells");
}

#[test]
fn module_handlers_reconstruct_catalog_and_scope_mutations_to_workspace() {
    let database = database();
    let workspace = create_workspace(&database);
    {
        let connection = database.connection().unwrap();
        connection
            .execute_batch(
                "INSERT INTO modules (
                    id, name, icon, developer, description, context_seen, offline_status,
                    enabled, visibility
                ) VALUES
                    ('module-visualizer', 'Function Visualizer', 'V', 'Axiom',
                     'Makes shell construction visible.', 'The current concept.',
                     'Works offline', 0, 'off'),
                    ('module-tutor', 'Socratic Tutor', 'T', 'Axiom',
                     'Asks one useful question at a time.', 'The current problem.',
                     'Online enhanced', 1, 'workspace');
                 INSERT INTO module_supported_concepts (module_id, position, concept_name)
                 VALUES ('module-visualizer', 0, 'Shell method');
                 INSERT INTO module_dependencies (module_id, position, works_with_module_id)
                 VALUES ('module-visualizer', 0, 'module-tutor');
                 INSERT INTO module_suitability (module_id, position, description)
                 VALUES ('module-visualizer', 0, 'Learners who think visually');
                 INSERT INTO module_privacy_notes (module_id, position, sentence)
                 VALUES ('module-visualizer', 0, 'Nothing leaves your device');
                 INSERT INTO workspace_templates (id, name, description, tool_count)
                 VALUES ('template-visual', 'Visual Learner', 'A visual starting point.', 4);",
            )
            .unwrap();
    }

    let catalog_module = module::get_module_handler(&database, "module-visualizer").unwrap();
    assert_eq!(
        catalog_module.supported_concept_names.unwrap(),
        vec!["Shell method"]
    );
    assert_eq!(
        module::get_marketplace_modules_handler(&database, None)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        module::get_workspace_templates_handler(&database).unwrap()[0].tool_count,
        4
    );
    assert!(
        !module::get_modules_by_workspace_handler(&database, &workspace.id).unwrap()[0].enabled
    );

    let installed =
        module::install_module_handler(&database, &workspace.id, "module-visualizer").unwrap();
    assert!(installed.enabled);
    assert!(
        module::get_marketplace_modules_handler(&database, Some(&workspace.id)).unwrap()[0].enabled
    );

    let disabled =
        module::set_module_enabled_handler(&database, &workspace.id, "module-visualizer", false)
            .unwrap();
    assert!(!disabled.enabled);
    let contextual = module::set_module_visibility_handler(
        &database,
        &workspace.id,
        "module-visualizer",
        "contextual",
    )
    .unwrap();
    assert!(contextual.enabled);
    assert_eq!(contextual.visibility, "contextual");
}

#[test]
fn session_handlers_cover_the_full_lifecycle() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");

    assert!(
        session::get_active_session_by_workspace_handler(&database, &workspace.id)
            .unwrap()
            .is_none()
    );
    let started = session::start_session_handler(
        &database,
        StartSessionInput {
            workspace_id: workspace.id.clone(),
            concept_id: "concept-shells".to_owned(),
            intent: SessionIntent {
                activity: "Practising".to_owned(),
                detail: Some("Non-zero axes".to_owned()),
                target_minutes: Some(8),
            },
        },
    )
    .unwrap();
    assert_eq!(
        started.resume_summary,
        "Ready to continue with Shell method."
    );
    assert_eq!(
        session::get_session_handler(&database, &started.id)
            .unwrap()
            .id,
        started.id
    );
    assert_eq!(
        session::get_active_session_by_workspace_handler(&database, &workspace.id)
            .unwrap()
            .unwrap()
            .id,
        started.id
    );

    let paused = session::pause_session_handler(&database, &started.id).unwrap();
    assert_eq!(paused.status, "paused");
    assert!(paused.paused_at.is_some());
    let resumed = session::resume_session_handler(&database, &started.id).unwrap();
    assert_eq!(resumed.status, "active");
    assert!(resumed.paused_at.is_none());
    let with_exchange = session::add_tutor_exchange_handler(
        &database,
        &started.id,
        "  What should I use as the radius?  ",
    )
    .unwrap();
    assert_eq!(with_exchange.exchanges.len(), 1);
    assert_eq!(
        with_exchange.exchanges[0].question,
        "What should I use as the radius?"
    );
    let completed = session::end_session_handler(&database, &started.id).unwrap();
    assert_eq!(completed.status, "completed");
    assert!(session::resume_session_handler(&database, &started.id).is_err());
}

#[test]
fn material_handlers_reconstruct_book_and_exclude_out_of_syllabus_results() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    {
        let connection = database.connection().unwrap();
        connection
            .execute(
                "INSERT INTO materials (
                    id, workspace_id, title, edition, total_pages, total_chapters,
                    highlights_count, notes_count
                ) VALUES ('material-calculus', ?1, 'Calculus', '9th edition', 712, 18, 31, 8)",
                [&workspace.id],
            )
            .unwrap();
        for (position, label, status) in [
            (0, "Ch 6–7", "read"),
            (1, "Ch 8", "inProgress"),
            (2, "Ch 10–11", "next"),
            (3, "Ch 12–18", "outOfSyllabus"),
        ] {
            connection
                .execute(
                    "INSERT INTO material_chapter_segments (
                        material_id, position, label, status
                    ) VALUES ('material-calculus', ?1, ?2, ?3)",
                    params![position, label, status],
                )
                .unwrap();
        }
        connection
            .execute_batch(
                "INSERT INTO material_most_marked_sections (material_id, position, section)
                 VALUES ('material-calculus', 0, '§7.3');
                 INSERT INTO material_results (
                    id, material_id, kind, page, title, reason, concept_id, in_syllabus
                 ) VALUES
                    ('result-shells', 'material-calculus', 'section', 421,
                     '§7.3 · Shell radius', 'Builds the shifted-axis setup.',
                     'concept-shells', 1),
                    ('result-series', 'material-calculus', 'section', 640,
                     '§11.2 · Series', 'Outside the current syllabus.',
                     'concept-shells', 0);",
            )
            .unwrap();
    }

    let material = material::get_material_handler(&database, &workspace.id).unwrap();
    assert_eq!(material.total_pages, 712);
    assert_eq!(material.segments.len(), 4);
    assert_eq!(material.most_marked_sections, vec!["§7.3"]);

    let search = material::search_material_handler(&database, &workspace.id, "shell axis").unwrap();
    assert_eq!(search.len(), 1);
    assert_eq!(search[0].id, "result-shells");
    let all_searchable = material::search_material_handler(&database, &workspace.id, "").unwrap();
    assert_eq!(all_searchable.len(), 1);
    assert_ne!(all_searchable[0].id, "result-series");
}

#[test]
fn note_handler_returns_workspace_notes_newest_first() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    {
        let connection = database.connection().unwrap();
        connection
            .execute(
                "INSERT INTO notes (id, workspace_id, concept_id, text, updated_at)
                 VALUES ('note-old', ?1, 'concept-shells', 'Older note', '2026-08-20T12:00:00Z')",
                [&workspace.id],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO notes (id, workspace_id, concept_id, text, updated_at)
                 VALUES ('note-new', ?1, 'concept-shells', 'Newer note', '2026-08-29T12:00:00Z')",
                [&workspace.id],
            )
            .unwrap();
    }

    let notes = note::get_recent_notes_handler(&database, &workspace.id).unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].id, "note-new");
    assert_eq!(notes[1].id, "note-old");
}

#[test]
fn command_dtos_use_frontend_camel_case_fields() {
    let database = database();
    let workspace = create_workspace(&database);
    let value = serde_json::to_value(workspace).unwrap();

    assert!(value.get("guidingGoalId").is_some());
    assert!(value.get("offlineAvailability").is_some());
    assert!(value.get("enabledModuleIds").is_some());
    assert!(value.get("guiding_goal_id").is_none());
    assert!(value.get("lastConceptName").is_none());

    let input: CreateWorkspaceInput = serde_json::from_value(serde_json::json!({
        "subject": "Physics",
        "goalText": "Prepare for mechanics"
    }))
    .unwrap();
    assert_eq!(input.goal_text, "Prepare for mechanics");
}
