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
fn malformed_mcp_json_produces_parse_error_resource() {
    let dir = setup_temp_repo(&[(".claude/mcp.json", "not valid json {{{")]);

    let resources = claude_parser::mcp_parser::parse_mcp(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 1);
    let err = &resources[0];
    assert_eq!(err.r#type, "mcp");
    assert!(err.error_message.is_some());
    assert!(err.error_message.as_ref().unwrap().contains("parse"));
}

#[test]
fn malformed_settings_json_produces_parse_error() {
    let dir = setup_temp_repo(&[(".claude/settings.json", "{broken")]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 1);
    assert!(resources[0].error_message.is_some());
}

#[test]
fn mcp_json_with_invalid_server_entry_handled() {
    let mcp_config = r#"[
        {"serverName": "good", "command": "echo"},
        "not an object",
        {"serverName": "also-good", "command": "ls"}
    ]"#;
    let dir = setup_temp_repo(&[(".claude/mcp.json", mcp_config)]);

    let resources = claude_parser::mcp_parser::parse_mcp(
        dir.path().to_str().unwrap(),
    );

    // Should have 2 good servers and 1 error resource
    assert_eq!(resources.len(), 3);
    let good_count = resources.iter().filter(|r| r.error_message.is_none()).count();
    let err_count = resources.iter().filter(|r| r.error_message.is_some()).count();
    assert_eq!(good_count, 2);
    assert_eq!(err_count, 1);
}

#[test]
fn missing_skills_dir_handled_gracefully() {
    let dir = setup_temp_repo(&[("README.md", "# No skills here")]);

    let resources = claude_parser::skill_parser::parse_skills(
        dir.path().to_str().unwrap(),
    );

    // Should not crash, return empty
    assert_eq!(resources.len(), 0);
}

#[test]
fn skill_dir_without_md_file_is_skipped() {
    let dir = setup_temp_repo(&[("skills/incomplete/.gitkeep", "")]);

    let resources = claude_parser::skill_parser::parse_skills(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 0);
}

#[test]
fn hooks_with_null_command_handled() {
    let settings = r#"{
        "hooks": {
            "PreToolUse": [
                {"name": "broken-hook"}
            ]
        }
    }"#;
    let dir = setup_temp_repo(&[(".claude/settings.json", settings)]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    // Should still create a resource, but with no command in metadata
    assert_eq!(resources.len(), 1);
    let meta: serde_json::Value =
        serde_json::from_str(resources[0].metadata_json.as_ref().unwrap()).unwrap();
    assert!(meta["command"].is_null());
}

#[test]
fn empty_skill_md_still_detected() {
    let dir = setup_temp_repo(&[("skills/empty-skill/SKILL.md", "")]);

    let resources = claude_parser::skill_parser::parse_skills(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "empty-skill");
    assert_eq!(resources[0].error_message, None);
}
