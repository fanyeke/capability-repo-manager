use std::fs;

fn setup_temp_repo(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel_path, content) in files {
        let full_path = dir.path().join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
    }
    dir
}

#[test]
fn parses_mcp_json_with_single_server() {
    let mcp_config = r#"[
        {
            "name": "filesystem",
            "command": "npx",
            "args": ["-y", "@anthropic/mcp-filesystem"],
            "env": { "HOME": "/tmp" }
        }
    ]"#;
    let dir = setup_temp_repo(&[(".claude/mcp.json", mcp_config)]);

    let resources = claude_parser::mcp_parser::parse_mcp(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    let mcp = &resources[0];
    assert_eq!(mcp.r#type, "mcp");
    assert_eq!(mcp.name, "filesystem");
    assert_eq!(mcp.scope, "project");
    assert!(mcp.source_path.as_ref().unwrap().contains(".claude/mcp.json"));

    let meta: serde_json::Value = serde_json::from_str(mcp.metadata_json.as_ref().unwrap()).unwrap();
    assert_eq!(meta["command"], "npx");
    assert_eq!(meta["args"][0], "-y");
    assert_eq!(meta["env"]["HOME"], "/tmp");
}

#[test]
fn parses_mcp_json_with_multiple_servers() {
    let mcp_config = r#"[
        { "name": "server-a", "command": "python", "args": ["-m", "a"] },
        { "name": "server-b", "command": "node", "args": ["b.js"] }
    ]"#;
    let dir = setup_temp_repo(&[(".claude/mcp.json", mcp_config)]);

    let resources = claude_parser::mcp_parser::parse_mcp(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 2);
    let names: Vec<&str> = resources.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"server-a"));
    assert!(names.contains(&"server-b"));
}

#[test]
fn mcp_local_json_has_local_scope() {
    let mcp_config = r#"[{"name": "local-server", "command": "echo"}]"#;
    let dir = setup_temp_repo(&[(".claude/mcp.local.json", mcp_config)]);

    let resources = claude_parser::mcp_parser::parse_mcp(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].scope, "local");
}

#[test]
fn mcp_json_with_empty_array_returns_empty() {
    let dir = setup_temp_repo(&[(".claude/mcp.json", "[]")]);

    let resources = claude_parser::mcp_parser::parse_mcp(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 0);
}

#[test]
fn no_mcp_config_returns_empty() {
    let dir = setup_temp_repo(&[("README.md", "# No MCP here")]);

    let resources = claude_parser::mcp_parser::parse_mcp(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 0);
}

#[test]
fn mcp_server_without_name_gets_unknown_name() {
    let mcp_config = r#"[{"command": "npx", "args": ["test"]}]"#;
    let dir = setup_temp_repo(&[(".claude/mcp.json", mcp_config)]);

    let resources = claude_parser::mcp_parser::parse_mcp(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "unknown");
}

#[test]
fn generates_content_hash_for_mcp_config() {
    let mcp_config = r#"[{"name": "hash-test", "command": "echo"}]"#;
    let dir = setup_temp_repo(&[(".claude/mcp.json", mcp_config)]);

    let resources = claude_parser::mcp_parser::parse_mcp(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    assert!(resources[0].content_hash.is_some());
    assert_eq!(resources[0].content_hash.as_ref().unwrap().len(), 64);
}
