use std::path::PathBuf;

use crate::state::AppSettings;

/// Path to the settings JSON file.
fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".capability-repo-manager")
        .join("settings.json")
}

/// Load settings from `~/.capability-repo-manager/settings.json`.
/// Returns `None` if the file doesn't exist or can't be read.
pub fn load_from_file() -> Option<AppSettings> {
    let path = settings_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save settings to `~/.capability-repo-manager/settings.json`.
/// Creates parent directories if they don't exist.
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
