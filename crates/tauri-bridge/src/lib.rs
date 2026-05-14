// Tauri Bridge - Glue layer between Tauri commands and domain engines.
//
// This crate contains Tauri command handlers that delegate to:
// - repo-scanner (scan_repositories, list_repositories, refresh_repository)
// - claude-parser (get_capability_inventory, get_resource_detail)
// - pack-engine (export_capability_pack, list_packs, validate_pack)
// - migration-engine (build_migration_plan, apply_migration_plan, rollback_migration)
// - doctor-engine (run_doctor, compare_repo_with_pack, compare_repos)

pub mod commands;
pub mod state;
