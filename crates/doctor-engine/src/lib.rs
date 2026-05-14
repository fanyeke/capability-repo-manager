pub mod checks;
pub mod drift;
pub mod scoring;

use checks::HookConfig;
use domain::{DoctorIssue, DoctorReport};

use std::path::Path;

/// Run all diagnostic checks on a repository and produce a health report.
///
/// # Parameters
/// - `repo_id`: The repository identifier for the report
/// - `repo_path`: Path to the repository root on disk
/// - `hooks`: Pre-parsed hook configurations from settings.json
/// - `config_texts`: Raw text content of configuration files as (filename, content) pairs
/// - `mcp_json`: Raw MCP configuration JSON string
/// - `report_id`: UUID for the generated report
/// - `created_at`: ISO 8601 timestamp for when the report is generated
pub fn run_doctor(
    repo_id: &str,
    repo_path: &Path,
    hooks: &[HookConfig],
    config_texts: &[(String, String)],
    mcp_json: Option<&str>,
    report_id: &str,
    created_at: &str,
) -> DoctorReport {
    let mut all_issues: Vec<DoctorIssue> = Vec::new();

    all_issues.extend(checks::check_skill_structure(repo_path));
    all_issues.extend(checks::check_hook_targets(repo_path, hooks));
    all_issues.extend(checks::check_env_placeholders(config_texts));
    all_issues.extend(checks::check_mcp_config(mcp_json));

    let score = DoctorReport::compute_score(&all_issues);

    DoctorReport {
        id: report_id.to_string(),
        repo_id: repo_id.to_string(),
        score,
        issues: all_issues,
        created_at: created_at.to_string(),
    }
}
