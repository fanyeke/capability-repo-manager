use std::sync::Mutex;

use storage::Database;

pub struct AppState {
    pub db: Mutex<Database>,
    pub settings: Mutex<AppSettings>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub scan_roots: Vec<String>,
    pub scan_depth: u32,
    pub ignore_patterns: Vec<String>,
    pub pack_storage_dir: String,
    pub file_watch_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            scan_roots: Vec::new(),
            scan_depth: 5,
            ignore_patterns: vec![
                "node_modules".to_string(),
                ".venv".to_string(),
                "vendor".to_string(),
                ".cache".to_string(),
                "build".to_string(),
            ],
            pack_storage_dir: default_pack_dir(),
            file_watch_enabled: false,
        }
    }
}

fn default_pack_dir() -> String {
    std::env::var("HOME")
        .map(|h| format!("{}/.capability-repo-manager/packs", h))
        .unwrap_or_else(|_| ".".into())
}

/// Load settings from `~/.capability-repo-manager/settings.json`.
/// Returns `None` if the file doesn't exist or can't be read.
fn load_settings_from_file() -> Option<AppSettings> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let path = std::path::PathBuf::from(home)
        .join(".capability-repo-manager")
        .join("settings.json");
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = Database::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;

        // Load persisted settings, fall back to defaults
        let settings = load_settings_from_file().unwrap_or_default();

        Ok(Self {
            db: Mutex::new(db),
            settings: Mutex::new(settings),
        })
    }
}
