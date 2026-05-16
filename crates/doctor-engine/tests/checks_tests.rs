use doctor_engine::checks;

use std::fs;
use tempfile::TempDir;

// ── Skill structure checks ──

#[test]
fn skill_structure_complete_skill() {
    let dir = TempDir::new().unwrap();
    let skills_dir = dir.path().join(".claude").join("skills").join("my-skill");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(skills_dir.join("SKILL.md"), "# My Skill\nDescription.").unwrap();

    let issues = checks::check_skill_structure(dir.path());
    assert!(issues.is_empty(), "Complete skill should produce no issues, got: {:?}", issues);
}

#[test]
fn skill_structure_missing_md() {
    let dir = TempDir::new().unwrap();
    let skills_dir = dir.path().join(".claude").join("skills").join("empty-skill");
    fs::create_dir_all(&skills_dir).unwrap();
    // No SKILL.md or skill.md

    let issues = checks::check_skill_structure(dir.path());
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, "warning");
    assert_eq!(issues[0].code, "SKILL_STRUCTURE_INCOMPLETE");
    assert!(issues[0].message.contains("SKILL.md"));
}

#[test]
fn skill_structure_empty_dir() {
    let dir = TempDir::new().unwrap();
    let skills_dir = dir.path().join(".claude").join("skills");
    fs::create_dir_all(&skills_dir).unwrap();
    // Empty skills dir is not an error — no skills defined

    let issues = checks::check_skill_structure(dir.path());
    assert!(issues.is_empty());
}

#[test]
fn skill_structure_no_skills_dir() {
    let dir = TempDir::new().unwrap();
    // No .claude/skills directory at all
    let issues = checks::check_skill_structure(dir.path());
    assert!(issues.is_empty());
}

#[test]
fn skill_structure_lowercase_skill_md() {
    let dir = TempDir::new().unwrap();
    let skills_dir = dir.path().join(".claude").join("skills").join("my-skill");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(skills_dir.join("skill.md"), "# My Skill\nDescription.").unwrap();

    let issues = checks::check_skill_structure(dir.path());
    assert!(issues.is_empty(), "skill.md (lowercase) should be accepted");
}

#[test]
fn skill_structure_multiple_skills_mixed() {
    let dir = TempDir::new().unwrap();
    let skills_base = dir.path().join(".claude").join("skills");

    // Good skill
    let good = skills_base.join("good-skill");
    fs::create_dir_all(&good).unwrap();
    fs::write(good.join("SKILL.md"), "# Good").unwrap();

    // Bad skill
    let bad = skills_base.join("bad-skill");
    fs::create_dir_all(&bad).unwrap();
    // missing SKILL.md

    let issues = checks::check_skill_structure(dir.path());
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "SKILL_STRUCTURE_INCOMPLETE");
}

// ── Hook target checks ──

use doctor_engine::checks::HookConfig;

#[test]
fn hook_target_all_exist() {
    let dir = TempDir::new().unwrap();
    let script = dir.path().join("scripts").join("deploy.sh");
    fs::create_dir_all(script.parent().unwrap()).unwrap();
    fs::write(&script, "#!/bin/bash").unwrap();

    let hooks = vec![HookConfig {
        name: "deploy-hook".into(),
        hook_type: "PostToolUse".into(),
        command: Some("bash scripts/deploy.sh".to_string()),
        script_path: None,
    }];

    let issues = checks::check_hook_targets(dir.path(), &hooks);
    assert!(issues.is_empty(), "Existing script: no issues, got {:?}", issues);
}

#[test]
fn hook_target_missing_script() {
    let dir = TempDir::new().unwrap();

    let hooks = vec![HookConfig {
        name: "broken-hook".into(),
        hook_type: "PostToolUse".into(),
        command: Some("bash scripts/nonexistent.sh".to_string()),
        script_path: None,
    }];

    let issues = checks::check_hook_targets(dir.path(), &hooks);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, "critical");
    assert_eq!(issues[0].code, "HOOK_TARGET_MISSING");
    assert!(issues[0].message.contains("nonexistent.sh"));
}

#[test]
fn hook_target_empty_commands() {
    let dir = TempDir::new().unwrap();

    let hooks = vec![HookConfig {
        name: "no-command-hook".into(),
        hook_type: "PreToolUse".into(),
        command: None,
        script_path: None,
    }];

    let issues = checks::check_hook_targets(dir.path(), &hooks);
    // Hook with no command/script is informational, not an error
    assert!(issues.is_empty() || issues.iter().all(|i| i.severity == "info"));
}

#[test]
fn hook_target_no_hooks() {
    let dir = TempDir::new().unwrap();
    let issues = checks::check_hook_targets(dir.path(), &[]);
    assert!(issues.is_empty());
}

// ── Env placeholder checks ──

#[test]
fn env_placeholder_all_resolved() {
    let configs = vec![("settings.json".to_string(), r#"{"hooks": {}, "model": "sonnet"}"#.to_string())];

    let issues = checks::check_env_placeholders(&configs);
    assert!(issues.is_empty());
}

#[test]
fn env_placeholder_unresolved_var() {
    let configs = vec![(
        "mcp.json".to_string(),
        r#"{"servers": [{"name": "test", "command": "node", "env": {"API_KEY": "${OPENAI_API_KEY}"}}]}"#.to_string(),
    )];

    let issues = checks::check_env_placeholders(&configs);
    assert!(!issues.is_empty());
    assert_eq!(issues[0].severity, "warning");
    assert_eq!(issues[0].code, "ENV_PLACEHOLDER_UNRESOLVED");
    assert!(issues[0].message.contains("OPENAI_API_KEY"));
}

#[test]
fn env_placeholder_multiple_vars() {
    let configs = vec![(
        "settings.json".to_string(),
        r#"{"api_key": "${SECRET}", "url": "${API_URL}", "public": "no-var"}"#.to_string(),
    )];

    let issues = checks::check_env_placeholders(&configs);
    assert_eq!(issues.len(), 2);
    let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
    assert!(codes.contains(&"ENV_PLACEHOLDER_UNRESOLVED"));
}

#[test]
fn env_placeholder_no_configs() {
    let issues = checks::check_env_placeholders(&[]);
    assert!(issues.is_empty());
}

// ── MCP config integrity checks ──

#[test]
fn mcp_config_valid() {
    let json = r#"[{"name": "server1", "command": "node", "args": ["server.js"]}]"#;
    let issues = checks::check_mcp_config(Some(json));
    assert!(issues.is_empty(), "Valid MCP config: no issues, got {:?}", issues);
}

#[test]
fn mcp_config_missing_server_name() {
    let json = r#"[{"command": "node", "args": ["server.js"]}]"#;
    let issues = checks::check_mcp_config(Some(json));
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, "critical");
    assert_eq!(issues[0].code, "MCP_CONFIG_MISSING_NAME");
}

#[test]
fn mcp_config_invalid_json() {
    let json = r#"not valid json {{"#;
    let issues = checks::check_mcp_config(Some(json));
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, "critical");
    assert_eq!(issues[0].code, "MCP_CONFIG_PARSE_ERROR");
}

#[test]
fn mcp_config_missing() {
    let issues = checks::check_mcp_config(None);
    // No MCP config is fine — no MCP servers configured
    assert!(issues.is_empty());
}

#[test]
fn mcp_config_empty_array() {
    let issues = checks::check_mcp_config(Some("[]"));
    assert!(issues.is_empty());
}
