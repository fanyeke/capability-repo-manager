use domain::paths::{CLAUDE_DIR, PLUGINS_DIR};
use domain::CapabilityResource;

pub fn parse_plugins(repo_path: &str) -> Vec<CapabilityResource> {
    crate::parse_md_dir(
        repo_path,
        &format!("{}/{}", CLAUDE_DIR, PLUGINS_DIR),
        "plugin",
        "project",
    )
}
