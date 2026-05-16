use domain::paths::{AGENTS_DIR, CLAUDE_DIR};
use domain::CapabilityResource;

pub fn parse_agents(repo_path: &str) -> Vec<CapabilityResource> {
    crate::parse_md_dir(repo_path, &format!("{}/{}", CLAUDE_DIR, AGENTS_DIR), "agent", "project")
}
