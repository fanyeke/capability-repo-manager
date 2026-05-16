pub mod agent_parser;
pub mod commands_parser;
pub mod hook_parser;
pub mod mcp_parser;
pub mod rule_parser;
pub mod skill_parser;

use std::fs;
use std::path::Path;

use domain::CapabilityResource;
use sha2::{Digest, Sha256};

pub struct CapabilityInventory {
    pub skills: Vec<CapabilityResource>,
    pub mcp: Vec<CapabilityResource>,
    pub hooks: Vec<CapabilityResource>,
    pub rules: Vec<CapabilityResource>,
    pub agents: Vec<CapabilityResource>,
    pub commands: Vec<CapabilityResource>,
    pub plugins: Vec<CapabilityResource>,
    pub settings: Vec<CapabilityResource>,
}

pub fn parse_repo(repo_path: &str) -> Result<CapabilityInventory, domain::AppError> {
    Ok(CapabilityInventory {
        skills: skill_parser::parse_skills(repo_path),
        mcp: mcp_parser::parse_mcp(repo_path),
        hooks: hook_parser::parse_hooks(repo_path),
        rules: rule_parser::parse_rules(repo_path),
        agents: agent_parser::parse_agents(repo_path),
        commands: commands_parser::parse_commands(repo_path),
        plugins: Vec::new(),  // TODO: implement plugins_parser
        settings: Vec::new(), // TODO: implement settings_parser
    })
}

/// Parse a directory of markdown files into capability resources.
/// Used by rule_parser and agent_parser.
pub(crate) fn parse_md_dir(
    repo_path: &str,
    relative_dir: &str,
    resource_type: &str,
    scope: &str,
) -> Vec<CapabilityResource> {
    let dir_path = Path::new(repo_path).join(relative_dir);
    if !dir_path.exists() || !dir_path.is_dir() {
        return Vec::new();
    }

    let mut resources = Vec::new();
    let entries = match fs::read_dir(&dir_path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" {
            continue;
        }

        let name = path.file_stem().unwrap().to_string_lossy().to_string();

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        let source_path = format!("{}/{}.md", relative_dir, name);

        resources.push(CapabilityResource {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id: None,
            pack_id: None,
            r#type: resource_type.to_string(),
            name,
            source_path: Some(source_path),
            scope: scope.to_string(),
            tracked_by_git: true,
            content_hash: Some(hash),
            metadata_json: None,
            error_message: None,
        });
    }

    resources
}
