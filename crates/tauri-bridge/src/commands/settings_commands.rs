use std::io::{Read, Write};
use std::path::PathBuf;

use tauri::State;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::state::{AppSettings, AppState};
use domain::redact_sensitive;

/// Path to the settings JSON file.
pub fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".capability-repo-manager")
        .join("settings.json")
}

/// Path to the log directory.
fn log_dir_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".capability-repo-manager")
        .join("logs")
}

/// Load settings from `~/.capability-repo-manager/settings.json`.
pub fn load_from_file() -> Option<AppSettings> {
    let path = settings_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save settings to `~/.capability-repo-manager/settings.json`.
pub fn save_to_file(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    Ok(())
}

/// Export a debug bundle (.zip) with logs, redacted settings, and operation summaries.
#[tauri::command]
pub fn export_debug_bundle(
    destination_path: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let file = std::fs::File::create(&destination_path)
        .map_err(|_| "无法写入目标路径，请选择其他目录".to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // app_info.json
    let app_info = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    zip.start_file("app_info.json", options)
        .map_err(|e| format!("Failed to start app_info.json in zip: {}", e))?;
    zip.write_all(serde_json::to_string_pretty(&app_info).unwrap().as_bytes())
        .map_err(|e| format!("Failed to write app_info.json: {}", e))?;

    // settings.json (redacted)
    let settings_content = std::fs::read_to_string(settings_path()).unwrap_or_default();
    let redacted_settings = redact_sensitive(&settings_content);
    zip.start_file("settings.json", options)
        .map_err(|e| format!("Failed to start settings.json in zip: {}", e))?;
    zip.write_all(redacted_settings.output.as_bytes())
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    // operation_events.json
    let event_store = storage::event_store::EventStore::new(db.conn());
    let events = event_store
        .list_events(100, 0, None)
        .map_err(|e| format!("Failed to query operation events: {}", e))?;
    let events_json = serde_json::to_string_pretty(&events)
        .map_err(|e| format!("Failed to serialize events: {}", e))?;
    zip.start_file("operation_events.json", options)
        .map_err(|e| format!("Failed to start operation_events.json in zip: {}", e))?;
    zip.write_all(events_json.as_bytes())
        .map_err(|e| format!("Failed to write operation_events.json: {}", e))?;

    // doctor_reports.json
    let reports = query_all_doctor_reports(db.conn());
    let reports_json = serde_json::to_string_pretty(&reports)
        .map_err(|e| format!("Failed to serialize doctor reports: {}", e))?;
    zip.start_file("doctor_reports.json", options)
        .map_err(|e| format!("Failed to start doctor_reports.json in zip: {}", e))?;
    zip.write_all(reports_json.as_bytes())
        .map_err(|e| format!("Failed to write doctor_reports.json: {}", e))?;

    // logs/ (last 7 days)
    let log_dir = log_dir_path();
    if log_dir.exists() {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
        if let Ok(entries) = std::fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() { continue; }
                if let Ok(metadata) = path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
                        if modified_time < cutoff { continue; }
                    }
                }
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let zip_path = format!("logs/{}", name);
                    if let Ok(mut f) = std::fs::File::open(&path) {
                        let mut buf = Vec::new();
                        if f.read_to_end(&mut buf).is_ok() {
                            let _ = zip.start_file(&zip_path, options);
                            let _ = zip.write_all(&buf);
                        }
                    }
                }
            }
        }
    }

    zip.finish().map_err(|e| format!("Failed to finalize zip: {}", e))?;
    tracing::info!(path = %destination_path, "debug_bundle_exported");
    Ok(())
}

fn query_all_doctor_reports(conn: &rusqlite::Connection) -> Vec<serde_json::Value> {
    let mut stmt = match conn.prepare(
        "SELECT id, repo_id, score, issues_json, created_at FROM doctor_reports ORDER BY created_at DESC LIMIT 50"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map([], |row| {
        let issues_json: String = row.get(3)?;
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "repo_id": row.get::<_, String>(1)?,
            "score": row.get::<_, i32>(2)?,
            "issues": serde_json::from_str::<serde_json::Value>(&issues_json).unwrap_or_default(),
            "created_at": row.get::<_, String>(4)?,
        }))
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    rows.filter_map(|r| r.ok()).collect()
}

/// Return the current log level from AppSettings.
#[tauri::command]
pub fn get_log_level(state: State<AppState>) -> Result<String, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.log_level.clone())
}

/// Set the log level at runtime and persist to settings.
#[tauri::command]
pub fn set_log_level(level: String, state: State<AppState>) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.log_level = level.clone();
        crate::commands::settings_commands::save_to_file(&settings)?;
    }
    crate::set_log_level(&level);
    Ok(())
}