use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationPlan {
    pub plan_id: String,
    pub source_type: String,
    pub source_id: String,
    pub target_repo_id: String,
    pub items: Vec<MigrationPlanItem>,
    pub conflicts: Vec<MigrationConflict>,
    pub missing_dependencies: Vec<ResourceDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationPlanItem {
    pub resource_id: String,
    pub action: String,
    pub source_path: Option<String>,
    pub target_path: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationConflict {
    pub resource_name: String,
    pub resource_type: String,
    pub reason: String,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceDependency {
    pub dep_type: String,
    pub name: String,
    pub required: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationRun {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub target_repo_id: String,
    pub status: String,
    pub plan_json: String,
    pub report_json: Option<String>,
    pub snapshot_path: Option<String>,
    pub created_at: String,
    pub executed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationReport {
    pub status: String,
    pub items: Vec<MigrationReportItem>,
    pub summary: MigrationReportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationReportItem {
    pub resource_name: String,
    pub action: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationReportSummary {
    pub added: u32,
    pub overwritten: u32,
    pub skipped: u32,
    pub failed: u32,
}
