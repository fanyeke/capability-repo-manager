use domain::paths::{CLAUDE_DIR, COMMANDS_DIR};
use domain::CapabilityResource;

pub fn parse_commands(repo_path: &str) -> Vec<CapabilityResource> {
    crate::parse_md_dir(repo_path, &format!("{}/{}", CLAUDE_DIR, COMMANDS_DIR), "command", "project")
}
