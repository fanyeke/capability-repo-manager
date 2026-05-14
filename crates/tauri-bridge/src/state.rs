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

impl AppState {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = Database::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;
        Ok(Self {
            db: Mutex::new(db),
            settings: Mutex::new(AppSettings::default()),
        })
    }
}
