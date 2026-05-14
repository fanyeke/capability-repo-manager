use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityPack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub pack_type: String,
    pub manifest_path: String,
    pub source_repo_id: Option<String>,
    pub source_commit: Option<String>,
    pub created_at: String,
    pub storage_dir: String,
}
