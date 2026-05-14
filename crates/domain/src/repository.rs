use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
    pub current_branch: Option<String>,
    pub head_commit: Option<String>,
    pub dirty_state: String,
    pub last_indexed_at: String,
}

impl Repository {
    pub fn new(id: String, name: String, path: String) -> Self {
        Self {
            id,
            name,
            path,
            remote_url: None,
            current_branch: None,
            head_commit: None,
            dirty_state: "unknown".to_string(),
            last_indexed_at: String::new(),
        }
    }
}
