use rusqlite::params;
use std::sync::Arc;
use tauri::async_runtime::RwLock;

use super::{concept, goal, material, module, note, seed, session, workspace};
use super::{
    CreateWorkspaceInput, Database, SampleWorkspaceSeed, SessionIntent, StartSessionInput,
    Workspace,
};
use crate::modules::{
    CallEnvelope, CapabilityCall, CapabilityId, CapabilityProvider, CapabilityRequirement,
    InvocationError, ModuleId,
};
use crate::modules::{ModuleInstallation, ModuleRegistry};
use crate::practice::{DescribeRequest, DescribeResponse};

fn database() -> Database {
    Database::open_in_memory().unwrap()
}

fn create_workspace(database: &Database) -> Workspace {
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let installation = ModuleInstallation {
        workspace_id: String::new(),
        enabled_module_ids: Vec::new(),
    };
    tauri::async_runtime::block_on(workspace::create_workspace_handler(
        database,
        &registry,
        &installation,
        CreateWorkspaceInput {
            subject: "  Calculus II  ".to_owned(),
            goal_text: "  Prepare for the final  ".to_owned(),
        },
    ))
    .unwrap()
}

fn create_workspace_with_registry(
    database: &Database,
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
) -> Workspace {
    tauri::async_runtime::block_on(workspace::create_workspace_handler(
        database,
        registry,
        installation,
        CreateWorkspaceInput {
            subject: "Calculus II".to_owned(),
            goal_text: "Prepare for the final".to_owned(),
        },
    ))
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

fn map_concept_to_knowledge_package(database: &Database, concept_id: &str) {
    database
        .connection()
        .unwrap()
        .execute(
            "UPDATE concepts SET knowledge_concept_id = 'shell.method_vertical_axis'
             WHERE id = ?1",
            [concept_id],
        )
        .unwrap();
}

fn session_input(workspace_id: &str) -> StartSessionInput {
    StartSessionInput {
        workspace_id: workspace_id.to_owned(),
        concept_id: "concept-shells".to_owned(),
        intent: SessionIntent {
            activity: "Practising".to_owned(),
            detail: None,
            target_minutes: Some(8),
        },
    }
}

fn practice_connection(workspace_id: &str) -> rusqlite::Connection {
    let mut connection = crate::db::open_in_memory().unwrap();
    let transaction = connection.transaction().unwrap();
    transaction
        .execute(
            "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
             VALUES (?1, 'Test', 'goal-practice', 0.0, 0)",
            [workspace_id],
        )
        .unwrap();
    transaction
        .execute(
            "INSERT INTO goals (id, workspace_id, text, state, created_at, updated_at)
             VALUES ('goal-practice', ?1, 'Test goal', 'Guiding', ?2, ?2)",
            params![workspace_id, "2026-09-08T12:00:00Z"],
        )
        .unwrap();
    transaction.commit().unwrap();
    connection
}

fn fixture_knowledge_package() -> crate::knowledge::KnowledgePackage {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/knowledge/tests/fixtures/canonical");
    crate::knowledge::load_knowledge_package(&fixture_root).unwrap()
}

struct FailingStartProvider;

#[async_trait::async_trait]
impl CapabilityProvider for FailingStartProvider {
    async fn invoke(
        &self,
        _capability_id: &CapabilityId,
        _version: u32,
        _input: serde_json::Value,
    ) -> Result<serde_json::Value, InvocationError> {
        Err(InvocationError::Failed {
            message: "forced practice.start failure".to_owned(),
        })
    }
}

struct FailingConceptsProvider;

#[async_trait::async_trait]
impl CapabilityProvider for FailingConceptsProvider {
    async fn invoke(
        &self,
        _capability_id: &CapabilityId,
        _version: u32,
        _input: serde_json::Value,
    ) -> Result<serde_json::Value, InvocationError> {
        Err(InvocationError::Failed {
            message: "forced practice.concepts failure".to_owned(),
        })
    }
}

fn sample_seed() -> SampleWorkspaceSeed {
    serde_json::from_value(serde_json::json!({
        "sampleWorkspaceId": "sample-workspace",
        "workspaces": [
            {
                "id": "sample-workspace",
                "name": "Calculus II",
                "guidingGoalId": "sample-goal",
                "progress": 0,
                "lastConceptName": "Shell method",
                "paused": false,
                "offlineAvailability": [{
                    "kind": "textbookAndLectureNotes",
                    "enabled": true,
                    "sizeBytes": 880803840
                }],
                "enabledModuleIds": ["sample-module"]
            },
            {
                "id": "sample-linear-workspace",
                "name": "Linear Algebra",
                "guidingGoalId": "sample-linear-goal",
                "progress": 0,
                "lastConceptName": "Eigenvectors",
                "paused": false,
                "offlineAvailability": [],
                "enabledModuleIds": []
            },
            {
                "id": "sample-physics-workspace",
                "name": "Mechanics",
                "guidingGoalId": "sample-physics-goal",
                "progress": 0,
                "lastConceptName": "Angular momentum",
                "paused": true,
                "offlineAvailability": [],
                "enabledModuleIds": []
            }
        ],
        "workspaceActivity": [{
            "id": "sample-activity",
            "workspaceId": "sample-workspace",
            "occurredAt": "2026-08-28T12:00:00.000Z",
            "summary": "A worked example was linked to Shell method."
        }],
        "goals": [
            {
                "id": "sample-goal",
                "workspaceId": "sample-workspace",
                "text": "Explain and solve every integration technique.",
                "state": "Guiding",
                "inferred": {
                    "conceptScope": 87,
                    "tools": ["Practice", "Visualizer"]
                },
                "createdAt": "2026-08-18T13:00:00.000Z",
                "updatedAt": "2026-08-26T17:30:00.000Z"
            },
            {
                "id": "sample-linear-goal",
                "workspaceId": "sample-linear-workspace",
                "text": "Write clear proofs about vector spaces.",
                "state": "Guiding",
                "inferred": {},
                "createdAt": "2026-08-18T13:00:00.000Z",
                "updatedAt": "2026-08-26T17:30:00.000Z"
            },
            {
                "id": "sample-physics-goal",
                "workspaceId": "sample-physics-workspace",
                "text": "Retain the mechanics needed for electromagnetism.",
                "state": "Guiding",
                "inferred": {},
                "createdAt": "2026-08-18T13:00:00.000Z",
                "updatedAt": "2026-08-26T17:30:00.000Z"
            }
        ],
        "concepts": [{
            "id": "sample-concept",
            "workspaceId": "sample-workspace",
            "name": "Shell method",
            "chapter": "6 · Applications of Integration",
            "masteryState": "Developing",
            "meaning": "Works with support but not yet independently.",
            "onExam": true,
            "blocksConceptIds": [],
            "prerequisiteConceptIds": [],
            "relatedConceptIds": [],
            "leadsToConceptIds": [],
            "whereItShowsUp": ["Volume by rotation"],
            "recentDiagnostics": [{
                "id": "sample-diagnostic",
                "expression": "2πx(4-x²)",
                "type": "positive",
                "note": "Radius and height were identified correctly.",
                "occurredAt": "2026-08-27T19:10:00.000Z"
            }],
            "notesCount": 4
        }],
        "modules": [{
            "id": "sample-module",
            "name": "Function Visualizer",
            "icon": "V",
            "trust": "verified",
            "developer": "Axiom Labs",
            "price": "Free",
            "description": "Makes shell construction visible.",
            "contextSeen": "The current concept.",
            "offlineStatus": "Works offline",
            "supportedConceptNames": ["Shell method"],
            "suits": ["Learners who think visually"],
            "privacyNotes": ["Nothing leaves your device"],
            "enabled": true,
            "visibility": "workspace"
        }],
        "workspaceTemplates": [{
            "id": "sample-template",
            "name": "Visual Learner",
            "description": "A visual starting point.",
            "toolCount": 7
        }],
        "materials": [{
            "id": "sample-material",
            "workspaceId": "sample-workspace",
            "title": "Calculus",
            "edition": "9th edition",
            "totalPages": 712,
            "totalChapters": 18,
            "segments": [
                { "label": "Ch 6–7", "status": "read" },
                { "label": "Ch 8", "status": "inProgress" },
                { "label": "Ch 10–11", "status": "next", "detail": "33 sections" },
                { "label": "Ch 12–18", "status": "outOfSyllabus" }
            ],
            "highlightsCount": 41,
            "notesCount": 18,
            "mostMarkedSections": ["§7.3"]
        }],
        "materialResults": [{
            "id": "sample-result",
            "kind": "section",
            "page": 442,
            "title": "§7.3 · Volumes by Cylindrical Shells",
            "reason": "Matches the current radius practice.",
            "conceptId": "sample-concept",
            "inSyllabus": true
        }],
        "notes": [{
            "id": "sample-note",
            "workspaceId": "sample-workspace",
            "conceptId": "sample-concept",
            "text": "Measure the shell radius from the axis.",
            "updatedAt": "2026-08-27T18:40:00.000Z"
        }]
    }))
    .unwrap()
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
fn workspace_creation_provisions_bundled_concepts_with_opaque_crosswalks() {
    let database = database();
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        bundled_knowledge_package(),
        crate::db::open_in_memory().unwrap(),
    );

    let workspace = create_workspace_with_registry(&database, &registry, &installation);
    let connection = database.connection().unwrap();
    let mut statement = connection
        .prepare(
            "SELECT name, chapter, meaning, mastery_state, on_exam, knowledge_concept_id
             FROM concepts WHERE workspace_id = ?1 ORDER BY knowledge_concept_id",
        )
        .unwrap();
    let concepts = statement
        .query_map([&workspace.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, bool>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();

    assert_eq!(concepts.len(), 3);
    assert!(concepts.iter().all(|concept| concept.3 == "New"));
    assert!(concepts.iter().all(|concept| !concept.4));
    assert!(concepts
        .iter()
        .all(|concept| !concept.0.is_empty() && !concept.1.is_empty() && !concept.2.is_empty()));
    assert!(concepts
        .iter()
        .any(|concept| concept.5 == "shell.method_vertical_axis"));
}

#[test]
fn workspace_creation_succeeds_with_zero_concepts_when_capability_is_absent() {
    let database = database();
    let workspace = create_workspace(&database);
    let count: i64 = database
        .connection()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM concepts WHERE workspace_id = ?1",
            [&workspace.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn workspace_creation_succeeds_with_zero_concepts_when_practice_is_disabled() {
    let database = database();
    let (registry, mut installation) = crate::commands::practice::build_practice_registry(
        bundled_knowledge_package(),
        crate::db::open_in_memory().unwrap(),
    );
    installation
        .enabled_module_ids
        .retain(|module_id| module_id.as_str() != "org.axiom.practice");

    let workspace = create_workspace_with_registry(&database, &registry, &installation);
    let count: i64 = database
        .connection()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM concepts WHERE workspace_id = ?1",
            [&workspace.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn workspace_creation_succeeds_with_zero_concepts_when_capability_invocation_fails() {
    let database = database();
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            crate::modules::parse(
                r#"
                    manifest_version = 1
                    id = "test.failing_concepts"
                    name = "Failing Concepts"
                    version = "1.0.0"
                    minimum_axiom_version = "0.1.0"
                    offline = "full"

                    [[provides]]
                    id = "practice.concepts"
                    version = 1
                "#,
            )
            .unwrap(),
            Box::new(FailingConceptsProvider),
        )
        .unwrap();
    let registry = Arc::new(RwLock::new(registry));
    let installation = ModuleInstallation {
        workspace_id: String::new(),
        enabled_module_ids: vec![ModuleId::new("test.failing_concepts").unwrap()],
    };

    let workspace = create_workspace_with_registry(&database, &registry, &installation);
    let count: i64 = database
        .connection()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM concepts WHERE workspace_id = ?1",
            [&workspace.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn a_created_workspace_shell_concept_starts_a_real_attempt() {
    let database = database();
    let (provisioning_registry, provisioning_installation) =
        crate::commands::practice::build_practice_registry(
            bundled_knowledge_package(),
            crate::db::open_in_memory().unwrap(),
        );
    let workspace = create_workspace_with_registry(
        &database,
        &provisioning_registry,
        &provisioning_installation,
    );
    let shell_concept_id: String = database
        .connection()
        .unwrap()
        .query_row(
            "SELECT id FROM concepts
             WHERE workspace_id = ?1 AND knowledge_concept_id = 'shell.method_vertical_axis'",
            [&workspace.id],
            |row| row.get(0),
        )
        .unwrap();
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        bundled_knowledge_package(),
        practice_connection(&workspace.id),
    );

    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        StartSessionInput {
            workspace_id: workspace.id,
            concept_id: shell_concept_id,
            intent: SessionIntent {
                activity: "Practising".to_owned(),
                detail: None,
                target_minutes: Some(8),
            },
        },
    ))
    .unwrap();

    assert!(started.current_attempt_id.is_some());
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
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let installation = ModuleInstallation {
        workspace_id: workspace.id.clone(),
        enabled_module_ids: Vec::new(),
    };
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        StartSessionInput {
            workspace_id: workspace.id.clone(),
            concept_id: "concept-shells".to_owned(),
            intent: SessionIntent {
                activity: "Practising".to_owned(),
                detail: Some("Non-zero axes".to_owned()),
                target_minutes: Some(8),
            },
        },
    ))
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
fn start_session_binds_an_attempt_for_a_mapped_concept_when_practice_is_enabled() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );

    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let attempt_id = started.current_attempt_id.unwrap();
    let requirement = CapabilityRequirement {
        id: CapabilityId::new("practice.describe").unwrap(),
        min_version: 1,
    };
    let described: DescribeResponse = tauri::async_runtime::block_on(async {
        let registry = registry.read().await;
        let handle = registry.resolve(&installation, &requirement).unwrap();
        registry
            .invoke(
                &handle,
                &installation,
                CapabilityCall {
                    envelope: CallEnvelope {
                        workspace_id: workspace.id.clone(),
                        capability_id: requirement.id,
                        version: 1,
                        calling_module_id: ModuleId::new("core.test_caller").unwrap(),
                    },
                    input: DescribeRequest {
                        workspace_id: workspace.id.clone(),
                        attempt_id,
                    },
                },
            )
            .await
    })
    .unwrap();
    assert!(!described.prompt.is_empty());
    assert_eq!(described.status, crate::practice::AttemptStatus::Open);
}

#[test]
fn start_session_reuses_an_open_session_and_its_attempt() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );

    let first = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let second = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let third = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();

    assert_eq!(second.id, first.id);
    assert_eq!(third.id, first.id);
    assert_eq!(second.current_attempt_id, first.current_attempt_id);
    assert_eq!(third.current_attempt_id, first.current_attempt_id);
    let count: i64 = database
        .connection()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE workspace_id = ?1 AND concept_id = ?2",
            params![workspace.id, "concept-shells"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn active_session_selection_prefers_recent_activity_then_newest_row() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    let connection = database.connection().unwrap();
    for (id, activity) in [
        ("session-older", Some("2026-09-08T10:00:00.000Z")),
        ("session-null", None),
        ("session-recent", Some("2026-09-08T12:00:00.000Z")),
    ] {
        connection
            .execute(
                "INSERT INTO sessions (
                    id, workspace_id, concept_id, status, intent_activity, resume_summary,
                    elapsed_minutes, started_at, last_activity_at
                 ) VALUES (?1, ?2, 'concept-shells', 'active', 'Practising',
                    'Ready.', 0, '2026-09-08T09:00:00.000Z', ?3)",
                params![id, workspace.id, activity],
            )
            .unwrap();
    }
    drop(connection);

    let selected = session::get_active_session_by_workspace_handler(&database, &workspace.id)
        .unwrap()
        .unwrap();
    assert_eq!(selected.id, "session-recent");

    let connection = database.connection().unwrap();
    connection
        .execute("UPDATE sessions SET last_activity_at = NULL", [])
        .unwrap();
    drop(connection);
    let selected = session::get_active_session_by_workspace_handler(&database, &workspace.id)
        .unwrap()
        .unwrap();
    assert_eq!(
        selected.id, "session-recent",
        "all-null activity falls back to rowid DESC"
    );
}

#[test]
fn start_session_rebinds_a_solved_attempt_on_the_same_session() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );
    let first = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let first_attempt = first.current_attempt_id.clone().unwrap();
    let described =
        tauri::async_runtime::block_on(crate::commands::practice::describe_attempt_handler(
            &registry,
            &installation,
            crate::commands::practice::DescribeAttemptInput {
                workspace_id: workspace.id.clone(),
                attempt_id: first_attempt.clone(),
            },
        ))
        .unwrap();
    let (coeff, b) = parameters_from_prompt(&described.prompt);
    tauri::async_runtime::block_on(crate::commands::practice::evaluate_attempt_handler(
        &database,
        &registry,
        &installation,
        crate::commands::practice::EvaluateAttemptInput {
            workspace_id: workspace.id.clone(),
            attempt_id: first_attempt.clone(),
            response: crate::commands::practice::ResponseValueInput::SymbolicExpression {
                value: format!("2*pi*({coeff}*{b}^3/3 - {b}^4/4)"),
            },
        },
    ))
    .unwrap();

    let restarted = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    assert_eq!(restarted.id, first.id);
    assert_ne!(restarted.current_attempt_id, Some(first_attempt));
    assert_eq!(restarted.problem_index, Some(2));
}

#[test]
fn session_activity_updates_both_timestamps_without_changing_elapsed_minutes() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    assert!(started.last_activity_at.is_some());
    assert_eq!(started.elapsed_minutes, 0);
    assert_eq!(
        workspace::get_workspace_handler(&database, &workspace.id)
            .unwrap()
            .last_activity_at,
        started.last_activity_at
    );

    for changed in [
        session::pause_session_handler(&database, &started.id).unwrap(),
        session::resume_session_handler(&database, &started.id).unwrap(),
    ] {
        assert!(changed.last_activity_at.is_some());
        assert_eq!(changed.elapsed_minutes, 0);
        assert_eq!(
            workspace::get_workspace_handler(&database, &workspace.id)
                .unwrap()
                .last_activity_at,
            changed.last_activity_at
        );
    }
    let attempt_id = started.current_attempt_id.clone().unwrap();
    tauri::async_runtime::block_on(crate::commands::practice::evaluate_attempt_handler(
        &database,
        &registry,
        &installation,
        crate::commands::practice::EvaluateAttemptInput {
            workspace_id: workspace.id.clone(),
            attempt_id,
            response: crate::commands::practice::ResponseValueInput::SymbolicExpression {
                value: "0".to_owned(),
            },
        },
    ))
    .unwrap();
    let evaluated = session::get_session_handler(&database, &started.id).unwrap();
    assert_eq!(evaluated.elapsed_minutes, 0);
    assert_eq!(
        workspace::get_workspace_handler(&database, &workspace.id)
            .unwrap()
            .last_activity_at,
        evaluated.last_activity_at
    );
    let advanced = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &registry,
        &installation,
        &started.id,
    ))
    .unwrap();
    assert_eq!(advanced.elapsed_minutes, 0);
    assert_eq!(
        workspace::get_workspace_handler(&database, &workspace.id)
            .unwrap()
            .last_activity_at,
        advanced.last_activity_at
    );
}

#[test]
fn non_session_mutations_do_not_advance_workspace_activity() {
    let database = database();
    let workspace = create_workspace(&database);
    assert_eq!(workspace.last_activity_at, None);
    workspace::set_workspace_offline_availability_handler(
        &database,
        &workspace.id,
        "problemBanks",
        true,
    )
    .unwrap();
    goal::update_goal_handler(&database, &workspace.guiding_goal_id, "Changed goal").unwrap();
    assert_eq!(
        workspace::get_workspace_handler(&database, &workspace.id)
            .unwrap()
            .last_activity_at,
        None
    );
}

#[test]
fn start_session_rejects_a_missing_workspace_without_creating_a_row() {
    let database = database();
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let installation = ModuleInstallation {
        workspace_id: "missing".to_owned(),
        enabled_module_ids: Vec::new(),
    };
    let result = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input("missing"),
    ));
    assert_eq!(result.unwrap_err(), "Workspace not found: missing");
    let count: i64 = database
        .connection()
        .unwrap()
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn start_session_leaves_attempt_unbound_for_an_unmapped_concept() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let installation = ModuleInstallation {
        workspace_id: workspace.id.clone(),
        enabled_module_ids: Vec::new(),
    };

    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();

    assert_eq!(started.current_attempt_id, None);
}

#[test]
fn start_session_succeeds_without_an_attempt_when_practice_is_disabled() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, mut installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );
    installation
        .enabled_module_ids
        .retain(|module_id| module_id.as_str() != "org.axiom.practice");

    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();

    assert_eq!(started.current_attempt_id, None);
}

#[test]
fn start_session_succeeds_without_an_attempt_when_practice_start_fails() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            crate::modules::parse(
                r#"
                    manifest_version = 1
                    id = "test.failing_practice"
                    name = "Failing Practice"
                    version = "1.0.0"
                    minimum_axiom_version = "0.1.0"
                    offline = "full"

                    [[provides]]
                    id = "practice.start"
                    version = 1
                "#,
            )
            .unwrap(),
            Box::new(FailingStartProvider),
        )
        .unwrap();
    let registry = Arc::new(RwLock::new(registry));
    let installation = ModuleInstallation {
        workspace_id: workspace.id.clone(),
        enabled_module_ids: vec![ModuleId::new("test.failing_practice").unwrap()],
    };

    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();

    assert_eq!(started.current_attempt_id, None);
}

#[test]
fn next_problem_rebinds_the_session_and_advances_the_counter() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let first_attempt = started.current_attempt_id.clone().unwrap();

    let advanced = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &registry,
        &installation,
        &started.id,
    ))
    .unwrap();

    assert_ne!(advanced.current_attempt_id, Some(first_attempt));
    assert!(advanced.current_attempt_id.is_some());
    assert_eq!(advanced.problem_index, Some(2));
}

#[test]
fn next_problem_leaves_the_counter_alone_when_practice_is_unavailable() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let installation = ModuleInstallation {
        workspace_id: workspace.id.clone(),
        enabled_module_ids: Vec::new(),
    };
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();

    let advanced = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &registry,
        &installation,
        &started.id,
    ))
    .unwrap();

    assert_eq!(advanced.current_attempt_id, None);
    assert_eq!(advanced.problem_index, started.problem_index);
}

#[test]
fn next_problem_keeps_the_existing_attempt_when_practice_fails() {
    // The design spec said `current_attempt_id` becomes NULL when Practice fails. On a
    // rebind that would destroy the problem the learner is part-way through, so the
    // handler leaves the existing binding alone instead. The pre-existing
    // "practice unavailable" test cannot show this, because there the session never had
    // an attempt to keep -- None passes under either behaviour. This pins the difference.
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let bound = started.current_attempt_id.clone().unwrap();

    // A registry with no Practice provider stands in for generation being unavailable.
    let empty_registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let empty_installation = ModuleInstallation {
        workspace_id: workspace.id.clone(),
        enabled_module_ids: Vec::new(),
    };
    let advanced = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &empty_registry,
        &empty_installation,
        &started.id,
    ))
    .unwrap();

    assert_eq!(
        advanced.current_attempt_id,
        Some(bound),
        "a failed rebind must not discard the attempt the learner is working on"
    );
    assert_eq!(advanced.problem_index, started.problem_index);
}

#[test]
fn next_problem_on_a_completed_session_is_rejected() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    session::end_session_handler(&database, &started.id).unwrap();

    let result = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &registry,
        &installation,
        &started.id,
    ));

    assert!(
        result.is_err(),
        "a completed session must not be advanced to a new problem"
    );
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
fn sample_import_normalizes_the_seed_and_preserves_owned_counts() {
    let database = database();
    let imported = seed::import_sample_workspace_handler(&database, &sample_seed()).unwrap();

    assert_eq!(imported.id, "sample-workspace");
    assert_eq!(imported.enabled_module_ids, vec!["sample-module"]);
    assert_eq!(
        goal::get_goals_by_workspace_handler(&database, &imported.id)
            .unwrap()
            .len(),
        1
    );
    let concept = concept::get_concept_handler(&database, "sample-concept").unwrap();
    assert_eq!(concept.notes_count, 1);
    assert_eq!(
        concept.where_it_shows_up.unwrap(),
        vec!["Volume by rotation"]
    );
    assert_eq!(concept.recent_diagnostics.unwrap().len(), 1);
    assert_eq!(
        module::get_workspace_templates_handler(&database)
            .unwrap()
            .len(),
        1
    );
    assert!(
        module::get_module_handler(&database, "sample-module")
            .unwrap()
            .enabled
    );
    let workspaces = workspace::get_workspaces_handler(&database).unwrap();
    assert_eq!(workspaces.len(), 3);
    for workspace in workspaces {
        assert_eq!(workspace.progress, 0.0);
        assert_eq!(workspace.last_activity_at, None);
        assert_eq!(
            session::get_active_session_by_workspace_handler(&database, &workspace.id).unwrap(),
            None
        );
    }
    {
        let connection = database.connection().unwrap();
        for table in ["sessions", "tutor_exchanges", "session_settled_conclusions"] {
            let row_count: i64 = connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(row_count, 0, "sample import wrote activity rows to {table}");
        }
    }
    let material = material::get_material_handler(&database, &imported.id).unwrap();
    assert_eq!(material.highlights_count, 41);
    assert_eq!(material.notes_count, 18);
    assert_eq!(
        material::search_material_handler(&database, &imported.id, "radius")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        note::get_recent_notes_handler(&database, &imported.id)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        workspace::get_recent_activity_handler(&database, &imported.id)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn sample_seed_crosswalk_resolves_to_a_bundled_knowledge_concept() {
    let database = database();
    seed::import_sample_workspace_handler(&database, &sample_seed()).unwrap();

    let knowledge_concept_id: String = database
        .connection()
        .unwrap()
        .query_row(
            "SELECT knowledge_concept_id FROM concepts WHERE name = 'Shell method'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let package_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package");
    let package = crate::knowledge::load_knowledge_package(&package_root).unwrap();
    assert!(package
        .concepts
        .iter()
        .any(|concept| concept.id.as_str() == knowledge_concept_id));
}

#[test]
fn sample_import_is_idempotent_and_does_not_reset_existing_sample_work() {
    let database = database();
    let seed = sample_seed();
    seed::import_sample_workspace_handler(&database, &seed).unwrap();
    {
        let connection = database.connection().unwrap();
        connection
            .execute(
                "UPDATE workspaces SET name = 'My Calculus II' WHERE id = 'sample-workspace'",
                [],
            )
            .unwrap();
    }

    let imported_again = seed::import_sample_workspace_handler(&database, &seed).unwrap();
    assert_eq!(imported_again.name, "My Calculus II");
    assert_eq!(
        workspace::get_workspaces_handler(&database).unwrap().len(),
        3
    );
}

#[test]
fn sample_import_rolls_back_every_table_when_one_fixture_is_invalid() {
    let database = database();
    let mut seed = sample_seed();
    seed.notes[0].concept_id = "missing-concept".to_owned();

    assert!(seed::import_sample_workspace_handler(&database, &seed).is_err());
    let connection = database.connection().unwrap();
    let workspace_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
        .unwrap();
    let module_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM modules", [], |row| row.get(0))
        .unwrap();
    assert_eq!(workspace_count, 0);
    assert_eq!(module_count, 0);
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

fn describe_via_capability(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    workspace_id: &str,
    attempt_id: &str,
) -> DescribeResponse {
    let requirement = CapabilityRequirement {
        id: CapabilityId::new("practice.describe").unwrap(),
        min_version: 1,
    };
    tauri::async_runtime::block_on(async {
        let registry = registry.read().await;
        let handle = registry.resolve(installation, &requirement).unwrap();
        registry
            .invoke(
                &handle,
                installation,
                CapabilityCall {
                    envelope: CallEnvelope {
                        workspace_id: workspace_id.to_owned(),
                        capability_id: requirement.id,
                        version: 1,
                        calling_module_id: ModuleId::new("core.test_caller").unwrap(),
                    },
                    input: DescribeRequest {
                        workspace_id: workspace_id.to_owned(),
                        attempt_id: attempt_id.to_owned(),
                    },
                },
            )
            .await
            .unwrap()
    })
}

#[test]
fn a_bound_practice_attempt_survives_reopening_the_database_file() {
    // Mirrors production wiring (`lib.rs`): Core and Practice each open their own
    // connection to the same `axiom.sqlite3`. The in-memory helpers used elsewhere in
    // this file cannot cover a restart, so this uses a real file and reopens both
    // connections to stand in for one.
    let dir = std::env::temp_dir().join(format!("axiom-restart-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("axiom.sqlite3");

    let (session_id, workspace_id, attempt_id, first_prompt) = {
        let database = Database::open(&db_path).unwrap();
        let workspace = create_workspace(&database);
        insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
        map_concept_to_knowledge_package(&database, "concept-shells");
        let (registry, installation) = crate::commands::practice::build_practice_registry(
            fixture_knowledge_package(),
            crate::db::open(&db_path).unwrap(),
        );

        let started = tauri::async_runtime::block_on(session::start_session_handler(
            &database,
            &registry,
            &installation,
            session_input(&workspace.id),
        ))
        .unwrap();
        let attempt_id = started.current_attempt_id.clone().unwrap();
        let described =
            describe_via_capability(&registry, &installation, &workspace.id, &attempt_id);
        (started.id, workspace.id, attempt_id, described.prompt)
    };

    // Both connections are dropped above, as on application exit.
    let reopened = Database::open(&db_path).unwrap();
    let session = session::get_session_handler(&reopened, &session_id).unwrap();
    assert_eq!(
        session.current_attempt_id,
        Some(attempt_id.clone()),
        "the session lost its attempt binding across a restart"
    );

    // Scoped, and `reopened` dropped below, so every connection to the file is closed
    // before the directory is removed -- the task 061 failure mode on Windows.
    {
        let (registry, installation) = crate::commands::practice::build_practice_registry(
            fixture_knowledge_package(),
            crate::db::open(&db_path).unwrap(),
        );
        let described =
            describe_via_capability(&registry, &installation, &workspace_id, &attempt_id);
        assert_eq!(
            described.prompt, first_prompt,
            "the resumed attempt is not the same problem the learner was working on"
        );
        assert_eq!(described.hints_revealed, 0);
    }

    drop(reopened);
    // Best-effort, matching `knowledge/tests/mod.rs`, `loader.rs` and `discover.rs`:
    // Windows can still hold a handle to the SQLite file here even with every
    // connection scoped and dropped, and a leaked temp directory is not worth
    // failing a persistence test over. The unique name means runs never collide.
    let _ = std::fs::remove_dir_all(&dir);
}

fn bundled_knowledge_package() -> crate::knowledge::KnowledgePackage {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package");
    crate::knowledge::load_knowledge_package(&root).unwrap()
}

/// Pulls `coeff` and `b` out of the rendered prompt, the way a learner reads them off the
/// page, so the test can compute the answer for whichever instance the seed produced.
fn parameters_from_prompt(prompt: &str) -> (i64, i64) {
    let after_fx = prompt.split("f(x) = ").nth(1).expect("prompt states f(x)");
    let coeff: i64 = after_fx
        .split('x')
        .next()
        .expect("coefficient precedes x")
        .trim()
        .parse()
        .expect("coefficient is an integer");

    let interval = prompt
        .split("interval [")
        .nth(1)
        .expect("prompt states the interval");
    let upper = interval
        .split(']')
        .next()
        .expect("interval closes")
        .split(',')
        .nth(1)
        .expect("interval has an upper bound");
    let b: i64 = upper.trim().parse().expect("upper bound is an integer");

    (coeff, b)
}

/// The whole learner loop across the real command layer: start a session, read the bound
/// problem, get a wrong answer rejected, take a hint, submit the right answer, advance.
///
/// Every step here already had its own test. Nothing exercised them *in sequence* against
/// the bundled knowledge package, which is exactly the composition task 064's criterion 1
/// asks about -- and it is the gap that let `master` ship a practice loop whose commands
/// were not registered while every check stayed green.
#[test]
fn the_full_practice_loop_runs_through_the_command_layer() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        bundled_knowledge_package(),
        practice_connection(&workspace.id),
    );

    // 1. Starting a session binds a generated problem.
    let session = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let attempt_id = session
        .current_attempt_id
        .clone()
        .expect("a mapped concept with Practice enabled must bind an attempt");

    // 2. The problem pane hydrates from the bound attempt.
    let described =
        tauri::async_runtime::block_on(crate::commands::practice::describe_attempt_handler(
            &registry,
            &installation,
            crate::commands::practice::DescribeAttemptInput {
                workspace_id: workspace.id.clone(),
                attempt_id: attempt_id.clone(),
            },
        ))
        .unwrap();
    assert!(!described.prompt.is_empty());
    assert!(described.hints_total > 0, "the family authors four hints");
    assert_eq!(described.hints_revealed, 0);
    assert_eq!(described.submission_count, 0);

    // 3. A wrong answer is rejected and counted, and the attempt stays open.
    let wrong =
        tauri::async_runtime::block_on(crate::commands::practice::evaluate_attempt_handler(
            &database,
            &registry,
            &installation,
            crate::commands::practice::EvaluateAttemptInput {
                workspace_id: workspace.id.clone(),
                attempt_id: attempt_id.clone(),
                response: crate::commands::practice::ResponseValueInput::SymbolicExpression {
                    value: "0".to_owned(),
                },
            },
        ))
        .unwrap();
    assert!(!wrong.correct);
    assert_eq!(wrong.submission_count, 1);

    // 4. A hint is available and reveals authored text, not a placeholder.
    let hint = tauri::async_runtime::block_on(crate::commands::practice::request_hint_handler(
        &registry,
        &installation,
        crate::commands::practice::RequestHintInput {
            workspace_id: workspace.id.clone(),
            attempt_id: attempt_id.clone(),
        },
    ))
    .unwrap();
    assert!(!hint.hint_text.trim().is_empty());
    assert!(
        !hint.hint_text.contains('{'),
        "a surviving placeholder means substitution did not run: {}",
        hint.hint_text
    );
    assert_eq!(hint.hints_revealed, 1);

    // 5. The canonical answer, computed from the prompt the learner was shown, is accepted.
    let (coeff, b) = parameters_from_prompt(&described.prompt);
    let answer = format!("2*pi*({coeff}*{b}^3/3 - {b}^4/4)");
    let right =
        tauri::async_runtime::block_on(crate::commands::practice::evaluate_attempt_handler(
            &database,
            &registry,
            &installation,
            crate::commands::practice::EvaluateAttemptInput {
                workspace_id: workspace.id.clone(),
                attempt_id: attempt_id.clone(),
                response: crate::commands::practice::ResponseValueInput::SymbolicExpression {
                    value: answer.clone(),
                },
            },
        ))
        .unwrap();
    assert!(
        right.correct,
        "answer {answer} derived from prompt {:?} was rejected",
        described.prompt
    );
    assert_eq!(right.submission_count, 2);

    // 6. Advancing binds a different problem, open and ready.
    let advanced = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &registry,
        &installation,
        &session.id,
    ))
    .unwrap();
    let next_attempt_id = advanced
        .current_attempt_id
        .clone()
        .expect("advancing must bind a new attempt");
    assert_ne!(
        next_attempt_id, attempt_id,
        "next problem must be a new attempt"
    );
    assert_eq!(advanced.problem_index, Some(2));

    let next_described =
        tauri::async_runtime::block_on(crate::commands::practice::describe_attempt_handler(
            &registry,
            &installation,
            crate::commands::practice::DescribeAttemptInput {
                workspace_id: workspace.id.clone(),
                attempt_id: next_attempt_id,
            },
        ))
        .unwrap();
    assert!(!next_described.prompt.is_empty());
    assert_eq!(next_described.submission_count, 0);
    assert_eq!(next_described.hints_revealed, 0);
}
