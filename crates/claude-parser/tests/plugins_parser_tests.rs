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
fn parses_plugin_from_plugins_dir() {
    let dir = setup_temp_repo(&[(
        ".claude/plugins/code-formatter.md",
        "# code-formatter\n\nFormats code on save.",
    )]);

    let resources = claude_parser::plugins_parser::parse_plugins(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].r#type, "plugin");
    assert_eq!(resources[0].name, "code-formatter");
}

#[test]
fn parses_multiple_plugins() {
    let dir = setup_temp_repo(&[
        (".claude/plugins/linter.md", "# Linter"),
        (".claude/plugins/formatter.md", "# Formatter"),
    ]);

    let resources = claude_parser::plugins_parser::parse_plugins(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 2);
}

#[test]
fn no_plugins_dir_returns_empty() {
    let dir = setup_temp_repo(&[("README.md", "# No plugins")]);

    let resources = claude_parser::plugins_parser::parse_plugins(
        dir.path().to_str().unwrap(),
    );

    assert_eq!(resources.len(), 0);
}
