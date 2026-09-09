pub mod capabilities;
pub mod commands;
pub mod db;
pub mod generation;
pub mod knowledge;
pub mod modules;
pub mod practice;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("axiom.sqlite3");
            app.manage(commands::Database::open(&database_path)?);

            let knowledge_package_dir = app.path().resource_dir()?.join("knowledge-package");
            let knowledge_package = knowledge::load_knowledge_package(&knowledge_package_dir)
                .expect(
                    "bundled knowledge-package must load -- a broken bundle is a build problem",
                );
            let practice_connection = db::open(&database_path)?;
            let (registry, installation) =
                commands::practice::build_practice_registry(knowledge_package, practice_connection);
            app.manage(registry);
            app.manage(installation);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::get_workspaces,
            commands::workspace::get_workspace,
            commands::workspace::get_recent_activity,
            commands::workspace::create_workspace,
            commands::workspace::set_workspace_offline_availability,
            commands::goal::get_goal,
            commands::goal::get_goals_by_workspace,
            commands::goal::update_goal,
            commands::goal::revert_goal,
            commands::concept::get_concepts_by_workspace,
            commands::concept::get_concept,
            commands::concept::search_concepts,
            commands::module::get_modules_by_workspace,
            commands::module::get_marketplace_modules,
            commands::module::get_workspace_templates,
            commands::module::get_module,
            commands::module::install_module,
            commands::module::set_module_enabled,
            commands::module::set_module_visibility,
            commands::session::get_active_session_by_workspace,
            commands::session::get_session,
            commands::session::start_session,
            commands::session::pause_session,
            commands::session::resume_session,
            commands::session::add_tutor_exchange,
            commands::session::end_session,
            commands::material::get_material,
            commands::material::search_material,
            commands::note::get_recent_notes,
            commands::seed::import_sample_workspace,
            commands::practice::generate_attempt,
            commands::practice::evaluate_attempt,
            commands::practice::request_hint,
            commands::practice::describe_attempt,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
