use domain::paths::{CLAUDE_DIR, RULES_DIR};
use domain::CapabilityResource;

pub fn parse_rules(repo_path: &str) -> Vec<CapabilityResource> {
    crate::parse_md_dir(repo_path, &format!("{}/{}", CLAUDE_DIR, RULES_DIR), "rule", "project")
}
