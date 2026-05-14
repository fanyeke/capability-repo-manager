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
fn parses_hooks_from_settings_json() {
    let settings = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "name": "tmux-reminder",
                    "command": "echo 'long running'"
                }
            ],
            "PostToolUse": [
                {
                    "name": "format-check",
                    "command": "prettier --check"
                }
            ]
        }
    }"#;
    let dir = setup_temp_repo(&[(".claude/settings.json", settings)]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 2);
    let types: Vec<&str> = resources.iter().map(|r| r.r#type.as_str()).collect();
    assert!(types.iter().all(|t| *t == "hook"));

    let names: Vec<&str> = resources.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"tmux-reminder"));
    assert!(names.contains(&"format-check"));

    // Verify hook type metadata
    let tmux = resources.iter().find(|r| r.name == "tmux-reminder").unwrap();
    let meta: serde_json::Value =
        serde_json::from_str(tmux.metadata_json.as_ref().unwrap()).unwrap();
    assert_eq!(meta["hook_type"], "PreToolUse");
    assert_eq!(meta["command"], "echo 'long running'");
}

#[test]
fn settings_local_json_has_local_scope() {
    let settings = r#"{
        "hooks": {
            "Stop": [{"name": "cleanup", "command": "rm -rf /tmp/build"}]
        }
    }"#;
    let dir = setup_temp_repo(&[(".claude/settings.local.json", settings)]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].scope, "local");
}

#[test]
fn settings_without_hooks_key_returns_empty() {
    let settings = r#"{"permissions": {"allow": ["read"]}}"#;
    let dir = setup_temp_repo(&[(".claude/settings.json", settings)]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 0);
}

#[test]
fn no_settings_file_returns_empty() {
    let dir = setup_temp_repo(&[("README.md", "# No settings")]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 0);
}

#[test]
fn hook_with_multiple_hook_types() {
    let settings = r#"{
        "hooks": {
            "PreToolUse": [
                {"name": "pre-hook-1", "command": "echo 1"},
                {"name": "pre-hook-2", "command": "echo 2"}
            ],
            "PostToolUse": [
                {"name": "post-hook-1", "command": "echo 3"}
            ],
            "Stop": [
                {"name": "stop-hook", "command": "echo stop"}
            ]
        }
    }"#;
    let dir = setup_temp_repo(&[(".claude/settings.json", settings)]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 4);

    let pre1 = resources.iter().find(|r| r.name == "pre-hook-1").unwrap();
    let meta1: serde_json::Value =
        serde_json::from_str(pre1.metadata_json.as_ref().unwrap()).unwrap();
    assert_eq!(meta1["hook_type"], "PreToolUse");

    let stop = resources.iter().find(|r| r.name == "stop-hook").unwrap();
    let meta_s: serde_json::Value =
        serde_json::from_str(stop.metadata_json.as_ref().unwrap()).unwrap();
    assert_eq!(meta_s["hook_type"], "Stop");
}

#[test]
fn hook_without_name_uses_command_as_name() {
    let settings = r#"{
        "hooks": {
            "PreToolUse": [{"command": "/usr/bin/mytool"}]
        }
    }"#;
    let dir = setup_temp_repo(&[(".claude/settings.json", settings)]);

    let resources = claude_parser::hook_parser::parse_hooks(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "mytool");
}
