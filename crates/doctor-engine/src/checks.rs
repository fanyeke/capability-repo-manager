use domain::DoctorIssue;
use std::fs;
use std::path::Path;

/// Represents a parsed hook configuration entry from settings.json
#[derive(Debug, Clone)]
pub struct HookConfig {
    pub name: String,
    pub hook_type: String,
    pub command: Option<String>,
    pub script_path: Option<String>,
}

/// Check skill directory structure for completeness.
///
/// Walks `.claude/skills/` and verifies each skill subdirectory contains
/// either `SKILL.md` or `skill.md`. Reports warnings for incomplete skills.
pub fn check_skill_structure(repo_path: &Path) -> Vec<DoctorIssue> {
    let skills_dir = repo_path.join(".claude").join("skills");
    if !skills_dir.exists() {
        return vec![];
    }

    let mut issues = Vec::new();
    let entries = match fs::read_dir(&skills_dir) {
        Ok(e) => e,
        Err(_) => return issues,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let has_skill_md = path.join("SKILL.md").exists() || path.join("skill.md").exists();

        if !has_skill_md {
            issues.push(DoctorIssue {
                severity: "warning".to_string(),
                code: "SKILL_STRUCTURE_INCOMPLETE".to_string(),
                message: format!(
                    "Skill '{}' is missing SKILL.md (or skill.md). The skill directory should contain a markdown file defining the skill.",
                    skill_name
                ),
                resource_ref: None,
                recommendation: Some(format!(
                    "Create {}/SKILL.md with the skill definition",
                    path.strip_prefix(repo_path).unwrap_or(&path).display()
                )),
            });
        }
    }

    issues
}

/// Check hook target existence on disk.
///
/// Examines hook commands and scripts to verify referenced files exist.
/// Reports critical issues for missing hook targets since hooks with broken
/// commands will fail at runtime.
pub fn check_hook_targets(repo_path: &Path, hooks: &[HookConfig]) -> Vec<DoctorIssue> {
    let mut issues = Vec::new();

    for hook in hooks {
        let target = match (&hook.command, &hook.script_path) {
            (Some(cmd), _) => extract_script_path(cmd),
            (_, Some(script)) => Some(script.clone()),
            (None, None) => continue,
        };

        if let Some(script_path) = target {
            let full_path = repo_path.join(&script_path);
            if !full_path.exists() {
                issues.push(DoctorIssue {
                    severity: "critical".to_string(),
                    code: "HOOK_TARGET_MISSING".to_string(),
                    message: format!(
                        "Hook '{}' ({}) references '{}' which does not exist on disk",
                        hook.name, hook.hook_type, script_path
                    ),
                    resource_ref: None,
                    recommendation: Some(format!(
                        "Verify the path '{}' is correct and the file exists, or update the hook command",
                        script_path
                    )),
                });
            }
        }
    }

    issues
}

/// Extract a script/executable path from a command string.
/// Returns the first path-like argument that references a file.
fn extract_script_path(command: &str) -> Option<String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    // Skip the first part if it looks like a program name (bash, node, python, etc.)
    let candidates: Vec<&str> = parts
        .iter()
        .filter(|p| {
            !matches!(**p, "bash" | "sh" | "zsh" | "node" | "python" | "python3" | "ruby" | "perl" | "php")
                && !p.starts_with('-')
                && !p.starts_with("--")
                && !p.starts_with('$')
                && !p.starts_with("${")
        })
        .copied()
        .collect();

    candidates.first().map(|p| p.to_string())
}

/// Check for unresolved environment variable placeholders in configuration files.
///
/// Scans config text for `${VAR}` patterns. Each unresolved variable is a warning
/// since it may fail at runtime if the env var is not set.
pub fn check_env_placeholders(configs: &[(String, String)]) -> Vec<DoctorIssue> {
    let mut issues = Vec::new();

    for (filename, content) in configs {
        let vars = find_env_placeholders(content);
        for var in vars {
            issues.push(DoctorIssue {
                severity: "warning".to_string(),
                code: "ENV_PLACEHOLDER_UNRESOLVED".to_string(),
                message: format!(
                    "Environment variable placeholder ${{{}}} found in {} — ensure this variable is set in the environment",
                    var, filename
                ),
                resource_ref: None,
                recommendation: Some(format!(
                    "Set the {} environment variable in your shell profile, or configure it in .claude/settings.local.json",
                    var
                )),
            });
        }
    }

    issues
}

/// Find all ${VAR} patterns in text content.
fn find_env_placeholders(text: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len().saturating_sub(1) {
        if chars[i] == '$' && chars.get(i + 1) == Some(&'{') {
            let start = i + 2;
            if let Some(end) = chars[start..].iter().position(|&c| c == '}') {
                let var = chars[start..start + end].iter().collect::<String>();
                if !var.is_empty() {
                    vars.push(var);
                }
                i = start + end + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    vars
}

/// Check MCP configuration for structural integrity.
///
/// Validates the mcp.json format and ensures each server entry has a name.
pub fn check_mcp_config(mcp_json: Option<&str>) -> Vec<DoctorIssue> {
    let mut issues = Vec::new();

    let json_str = match mcp_json {
        Some(s) if s.trim().is_empty() => return issues,
        Some(s) => s,
        None => return issues,
    };

    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            issues.push(DoctorIssue {
                severity: "critical".to_string(),
                code: "MCP_CONFIG_PARSE_ERROR".to_string(),
                message: format!("MCP configuration file (mcp.json) contains invalid JSON: {}", e),
                resource_ref: None,
                recommendation: Some("Validate the JSON syntax of .claude/mcp.json using a JSON validator".to_string()),
            });
            return issues;
        }
    };

    let servers = match parsed {
        serde_json::Value::Array(arr) => arr,
        _ => {
            issues.push(DoctorIssue {
                severity: "critical".to_string(),
                code: "MCP_CONFIG_INVALID_STRUCTURE".to_string(),
                message: "MCP configuration should be a JSON array of server objects".to_string(),
                resource_ref: None,
                recommendation: Some(
                    "MCP config should be an array like: [{\"name\": \"server-name\", \"command\": \"...\"}]"
                        .to_string(),
                ),
            });
            return issues;
        }
    };

    for (i, server) in servers.iter().enumerate() {
        if let Some(obj) = server.as_object() {
            if !obj.contains_key("name") {
                issues.push(DoctorIssue {
                    severity: "critical".to_string(),
                    code: "MCP_CONFIG_MISSING_NAME".to_string(),
                    message: format!("MCP server at index {} is missing the required 'name' field", i),
                    resource_ref: None,
                    recommendation: Some(format!(
                        "Add a 'name' field to the MCP server at index {} in .claude/mcp.json",
                        i
                    )),
                });
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_script_path_simple() {
        assert_eq!(extract_script_path("bash scripts/deploy.sh"), Some("scripts/deploy.sh".to_string()));
    }

    #[test]
    fn extract_script_path_node() {
        assert_eq!(extract_script_path("node server.js --flag"), Some("server.js".to_string()));
    }

    #[test]
    fn extract_script_path_no_script() {
        assert_eq!(extract_script_path("bash"), None);
    }

    #[test]
    fn find_env_placeholders_multiple() {
        let text = r#"{"key": "${SECRET}", "url": "${API_URL}"}"#;
        let vars = find_env_placeholders(text);
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"SECRET".to_string()));
        assert!(vars.contains(&"API_URL".to_string()));
    }

    #[test]
    fn find_env_placeholders_none() {
        let text = r#"{"key": "plain-value"}"#;
        let vars = find_env_placeholders(text);
        assert!(vars.is_empty());
    }
}
