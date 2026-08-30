use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

use super::{database_error, CommandResult, Database, Module, WorkspaceTemplate};

fn optional_text_list(
    connection: &Connection,
    sql: &str,
    module_id: &str,
) -> CommandResult<Option<Vec<String>>> {
    let mut statement = connection.prepare(sql).map_err(database_error)?;
    let values = statement
        .query_map([module_id], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok((!values.is_empty()).then_some(values))
}

fn workspace_exists(connection: &Connection, workspace_id: &str) -> CommandResult<bool> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM workspaces WHERE id = ?1)",
            [workspace_id],
            |row| row.get(0),
        )
        .map_err(database_error)
}

pub(crate) fn load_module(
    connection: &Connection,
    id: &str,
    workspace_id: Option<&str>,
) -> CommandResult<Option<Module>> {
    let module = connection
        .query_row(
            "SELECT id, name, icon, trust, trust_detail, last_updated_label,
                    learner_count_label, developer, price, description,
                    learning_value_detail, context_seen, offline_status, enabled, visibility
             FROM modules WHERE id = ?1",
            [id],
            |row| {
                Ok(Module {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    trust: row.get(3)?,
                    trust_detail: row.get(4)?,
                    last_updated_label: row.get(5)?,
                    learner_count_label: row.get(6)?,
                    developer: row.get(7)?,
                    price: row.get(8)?,
                    description: row.get(9)?,
                    learning_value_detail: row.get(10)?,
                    context_seen: row.get(11)?,
                    offline_status: row.get(12)?,
                    enabled: row.get(13)?,
                    visibility: row.get(14)?,
                    supported_concept_names: None,
                    works_with_module_ids: None,
                    suits: None,
                    privacy_notes: None,
                })
            },
        )
        .optional()
        .map_err(database_error)?;
    let Some(mut module) = module else {
        return Ok(None);
    };

    module.supported_concept_names = optional_text_list(
        connection,
        "SELECT concept_name FROM module_supported_concepts
         WHERE module_id = ?1 ORDER BY position",
        id,
    )?;
    module.works_with_module_ids = optional_text_list(
        connection,
        "SELECT works_with_module_id FROM module_dependencies
         WHERE module_id = ?1 ORDER BY position",
        id,
    )?;
    module.suits = optional_text_list(
        connection,
        "SELECT description FROM module_suitability
         WHERE module_id = ?1 ORDER BY position",
        id,
    )?;
    module.privacy_notes = optional_text_list(
        connection,
        "SELECT sentence FROM module_privacy_notes
         WHERE module_id = ?1 ORDER BY position",
        id,
    )?;

    if let Some(workspace_id) = workspace_id {
        let workspace_state = connection
            .query_row(
                "SELECT enabled, visibility FROM workspace_modules
                 WHERE workspace_id = ?1 AND module_id = ?2",
                params![workspace_id, id],
                |row| Ok((row.get::<_, bool>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(database_error)?;
        if let Some((enabled, visibility)) = workspace_state {
            module.enabled = enabled;
            module.visibility = visibility;
        } else {
            module.enabled = false;
        }
    }

    Ok(Some(module))
}

fn module_ids(connection: &Connection) -> CommandResult<Vec<String>> {
    let mut statement = connection
        .prepare("SELECT id FROM modules ORDER BY rowid")
        .map_err(database_error)?;
    let ids = statement
        .query_map([], |row| row.get(0))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok(ids)
}

pub fn get_modules_by_workspace_handler(
    database: &Database,
    workspace_id: &str,
) -> CommandResult<Vec<Module>> {
    let connection = database.connection()?;
    if !workspace_exists(&connection, workspace_id)? {
        return Err(format!("Workspace not found: {workspace_id}"));
    }
    module_ids(&connection)?
        .iter()
        .map(|id| load_module(&connection, id, Some(workspace_id)).map(Option::unwrap))
        .collect()
}

pub fn get_marketplace_modules_handler(
    database: &Database,
    for_workspace_id: Option<&str>,
) -> CommandResult<Vec<Module>> {
    if let Some(workspace_id) = for_workspace_id {
        return get_modules_by_workspace_handler(database, workspace_id);
    }
    let connection = database.connection()?;
    module_ids(&connection)?
        .iter()
        .map(|id| load_module(&connection, id, None).map(Option::unwrap))
        .collect()
}

pub fn get_workspace_templates_handler(
    database: &Database,
) -> CommandResult<Vec<WorkspaceTemplate>> {
    let connection = database.connection()?;
    let mut statement = connection
        .prepare(
            "SELECT id, name, description, tool_count
             FROM workspace_templates ORDER BY rowid",
        )
        .map_err(database_error)?;
    let templates = statement
        .query_map([], |row| {
            Ok(WorkspaceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                tool_count: row.get(3)?,
            })
        })
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    Ok(templates)
}

pub fn get_module_handler(database: &Database, id: &str) -> CommandResult<Module> {
    let connection = database.connection()?;
    load_module(&connection, id, None)?.ok_or_else(|| format!("Module not found: {id}"))
}

fn workspace_module_mutation(
    database: &Database,
    workspace_id: &str,
    module_id: &str,
    enabled: bool,
    visibility: Option<&str>,
) -> CommandResult<Module> {
    let connection = database.connection()?;
    if !workspace_exists(&connection, workspace_id)? {
        return Err(format!("Workspace not found: {workspace_id}"));
    }
    let module = load_module(&connection, module_id, None)?
        .ok_or_else(|| format!("Module not found: {module_id}"))?;
    let visibility_override = visibility;
    let visibility = visibility_override.unwrap_or(&module.visibility);
    connection
        .execute(
            "INSERT INTO workspace_modules (workspace_id, module_id, enabled, visibility)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(workspace_id, module_id) DO UPDATE SET
                 enabled = excluded.enabled,
                 visibility = CASE
                     WHEN ?5 IS NULL THEN workspace_modules.visibility
                     ELSE excluded.visibility
                 END",
            params![
                workspace_id,
                module_id,
                enabled,
                visibility,
                visibility_override
            ],
        )
        .map_err(database_error)?;
    load_module(&connection, module_id, Some(workspace_id))?
        .ok_or_else(|| format!("Module not found: {module_id}"))
}

pub fn install_module_handler(
    database: &Database,
    workspace_id: &str,
    module_id: &str,
) -> CommandResult<Module> {
    workspace_module_mutation(database, workspace_id, module_id, true, None)
}

pub fn set_module_enabled_handler(
    database: &Database,
    workspace_id: &str,
    module_id: &str,
    enabled: bool,
) -> CommandResult<Module> {
    workspace_module_mutation(database, workspace_id, module_id, enabled, None)
}

pub fn set_module_visibility_handler(
    database: &Database,
    workspace_id: &str,
    module_id: &str,
    visibility: &str,
) -> CommandResult<Module> {
    workspace_module_mutation(
        database,
        workspace_id,
        module_id,
        visibility != "off",
        Some(visibility),
    )
}

#[tauri::command(rename = "getModulesByWorkspace", rename_all = "camelCase")]
pub fn get_modules_by_workspace(
    database: State<'_, Database>,
    workspace_id: String,
) -> CommandResult<Vec<Module>> {
    get_modules_by_workspace_handler(&database, &workspace_id)
}

#[tauri::command(rename = "getMarketplaceModules", rename_all = "camelCase")]
pub fn get_marketplace_modules(
    database: State<'_, Database>,
    for_workspace_id: Option<String>,
) -> CommandResult<Vec<Module>> {
    get_marketplace_modules_handler(&database, for_workspace_id.as_deref())
}

#[tauri::command(rename = "getWorkspaceTemplates")]
pub fn get_workspace_templates(
    database: State<'_, Database>,
) -> CommandResult<Vec<WorkspaceTemplate>> {
    get_workspace_templates_handler(&database)
}

#[tauri::command(rename = "getModule")]
pub fn get_module(database: State<'_, Database>, id: String) -> CommandResult<Module> {
    get_module_handler(&database, &id)
}

#[tauri::command(rename = "installModule", rename_all = "camelCase")]
pub fn install_module(
    database: State<'_, Database>,
    workspace_id: String,
    module_id: String,
) -> CommandResult<Module> {
    install_module_handler(&database, &workspace_id, &module_id)
}

#[tauri::command(rename = "setModuleEnabled", rename_all = "camelCase")]
pub fn set_module_enabled(
    database: State<'_, Database>,
    workspace_id: String,
    module_id: String,
    enabled: bool,
) -> CommandResult<Module> {
    set_module_enabled_handler(&database, &workspace_id, &module_id, enabled)
}

#[tauri::command(rename = "setModuleVisibility", rename_all = "camelCase")]
pub fn set_module_visibility(
    database: State<'_, Database>,
    workspace_id: String,
    module_id: String,
    visibility: String,
) -> CommandResult<Module> {
    set_module_visibility_handler(&database, &workspace_id, &module_id, &visibility)
}
