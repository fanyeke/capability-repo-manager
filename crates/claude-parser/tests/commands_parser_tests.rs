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
fn parses_command_from_commands_dir() {
    let dir = setup_temp_repo(&[(".claude/commands/deploy.md", "# deploy\n\nDeploy the application.")]);

    let resources = claude_parser::commands_parser::parse_commands(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    let cmd = &resources[0];
    assert_eq!(cmd.r#type, "command");
    assert_eq!(cmd.name, "deploy");
    assert!(cmd.source_path.as_ref().unwrap().contains(".claude/commands/deploy.md"));
}

#[test]
fn parses_multiple_commands() {
    let dir = setup_temp_repo(&[
        (".claude/commands/deploy.md", "# Deploy\n\nDeploy script."),
        (".claude/commands/rollback.md", "# Rollback\n\nRollback script."),
    ]);

    let resources = claude_parser::commands_parser::parse_commands(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 2);
    let names: Vec<&str> = resources.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"deploy"));
    assert!(names.contains(&"rollback"));
}

#[test]
fn no_commands_dir_returns_empty() {
    let dir = setup_temp_repo(&[("README.md", "# No commands")]);

    let resources = claude_parser::commands_parser::parse_commands(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 0);
}

#[test]
fn skips_non_md_files_in_commands_dir() {
    let dir = setup_temp_repo(&[(".claude/commands/.gitkeep", ""), (".claude/commands/deploy.md", "# Deploy")]);

    let resources = claude_parser::commands_parser::parse_commands(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "deploy");
}
