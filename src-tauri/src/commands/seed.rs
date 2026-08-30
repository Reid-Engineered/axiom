use std::collections::HashMap;

use rusqlite::{params, Transaction};
use tauri::State;

use super::{
    database_error, workspace, CommandResult, Concept, Database, Goal, Material, MaterialResult,
    Module, SampleWorkspaceSeed, Session, Workspace,
};

fn insert_modules(transaction: &Transaction<'_>, modules: &[Module]) -> CommandResult<()> {
    for module in modules {
        transaction
            .execute(
                "INSERT INTO modules (
                    id, name, icon, trust, trust_detail, last_updated_label,
                    learner_count_label, developer, price, description, learning_value_detail,
                    context_seen, offline_status, enabled, visibility
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
                )",
                params![
                    module.id,
                    module.name,
                    module.icon,
                    module.trust,
                    module.trust_detail,
                    module.last_updated_label,
                    module.learner_count_label,
                    module.developer,
                    module.price,
                    module.description,
                    module.learning_value_detail,
                    module.context_seen,
                    module.offline_status,
                    module.enabled,
                    module.visibility,
                ],
            )
            .map_err(database_error)?;
    }

    for module in modules {
        for (position, concept_name) in module
            .supported_concept_names
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            transaction
                .execute(
                    "INSERT INTO module_supported_concepts (
                        module_id, position, concept_name
                    ) VALUES (?1, ?2, ?3)",
                    params![module.id, position as i64, concept_name],
                )
                .map_err(database_error)?;
        }
        for (position, dependency_id) in module
            .works_with_module_ids
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            transaction
                .execute(
                    "INSERT INTO module_dependencies (
                        module_id, position, works_with_module_id
                    ) VALUES (?1, ?2, ?3)",
                    params![module.id, position as i64, dependency_id],
                )
                .map_err(database_error)?;
        }
        for (position, description) in module
            .suits
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            transaction
                .execute(
                    "INSERT INTO module_suitability (
                        module_id, position, description
                    ) VALUES (?1, ?2, ?3)",
                    params![module.id, position as i64, description],
                )
                .map_err(database_error)?;
        }
        for (position, sentence) in module
            .privacy_notes
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            transaction
                .execute(
                    "INSERT INTO module_privacy_notes (
                        module_id, position, sentence
                    ) VALUES (?1, ?2, ?3)",
                    params![module.id, position as i64, sentence],
                )
                .map_err(database_error)?;
        }
    }

    Ok(())
}

fn insert_goals(transaction: &Transaction<'_>, goals: &[Goal]) -> CommandResult<()> {
    for goal in goals {
        transaction
            .execute(
                "INSERT INTO goals (
                    id, workspace_id, text, state, inferred_deadline, inferred_mastery_type,
                    inferred_concept_scope, inferred_pacing, previous_text, achieved_summary,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    goal.id,
                    goal.workspace_id,
                    goal.text,
                    goal.state,
                    goal.inferred.deadline,
                    goal.inferred.mastery_type,
                    goal.inferred.concept_scope,
                    goal.inferred.pacing,
                    goal.previous_text,
                    goal.achieved_summary,
                    goal.created_at,
                    goal.updated_at,
                ],
            )
            .map_err(database_error)?;

        for (position, tool) in goal
            .inferred
            .tools
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            transaction
                .execute(
                    "INSERT INTO goal_tools (goal_id, position, tool) VALUES (?1, ?2, ?3)",
                    params![goal.id, position as i64, tool],
                )
                .map_err(database_error)?;
        }
    }
    Ok(())
}

fn insert_concept_edges(
    transaction: &Transaction<'_>,
    source_id: &str,
    edge_kind: &str,
    target_ids: &[String],
) -> CommandResult<()> {
    for (position, target_id) in target_ids.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO concept_edges (
                    source_concept_id, edge_kind, position, target_concept_id
                ) VALUES (?1, ?2, ?3, ?4)",
                params![source_id, edge_kind, position as i64, target_id],
            )
            .map_err(database_error)?;
    }
    Ok(())
}

fn insert_concepts(transaction: &Transaction<'_>, concepts: &[Concept]) -> CommandResult<()> {
    for concept in concepts {
        // notes_count is deliberately omitted because SQLite's note triggers own this value.
        transaction
            .execute(
                "INSERT INTO concepts (
                    id, workspace_id, name, chapter, mastery_state, was_mastery_state,
                    decayed_at, meaning, due_for_review_in_days, on_exam, display_formula,
                    explanation, learner_heuristic, heuristic_evidence, last_activity_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
                )",
                params![
                    concept.id,
                    concept.workspace_id,
                    concept.name,
                    concept.chapter,
                    concept.mastery_state,
                    concept.was_mastery_state,
                    concept.decayed_at,
                    concept.meaning,
                    concept.due_for_review_in_days,
                    concept.on_exam,
                    concept.display_formula,
                    concept.explanation,
                    concept.learner_heuristic,
                    concept.heuristic_evidence,
                    concept.last_activity_at,
                ],
            )
            .map_err(database_error)?;
    }

    for concept in concepts {
        insert_concept_edges(
            transaction,
            &concept.id,
            "blocks",
            &concept.blocks_concept_ids,
        )?;
        insert_concept_edges(
            transaction,
            &concept.id,
            "prerequisite",
            &concept.prerequisite_concept_ids,
        )?;
        insert_concept_edges(
            transaction,
            &concept.id,
            "related",
            &concept.related_concept_ids,
        )?;
        insert_concept_edges(
            transaction,
            &concept.id,
            "leadsTo",
            &concept.leads_to_concept_ids,
        )?;

        for (position, description) in concept
            .where_it_shows_up
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            transaction
                .execute(
                    "INSERT INTO concept_where_it_shows_up (
                        concept_id, position, description
                    ) VALUES (?1, ?2, ?3)",
                    params![concept.id, position as i64, description],
                )
                .map_err(database_error)?;
        }
        for diagnostic in concept.recent_diagnostics.as_deref().unwrap_or_default() {
            transaction
                .execute(
                    "INSERT INTO concept_diagnostics (
                        id, concept_id, expression, diagnostic_type, note, occurred_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        diagnostic.id,
                        concept.id,
                        diagnostic.expression,
                        diagnostic.diagnostic_type,
                        diagnostic.note,
                        diagnostic.occurred_at,
                    ],
                )
                .map_err(database_error)?;
        }
    }

    Ok(())
}

fn insert_sessions(transaction: &Transaction<'_>, sessions: &[Session]) -> CommandResult<()> {
    for session in sessions {
        transaction
            .execute(
                "INSERT INTO sessions (
                    id, workspace_id, concept_id, status, intent_activity, intent_detail,
                    intent_target_minutes, resume_summary, thumbnail_url, elapsed_minutes,
                    problem_index, problem_count, open_question, started_at, paused_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
                )",
                params![
                    session.id,
                    session.workspace_id,
                    session.concept_id,
                    session.status,
                    session.intent.activity,
                    session.intent.detail,
                    session.intent.target_minutes,
                    session.resume_summary,
                    session.thumbnail_url,
                    session.elapsed_minutes,
                    session.problem_index,
                    session.problem_count,
                    session.open_question,
                    session.started_at,
                    session.paused_at,
                ],
            )
            .map_err(database_error)?;

        for (position, exchange) in session.exchanges.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO tutor_exchanges (
                        id, session_id, position, question, answer, occurred_at,
                        pinned_to_visualization
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        exchange.id,
                        session.id,
                        position as i64,
                        exchange.question,
                        exchange.answer,
                        exchange.occurred_at,
                        exchange.pinned_to_visualization,
                    ],
                )
                .map_err(database_error)?;
        }
        for (position, conclusion) in session.settled_conclusions.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO session_settled_conclusions (
                        session_id, position, conclusion
                    ) VALUES (?1, ?2, ?3)",
                    params![session.id, position as i64, conclusion],
                )
                .map_err(database_error)?;
        }
    }

    Ok(())
}

fn insert_material(transaction: &Transaction<'_>, material: &Material) -> CommandResult<()> {
    transaction
        .execute(
            "INSERT INTO materials (
                id, workspace_id, title, edition, total_pages, total_chapters,
                highlights_count, notes_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                material.id,
                material.workspace_id,
                material.title,
                material.edition,
                material.total_pages,
                material.total_chapters,
                material.highlights_count,
                material.notes_count,
            ],
        )
        .map_err(database_error)?;
    for (position, segment) in material.segments.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO material_chapter_segments (
                    material_id, position, label, status, detail
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    material.id,
                    position as i64,
                    segment.label,
                    segment.status,
                    segment.detail,
                ],
            )
            .map_err(database_error)?;
    }
    for (position, section) in material.most_marked_sections.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO material_most_marked_sections (
                    material_id, position, section
                ) VALUES (?1, ?2, ?3)",
                params![material.id, position as i64, section],
            )
            .map_err(database_error)?;
    }
    Ok(())
}

fn insert_material_result(
    transaction: &Transaction<'_>,
    result: &MaterialResult,
    material_id: &str,
) -> CommandResult<()> {
    transaction
        .execute(
            "INSERT INTO material_results (
                id, material_id, kind, page, title, reason, concept_id, in_syllabus,
                highlighted_at, exercise_total, exercise_attempted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                result.id,
                material_id,
                result.kind,
                result.page,
                result.title,
                result.reason,
                result.concept_id,
                result.in_syllabus,
                result.highlighted_at,
                result.exercise_total,
                result.exercise_attempted,
            ],
        )
        .map_err(database_error)?;
    Ok(())
}

fn import_seed(transaction: &Transaction<'_>, seed: &SampleWorkspaceSeed) -> CommandResult<()> {
    insert_modules(transaction, &seed.modules)?;
    for template in &seed.workspace_templates {
        transaction
            .execute(
                "INSERT INTO workspace_templates (id, name, description, tool_count)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    template.id,
                    template.name,
                    template.description,
                    template.tool_count,
                ],
            )
            .map_err(database_error)?;
    }

    for workspace in &seed.workspaces {
        transaction
            .execute(
                "INSERT INTO workspaces (
                    id, name, guiding_goal_id, progress, last_concept_name,
                    last_activity_at, paused
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    workspace.id,
                    workspace.name,
                    workspace.guiding_goal_id,
                    workspace.progress,
                    workspace.last_concept_name,
                    workspace.last_activity_at,
                    workspace.paused,
                ],
            )
            .map_err(database_error)?;
    }
    insert_goals(transaction, &seed.goals)?;

    for workspace in &seed.workspaces {
        for availability in &workspace.offline_availability {
            transaction
                .execute(
                    "INSERT INTO workspace_offline_availability (
                        workspace_id, kind, enabled, size_bytes, partial_available_count,
                        partial_total_count, partial_limit_reason
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        workspace.id,
                        availability.kind,
                        availability.enabled,
                        availability.size_bytes,
                        availability
                            .partial
                            .as_ref()
                            .map(|partial| partial.available_count),
                        availability
                            .partial
                            .as_ref()
                            .map(|partial| partial.total_count),
                        availability
                            .partial
                            .as_ref()
                            .map(|partial| partial.limit_reason.as_str()),
                    ],
                )
                .map_err(database_error)?;
        }
        for module_id in &workspace.enabled_module_ids {
            let module = seed
                .modules
                .iter()
                .find(|module| module.id == *module_id)
                .ok_or_else(|| format!("Sample module not found: {module_id}"))?;
            transaction
                .execute(
                    "INSERT INTO workspace_modules (
                        workspace_id, module_id, enabled, visibility
                    ) VALUES (?1, ?2, 1, ?3)",
                    params![workspace.id, module_id, module.visibility],
                )
                .map_err(database_error)?;
        }
    }

    for event in &seed.workspace_activity {
        transaction
            .execute(
                "INSERT INTO workspace_activity_events (
                    id, workspace_id, occurred_at, summary
                ) VALUES (?1, ?2, ?3, ?4)",
                params![
                    event.id,
                    event.workspace_id,
                    event.occurred_at,
                    event.summary
                ],
            )
            .map_err(database_error)?;
    }

    insert_concepts(transaction, &seed.concepts)?;
    insert_sessions(transaction, &seed.sessions)?;

    let concept_workspaces: HashMap<&str, &str> = seed
        .concepts
        .iter()
        .map(|concept| (concept.id.as_str(), concept.workspace_id.as_str()))
        .collect();
    let materials_by_workspace: HashMap<&str, &str> = seed
        .materials
        .iter()
        .map(|material| (material.workspace_id.as_str(), material.id.as_str()))
        .collect();
    for material in &seed.materials {
        insert_material(transaction, material)?;
    }
    for result in &seed.material_results {
        let workspace_id = concept_workspaces
            .get(result.concept_id.as_str())
            .ok_or_else(|| format!("Sample concept not found: {}", result.concept_id))?;
        let material_id = materials_by_workspace.get(workspace_id).ok_or_else(|| {
            format!(
                "Sample material not found for concept: {}",
                result.concept_id
            )
        })?;
        insert_material_result(transaction, result, material_id)?;
    }

    for note in &seed.notes {
        transaction
            .execute(
                "INSERT INTO notes (id, workspace_id, concept_id, text, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    note.id,
                    note.workspace_id,
                    note.concept_id,
                    note.text,
                    note.updated_at,
                ],
            )
            .map_err(database_error)?;
    }

    Ok(())
}

pub fn import_sample_workspace_handler(
    database: &Database,
    seed: &SampleWorkspaceSeed,
) -> CommandResult<Workspace> {
    if !seed
        .workspaces
        .iter()
        .any(|workspace| workspace.id == seed.sample_workspace_id)
    {
        return Err(format!(
            "Sample workspace not found in seed: {}",
            seed.sample_workspace_id
        ));
    }

    let mut connection = database.connection()?;
    if let Some(workspace) = workspace::load_workspace(&connection, &seed.sample_workspace_id)? {
        return Ok(workspace);
    }

    let transaction = connection.transaction().map_err(database_error)?;
    import_seed(&transaction, seed)?;
    transaction.commit().map_err(database_error)?;

    workspace::load_workspace(&connection, &seed.sample_workspace_id)?
        .ok_or_else(|| format!("Workspace not found: {}", seed.sample_workspace_id))
}

#[tauri::command(rename = "importSampleWorkspace")]
pub fn import_sample_workspace(
    database: State<'_, Database>,
    seed: SampleWorkspaceSeed,
) -> CommandResult<Workspace> {
    import_sample_workspace_handler(&database, &seed)
}
