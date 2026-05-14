use std::fs;
use std::path::Path;

use domain::CapabilityResource;
use sha2::{Digest, Sha256};

pub fn parse_mcp(repo_path: &str) -> Vec<CapabilityResource> {
    let mut resources = Vec::new();

    for (rel_path, scope) in &[
        (".claude/mcp.json", "project"),
        (".claude/mcp.local.json", "local"),
    ] {
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
            Ok(serde_json::Value::Array(servers)) => {
                for entry in servers {
                    resources.push(server_entry_to_resource(
                        &entry,
                        rel_path,
                        scope,
                        &file_hash,
                    ));
                }
            }
            Ok(_) => {
                // Not an array — produce error resource
                resources.push(CapabilityResource {
                    id: uuid::Uuid::new_v4().to_string(),
                    repo_id: None,
                    pack_id: None,
                    r#type: "mcp".to_string(),
                    name: "mcp-config".to_string(),
                    source_path: Some(rel_path.to_string()),
                    scope: scope.to_string(),
                    tracked_by_git: *scope == "project",
                    content_hash: Some(file_hash),
                    metadata_json: None,
                    error_message: Some(
                        "MCP config must be a JSON array".to_string(),
                    ),
                });
            }
            Err(e) => {
                resources.push(CapabilityResource {
                    id: uuid::Uuid::new_v4().to_string(),
                    repo_id: None,
                    pack_id: None,
                    r#type: "mcp".to_string(),
                    name: "mcp-config".to_string(),
                    source_path: Some(rel_path.to_string()),
                    scope: scope.to_string(),
                    tracked_by_git: *scope == "project",
                    content_hash: Some(file_hash),
                    metadata_json: None,
                    error_message: Some(format!(
                        "Failed to parse MCP config: {}",
                        e
                    )),
                });
            }
        }
    }

    resources
}

fn server_entry_to_resource(
    entry: &serde_json::Value,
    source_path: &str,
    scope: &str,
    file_hash: &str,
) -> CapabilityResource {
    let obj = entry.as_object();

    let name = obj
        .and_then(|o| o.get("serverName"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let command = obj
        .and_then(|o| o.get("command"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let args = obj
        .and_then(|o| o.get("args"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let env = obj
        .and_then(|o| o.get("env"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let error = if obj.is_none() {
        Some("MCP server entry is not a JSON object".to_string())
    } else {
        None
    };

    let metadata = serde_json::json!({
        "command": command,
        "args": args,
        "env": env,
    });

    CapabilityResource {
        id: uuid::Uuid::new_v4().to_string(),
        repo_id: None,
        pack_id: None,
        r#type: "mcp".to_string(),
        name,
        source_path: Some(source_path.to_string()),
        scope: scope.to_string(),
        tracked_by_git: scope == "project",
        content_hash: Some(file_hash.to_string()),
        metadata_json: Some(metadata.to_string()),
        error_message: error,
    }
}
