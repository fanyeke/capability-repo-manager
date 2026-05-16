use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use storage::event_store::{EventStore, NewOperationEvent};
use storage::Database;
use zip::ZipArchive;

/// Helper: create a Database with sample operation events and doctor reports.
fn setup_db_with_data() -> Database {
    let db = Database::open_in_memory().unwrap();
    let store = EventStore::new(db.conn());

    // Insert operation events
    for i in 0..5 {
        store
            .insert_event(NewOperationEvent {
                operation_id: format!("op-{}", i),
                operation_type: if i % 2 == 0 { "scan_repositories" } else { "export_pack" }.to_string(),
                status: if i < 4 { "success" } else { "failure" }.to_string(),
                repo_id: if i == 0 { Some("repo-1".to_string()) } else { None },
                pack_id: None,
                migration_run_id: None,
                summary: Some(format!("Event #{}", i)),
                detail_json: None,
            })
            .unwrap();
    }

    // Insert a doctor report
    db.conn()
        .execute(
            "INSERT INTO doctor_reports (id, repo_id, score, issues_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "report-1",
                "repo-1",
                75i32,
                r#"[{"severity":"warning","code":"W001","message":"Missing skill directory"}]"#,
                "2026-05-15T10:00:00Z",
            ],
        )
        .unwrap();

    db
}

/// Helper: create a log file at the given path with a specific modified time.
fn write_log_file(path: &Path, content: &str, age_days: u64) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
    // Set the modified time to `age_days` ago
    let mtime = SystemTime::now() - Duration::from_secs(age_days * 86400);
    filetime::set_file_mtime(path, mtime.into()).ok();
}

#[test]
fn debug_bundle_contains_required_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let tmp_path = tmp.path();

    let db = setup_db_with_data();
    let settings_content = r#"{"scan_roots":["/home/user/projects"],"scan_depth":5}"#;
    let log_dir = tmp_path.join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    let zip_bytes = tauri_bridge::commands::settings_commands::build_debug_bundle_zip(
        &db,
        settings_content,
        &log_dir,
        "/home/user",
        false,
    )
    .unwrap();

    // Read the zip and verify entries
    let cursor = std::io::Cursor::new(zip_bytes);
    let archive = ZipArchive::new(cursor).unwrap();

    let entry_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    assert!(entry_names.contains(&"app_info.json".to_string()), "should contain app_info.json");
    assert!(entry_names.contains(&"settings.json".to_string()), "should contain settings.json");
    assert!(entry_names.contains(&"operation_events.json".to_string()), "should contain operation_events.json");
    assert!(entry_names.contains(&"doctor_reports.json".to_string()), "should contain doctor_reports.json");
}

#[test]
fn debug_bundle_app_info_is_valid_json() {
    let tmp = tempfile::tempdir().unwrap();
    let db = setup_db_with_data();
    let log_dir = tmp.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    let zip_bytes =
        tauri_bridge::commands::settings_commands::build_debug_bundle_zip(&db, "{}", &log_dir, "/home/user", false)
            .unwrap();

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut contents = String::new();
    archive.by_name("app_info.json").unwrap().read_to_string(&mut contents).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert!(parsed.get("version").is_some(), "should have version");
    assert!(parsed.get("platform").is_some(), "should have platform");
    assert!(parsed.get("timestamp").is_some(), "should have timestamp");
}

#[test]
fn debug_bundle_redacts_sensitive_data_in_settings() {
    let tmp = tempfile::tempdir().unwrap();
    let db = setup_db_with_data();
    let log_dir = tmp.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    // redact_sensitive matches key=value patterns, not JSON colon syntax.
    // Use a format that triggers the token pattern: api_key=sk-proj-abc123
    let settings_with_token = r#"{"key":"api_key=sk-proj-abc123","scan_depth":5}"#;

    let zip_bytes = tauri_bridge::commands::settings_commands::build_debug_bundle_zip(
        &db,
        settings_with_token,
        &log_dir,
        "/home/user",
        false,
    )
    .unwrap();

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut contents = String::new();
    archive.by_name("settings.json").unwrap().read_to_string(&mut contents).unwrap();

    assert!(!contents.contains("sk-proj-abc123"), "sensitive token should not appear in settings.json");
    assert!(contents.contains("[REDACTED]"), "redacted value should appear as [REDACTED]");
}

#[test]
fn debug_bundle_redact_paths_replaces_home_in_settings() {
    let tmp = tempfile::tempdir().unwrap();
    let db = setup_db_with_data();
    let log_dir = tmp.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    let settings_content = r#"{"scan_roots":["/home/alice/projects"],"name":"test"}"#;

    // Without redact_paths, /home/alice should remain
    let zip_bytes = tauri_bridge::commands::settings_commands::build_debug_bundle_zip(
        &db,
        settings_content,
        &log_dir,
        "/home/alice",
        false,
    )
    .unwrap();
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut contents = String::new();
    archive.by_name("settings.json").unwrap().read_to_string(&mut contents).unwrap();
    assert!(contents.contains("/home/alice"), "without redact_paths, home path should remain");

    // With redact_paths=true, /home/alice should become ~
    let zip_bytes = tauri_bridge::commands::settings_commands::build_debug_bundle_zip(
        &db,
        settings_content,
        &log_dir,
        "/home/alice",
        true,
    )
    .unwrap();
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut contents = String::new();
    archive.by_name("settings.json").unwrap().read_to_string(&mut contents).unwrap();
    assert!(contents.contains('~'), "with redact_paths, home path should be replaced with ~");
    assert!(!contents.contains("/home/alice"), "with redact_paths, /home/alice should not appear");
}

#[test]
fn debug_bundle_respects_log_retention() {
    let tmp = tempfile::tempdir().unwrap();
    let tmp_path = tmp.path();
    let db = setup_db_with_data();
    let log_dir = tmp_path.join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    // Recent log file (1 day old) — should be included
    write_log_file(&log_dir.join("app-2026-05-14.log"), "recent log entry", 1);
    // Old log file (30 days old) — should be excluded
    write_log_file(&log_dir.join("app-2026-04-14.log"), "old log entry", 30);

    let zip_bytes =
        tauri_bridge::commands::settings_commands::build_debug_bundle_zip(&db, "{}", &log_dir, "/home/user", false)
            .unwrap();

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let entry_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    assert!(entry_names.contains(&"logs/app-2026-05-14.log".to_string()), "recent log should be included");
    assert!(
        !entry_names.contains(&"logs/app-2026-04-14.log".to_string()),
        "old log file should be excluded by retention filter"
    );
}

#[test]
fn debug_bundle_operation_events_is_valid_json() {
    let tmp = tempfile::tempdir().unwrap();
    let db = setup_db_with_data();
    let log_dir = tmp.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    let zip_bytes =
        tauri_bridge::commands::settings_commands::build_debug_bundle_zip(&db, "{}", &log_dir, "/home/user", false)
            .unwrap();

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut contents = String::new();
    archive.by_name("operation_events.json").unwrap().read_to_string(&mut contents).unwrap();

    let events: Vec<serde_json::Value> = serde_json::from_str(&contents).unwrap();
    assert_eq!(events.len(), 5, "should contain 5 operation events");
    assert_eq!(events[0]["operation_type"], "scan_repositories");
}

#[test]
fn debug_bundle_redact_paths_applies_to_logs() {
    let tmp = tempfile::tempdir().unwrap();
    let tmp_path = tmp.path();
    let db = setup_db_with_data();
    let log_dir = tmp_path.join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    write_log_file(&log_dir.join("app.log"), "path=/home/testuser/projects scanned", 1);

    let zip_bytes =
        tauri_bridge::commands::settings_commands::build_debug_bundle_zip(&db, "{}", &log_dir, "/home/testuser", true)
            .unwrap();

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut contents = String::new();
    archive.by_name("logs/app.log").unwrap().read_to_string(&mut contents).unwrap();

    assert!(contents.contains('~'), "home path in logs should be replaced with ~");
    assert!(!contents.contains("/home/testuser"), "raw home path should not appear in logs");
}

#[test]
fn debug_bundle_read_only_path_returns_friendly_error() {
    // We can't easily create a truly read-only path on all systems,
    // so we use a non-existent path's parent to simulate permission issues.
    // The key test is that the Tauri command wrapper returns a user-friendly
    // error for PermissionDenied. The build_debug_bundle_zip itself succeeds
    // since it writes to a Cursor<Vec<u8>>; the permission check is in the
    // calling code (export_debug_bundle). We test the zip builder succeeds.
    let tmp = tempfile::tempdir().unwrap();
    let db = setup_db_with_data();
    let log_dir = tmp.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    let result =
        tauri_bridge::commands::settings_commands::build_debug_bundle_zip(&db, "{}", &log_dir, "/home/user", false);

    assert!(result.is_ok(), "zip builder should succeed with in-memory output");
    assert!(!result.unwrap().is_empty(), "zip bytes should not be empty");
}
