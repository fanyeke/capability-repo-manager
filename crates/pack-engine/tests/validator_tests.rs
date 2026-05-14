use std::fs;

use pack_engine::manifest::{Manifest, ManifestInput, ManifestResource};
use pack_engine::validator::*;

fn make_manifest(
    name: &str,
    version: &str,
    resources: Vec<ManifestResource>,
) -> Manifest {
    Manifest::build(ManifestInput {
        name: name.to_string(),
        version: version.to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources,
        env_placeholders: vec![],
        validation_rules: vec![],
    })
    .unwrap()
}

fn make_resource(resource_type: &str, name: &str, source: &str) -> ManifestResource {
    ManifestResource {
        resource_type: resource_type.to_string(),
        name: name.to_string(),
        source: source.to_string(),
        hash: None,
        dependencies: None,
    }
}

#[test]
fn test_validator_schema_conformance() {
    let manifest = make_manifest(
        "valid-pack",
        "1.0.0",
        vec![make_resource("skill", "my-skill", "resources/skills/my-skill/SKILL.md")],
    );
    let result = validate_schema(&manifest);
    assert!(result.valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_validator_missing_schema_version_fails() {
    let mut manifest = make_manifest(
        "test",
        "1.0.0",
        vec![make_resource("skill", "s", "path.md")],
    );
    manifest.schema_version = "2.0".to_string();
    let result = validate_schema(&manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("schemaVersion")));
}

#[test]
fn test_validator_missing_name_fails() {
    let mut manifest = make_manifest(
        "test",
        "1.0.0",
        vec![make_resource("skill", "s", "path.md")],
    );
    manifest.name = String::new();
    let result = validate_schema(&manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("name is empty")));
}

#[test]
fn test_validator_invalid_version_format_fails() {
    let mut manifest = make_manifest(
        "test",
        "1.0.0",
        vec![make_resource("skill", "s", "path.md")],
    );
    manifest.version = "not-semver".to_string();
    let result = validate_schema(&manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("Invalid version")));
}

#[test]
fn test_validator_empty_resources_fails() {
    let manifest = Manifest {
        schema_version: "1.0".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source: None,
        resources: vec![],
        env: None,
        validation: None,
    };
    let result = validate_schema(&manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("no resources")));
}

#[test]
fn test_validator_file_existence_check() {
    let temp = tempfile::TempDir::new().unwrap();
    let manifest = make_manifest(
        "file-test",
        "1.0.0",
        vec![make_resource("rule", "missing-rule", "rules/missing.md")],
    );
    let result = validate_file_existence(temp.path(), &manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("not found")));
}

#[test]
fn test_validator_hash_integrity_check() {
    let temp = tempfile::TempDir::new().unwrap();
    let file_path = temp.path().join("test.md");
    fs::write(&file_path, "some content").unwrap();

    let mut resource = make_resource("rule", "test-rule", "test.md");
    resource.hash = Some("wronghash".to_string());

    let manifest = Manifest {
        schema_version: "1.0".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source: None,
        resources: vec![resource],
        env: None,
        validation: None,
    };

    let result = validate_hash_integrity(temp.path(), &manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("hash mismatch")));
}

#[test]
fn test_validator_hash_match_passes() {
    let temp = tempfile::TempDir::new().unwrap();
    let file_path = temp.path().join("test.md");
    fs::write(&file_path, "hello world").unwrap();

    let mut resource = make_resource("rule", "test-rule", "test.md");
    // SHA256 of "hello world"
    resource.hash = Some(
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".to_string(),
    );

    let manifest = Manifest {
        schema_version: "1.0".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source: None,
        resources: vec![resource],
        env: None,
        validation: None,
    };

    let result = validate_hash_integrity(temp.path(), &manifest);
    assert!(result.valid);
}

#[test]
fn test_validator_env_placeholder_detection() {
    let temp = tempfile::TempDir::new().unwrap();
    let file_path = temp.path().join("settings.json");
    fs::write(&file_path, r#"{"key": "${API_KEY}"}"#).unwrap();

    let resource = make_resource("settings", "env-settings", "settings.json");
    let manifest = Manifest {
        schema_version: "1.0".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source: None,
        resources: vec![resource],
        env: None,
        validation: None,
    };

    let result = validate_env_placeholders(temp.path(), &manifest);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("unresolved env placeholders")));
}

#[test]
fn test_validator_invalid_resource_type() {
    let mut manifest = make_manifest(
        "test",
        "1.0.0",
        vec![make_resource("skill", "s", "path.md")],
    );
    manifest.resources[0].resource_type = "bad_type".to_string();
    let result = validate_schema(&manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("invalid type")));
}

#[test]
fn test_validator_invalid_pack_type() {
    let mut manifest = make_manifest(
        "test",
        "1.0.0",
        vec![make_resource("skill", "s", "path.md")],
    );
    manifest.pack_type = Some("unknown_type".to_string());
    let result = validate_schema(&manifest);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("Invalid packType")));
}

#[test]
fn test_validate_pack_all_checks() {
    let temp = tempfile::TempDir::new().unwrap();

    // Create a valid pack directory
    let pack_dir = temp.path().join("my-pack");
    fs::create_dir_all(pack_dir.join("resources/skills/my-skill")).unwrap();
    fs::write(
        pack_dir.join("resources/skills/my-skill/SKILL.md"),
        "# My Skill",
    )
    .unwrap();

    let mut resource = make_resource("skill", "my-skill", "resources/skills/my-skill/SKILL.md");
    resource.hash = Some(
        "365578fd8b25523351c815c9e6b86a4092f6df9d0392da7e5351510814ecdace".to_string(),
    );

    let manifest = Manifest {
        schema_version: "1.0".to_string(),
        name: "my-pack".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source: None,
        resources: vec![resource],
        env: None,
        validation: None,
    };

    let result = validate_pack(&pack_dir, &manifest);
    assert!(result.valid, "Expected valid pack, got errors: {:?}", result.errors);
}
