use domain::CapabilityResource;

pub fn parse_agents(repo_path: &str) -> Vec<CapabilityResource> {
    crate::parse_md_dir(repo_path, ".claude/agents", "agent", "project")
}
