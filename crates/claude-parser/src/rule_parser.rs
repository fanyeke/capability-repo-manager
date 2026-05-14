use domain::CapabilityResource;

pub fn parse_rules(repo_path: &str) -> Vec<CapabilityResource> {
    crate::parse_md_dir(repo_path, ".claude/rules", "rule", "project")
}
