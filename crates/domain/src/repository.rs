use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub path: String,
    pub canonical_path: String,
    pub remote_url: Option<String>,
    pub current_branch: Option<String>,
    pub head_commit: Option<String>,
    pub dirty_state: String,
    pub first_indexed_at: String,
    pub last_indexed_at: String,
    pub capability_index_status: String,
    pub last_capability_indexed_at: Option<String>,
    pub last_capability_error: Option<String>,
}

impl Repository {
    pub fn new(id: String, name: String, path: String) -> Self {
        let canonical_path = path.clone();
        Self {
            id,
            name,
            path,
            canonical_path,
            remote_url: None,
            current_branch: None,
            head_commit: None,
            dirty_state: "unknown".to_string(),
            first_indexed_at: String::new(),
            last_indexed_at: String::new(),
            capability_index_status: "never_indexed".to_string(),
            last_capability_indexed_at: None,
            last_capability_error: None,
        }
    }
}
