use std::fs;

/// Helper to create a temp directory with the given file structure.
/// `files` is a list of (relative_path, content) tuples.
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
fn detects_skill_with_skill_md_uppercase() {
    let dir = setup_temp_repo(&[(".claude/skills/my-skill/SKILL.md", "# My Skill\n\nDoes something useful.")]);

    let resources = claude_parser::skill_parser::parse_skills(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    let skill = &resources[0];
    assert_eq!(skill.r#type, "skill");
    assert_eq!(skill.name, "my-skill");
    assert!(skill.source_path.as_ref().unwrap().contains(".claude/skills/my-skill"));
    assert_eq!(skill.scope, "project");

    let meta: serde_json::Value = serde_json::from_str(skill.metadata_json.as_ref().unwrap()).unwrap();
    assert_eq!(meta["summary"], "Does something useful.");
}

#[test]
fn detects_skill_with_skill_md_lowercase() {
    let dir = setup_temp_repo(&[(".claude/skills/another-skill/skill.md", "## Another Skill\n\nDescription here.")]);

    let resources = claude_parser::skill_parser::parse_skills(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "another-skill");
}

#[test]
fn detects_multiple_skills() {
    let dir = setup_temp_repo(&[
        (".claude/skills/skill-a/SKILL.md", "# Skill A\n\nA description."),
        (".claude/skills/skill-b/SKILL.md", "# Skill B\n\nB description."),
        (".claude/skills/skill-c/skill.md", "# Skill C"),
    ]);

    let resources = claude_parser::skill_parser::parse_skills(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 3);
    let names: Vec<&str> = resources.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"skill-a"));
    assert!(names.contains(&"skill-b"));
    assert!(names.contains(&"skill-c"));
}

#[test]
fn skips_directories_without_skill_md() {
    let dir = setup_temp_repo(&[(".claude/skills/empty-dir/.gitkeep", "")]);

    let resources = claude_parser::skill_parser::parse_skills(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 0);
}

#[test]
fn returns_empty_when_no_skills_dir() {
    let dir = setup_temp_repo(&[("README.md", "# My Repo")]);

    let resources = claude_parser::skill_parser::parse_skills(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 0);
}

#[test]
fn extracts_summary_from_first_paragraph() {
    let dir = setup_temp_repo(&[(
        ".claude/skills/docs/SKILL.md",
        "# Documentation Helper\n\nGenerates and updates project documentation.\n\n## Usage\n\nRun it.",
    )]);

    let resources = claude_parser::skill_parser::parse_skills(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    let meta: serde_json::Value = serde_json::from_str(resources[0].metadata_json.as_ref().unwrap()).unwrap();
    assert_eq!(meta["summary"], "Generates and updates project documentation.");
}

#[test]
fn generates_content_hash() {
    let dir = setup_temp_repo(&[(".claude/skills/hash-test/SKILL.md", "content for hashing")]);

    let resources = claude_parser::skill_parser::parse_skills(dir.path().to_str().unwrap());

    assert_eq!(resources.len(), 1);
    assert!(resources[0].content_hash.is_some());
    // SHA256 hash should be 64 hex chars
    assert_eq!(resources[0].content_hash.as_ref().unwrap().len(), 64);
}
