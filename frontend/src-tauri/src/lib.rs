use std::path::PathBuf;

fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".capability-repo-manager").join("data.db")
}

fn init_app_state() -> tauri_bridge::state::AppState {
    let db_path = default_db_path();

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    tauri_bridge::state::AppState::new(
        db_path.to_str().unwrap_or("data.db"),
    )
    .expect("Failed to initialize app state")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = init_app_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            tauri_bridge::commands::repo_commands::scan_repositories,
            tauri_bridge::commands::repo_commands::list_repositories,
            tauri_bridge::commands::repo_commands::refresh_repository,
            tauri_bridge::commands::repo_commands::refresh_all_repositories,
            tauri_bridge::commands::repo_commands::get_repository_detail,
            tauri_bridge::commands::repo_commands::remove_repository,
            tauri_bridge::commands::repo_commands::get_settings,
            tauri_bridge::commands::repo_commands::update_settings,
            tauri_bridge::commands::capability_commands::get_capability_inventory,
            tauri_bridge::commands::capability_commands::get_resource_detail,
            tauri_bridge::commands::pack_commands::export_capability_pack,
            tauri_bridge::commands::pack_commands::list_packs,
            tauri_bridge::commands::pack_commands::get_pack_detail,
            tauri_bridge::commands::pack_commands::delete_pack,
            tauri_bridge::commands::pack_commands::validate_pack,
            tauri_bridge::commands::migration_commands::build_migration_plan,
            tauri_bridge::commands::migration_commands::apply_migration_plan,
            tauri_bridge::commands::migration_commands::rollback_migration,
            tauri_bridge::commands::migration_commands::get_migration_history,
            tauri_bridge::commands::doctor_commands::run_doctor,
            tauri_bridge::commands::doctor_commands::get_latest_doctor_report,
            tauri_bridge::commands::doctor_commands::compare_repo_with_pack,
            tauri_bridge::commands::doctor_commands::compare_repos,
            tauri_bridge::commands::settings_commands::export_debug_bundle,
            tauri_bridge::commands::settings_commands::get_log_level,
            tauri_bridge::commands::settings_commands::set_log_level,
            tauri_bridge::commands::repo_commands::list_operation_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
