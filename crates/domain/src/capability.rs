use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityResource {
    pub id: String,
    pub repo_id: Option<String>,
    pub pack_id: Option<String>,
    pub r#type: String,
    pub name: String,
    pub source_path: Option<String>,
    pub scope: String,
    pub tracked_by_git: bool,
    pub content_hash: Option<String>,
    pub metadata_json: Option<String>,
    pub error_message: Option<String>,
}
