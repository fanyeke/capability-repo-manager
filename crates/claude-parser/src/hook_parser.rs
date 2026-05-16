use std::fs;
use std::path::Path;

use domain::paths::{CLAUDE_DIR, SETTINGS_JSON};
use domain::CapabilityResource;
use sha2::{Digest, Sha256};

pub fn parse_hooks(repo_path: &str) -> Vec<CapabilityResource> {
    let mut resources = Vec::new();

    let settings_paths = [
        (format!("{}/{}", CLAUDE_DIR, SETTINGS_JSON), "project"),
        (format!("{}/settings.local.json", CLAUDE_DIR), "local"),
    ];
    for (rel_path, scope) in &settings_paths {
        let file_path = Path::new(repo_path).join(rel_path);
        if !file_path.exists() {
            continue;
        }

        let raw = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_hash = format!("{:x}", Sha256::digest(raw.as_bytes()));

        match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(config) => {
                if let Some(hooks) = config.get("hooks") {
                    if let Some(hooks_obj) = hooks.as_object() {
                        for (hook_type, hook_list) in hooks_obj {
                            if let Some(hooks_array) = hook_list.as_array() {
                                for entry in hooks_array {
                                    resources
                                        .push(hook_entry_to_resource(entry, hook_type, rel_path, scope, &file_hash));
                                }
                            }
                        }
                    }
                }
                // Also emit a settings resource for this file
                if *scope == "project" {
                    let prefix = format!("{}/", CLAUDE_DIR);
                    let name = rel_path.trim_start_matches(&prefix).trim_end_matches(".json").to_string();
                    resources.push(CapabilityResource {
                        id: uuid::Uuid::new_v4().to_string(),
                        repo_id: None,
                        pack_id: None,
                        r#type: "settings".to_string(),
                        name,
                        source_path: Some(rel_path.to_string()),
                        scope: scope.to_string(),
                        tracked_by_git: *scope == "project",
                        content_hash: Some(file_hash.clone()),
                        metadata_json: None,
                        error_message: None,
                    });
                }
            }
            Err(e) => {
                resources.push(CapabilityResource {
                    id: uuid::Uuid::new_v4().to_string(),
                    repo_id: None,
                    pack_id: None,
                    r#type: "hook".to_string(),
                    name: "settings-config".to_string(),
                    source_path: Some(rel_path.to_string()),
                    scope: scope.to_string(),
                    tracked_by_git: *scope == "project",
                    content_hash: Some(file_hash),
                    metadata_json: None,
                    error_message: Some(format!("Failed to parse settings: {}", e)),
                });
            }
        }
    }

    resources
}

fn hook_entry_to_resource(
    entry: &serde_json::Value,
    hook_type: &str,
    source_path: &str,
    scope: &str,
    file_hash: &str,
) -> CapabilityResource {
    let obj = entry.as_object();

    let name = obj
        .and_then(|o| o.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            obj.and_then(|o| o.get("command")).and_then(|v| v.as_str()).map(extract_command_name).unwrap_or("unknown")
        })
        .to_string();

    let command = obj.and_then(|o| o.get("command")).cloned().unwrap_or(serde_json::Value::Null);

    let metadata = serde_json::json!({
        "hook_type": hook_type,
        "command": command,
    });

    CapabilityResource {
        id: uuid::Uuid::new_v4().to_string(),
        repo_id: None,
        pack_id: None,
        r#type: "hook".to_string(),
        name,
        source_path: Some(source_path.to_string()),
        scope: scope.to_string(),
        tracked_by_git: scope == "project",
        content_hash: Some(file_hash.to_string()),
        metadata_json: Some(metadata.to_string()),
        error_message: None,
    }
}

fn extract_command_name(cmd: &str) -> &str {
    Path::new(cmd).file_name().and_then(|n| n.to_str()).unwrap_or(cmd)
}
