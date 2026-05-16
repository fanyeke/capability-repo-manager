use pack_engine::manifest::*;

#[test]
fn test_build_manifest_basic() {
    let manifest = Manifest::build(ManifestInput {
        name: "my-pack".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test pack".to_string()),
        pack_type: Some("project".to_string()),
        source_repo: Some("/home/user/repo".to_string()),
        source_commit: Some("abc123def".to_string()),
        resources: vec![ManifestResource {
            resource_type: "skill".to_string(),
            name: "my-skill".to_string(),
            source: "resources/skills/my-skill/SKILL.md".to_string(),
            hash: Some("sha256hash".to_string()),
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    })
    .unwrap();

    assert_eq!(manifest.schema_version, "1.0");
    assert_eq!(manifest.name, "my-pack");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.description.as_deref(), Some("A test pack"));
    assert_eq!(manifest.pack_type.as_deref(), Some("project"));
    assert!(manifest.source.is_some());
    assert_eq!(manifest.resources.len(), 1);
    assert_eq!(manifest.resources[0].name, "my-skill");
}

#[test]
fn test_manifest_serialization_roundtrip() {
    let manifest = Manifest::build(ManifestInput {
        name: "roundtrip-pack".to_string(),
        version: "2.1.0-beta.1".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![
            ManifestResource {
                resource_type: "skill".to_string(),
                name: "skill-a".to_string(),
                source: "resources/skills/skill-a/SKILL.md".to_string(),
                hash: Some("hash-a".to_string()),
                dependencies: None,
            },
            ManifestResource {
                resource_type: "mcp".to_string(),
                name: "my-server".to_string(),
                source: "mcp/mcp.json".to_string(),
                hash: None,
                dependencies: Some(vec![ResourceDependency {
                    dep_type: "command".to_string(),
                    name: "node".to_string(),
                    required: Some(true),
                }]),
            },
        ],
        env_placeholders: vec![EnvPlaceholder {
            name: "API_KEY".to_string(),
            required: Some(true),
            description: Some("API key for service".to_string()),
        }],
        validation_rules: vec!["skill-structure-valid".to_string()],
    })
    .unwrap();

    let json = manifest.to_json().unwrap();
    let parsed = Manifest::from_json(&json).unwrap();

    assert_eq!(manifest, parsed);
}

#[test]
fn test_manifest_requires_at_least_one_resource() {
    let result = Manifest::build(ManifestInput {
        name: "empty".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![],
        env_placeholders: vec![],
        validation_rules: vec![],
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least one resource"));
}

#[test]
fn test_manifest_schema_version_is_required() {
    let manifest = Manifest::build(ManifestInput {
        name: "p".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![ManifestResource {
            resource_type: "rule".to_string(),
            name: "r".to_string(),
            source: "r.md".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    })
    .unwrap();
    assert_eq!(manifest.schema_version, "1.0");
}

#[test]
fn test_manifest_includes_source_info() {
    let manifest = Manifest::build(ManifestInput {
        name: "sourced-pack".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: Some("baseline".to_string()),
        source_repo: Some("/repos/my-project".to_string()),
        source_commit: Some("deadbeef".to_string()),
        resources: vec![ManifestResource {
            resource_type: "hook".to_string(),
            name: "pre-commit".to_string(),
            source: "resources/pre-commit.md".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    })
    .unwrap();

    let source = manifest.source.unwrap();
    assert_eq!(source.repo.as_deref(), Some("/repos/my-project"));
    assert_eq!(source.commit.as_deref(), Some("deadbeef"));
}

#[test]
fn test_manifest_resources_have_required_fields() {
    let manifest = Manifest::build(ManifestInput {
        name: "required-fields".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![
            ManifestResource {
                resource_type: "skill".to_string(),
                name: "s1".to_string(),
                source: "s1/SKILL.md".to_string(),
                hash: None,
                dependencies: None,
            },
            ManifestResource {
                resource_type: "mcp".to_string(),
                name: "m1".to_string(),
                source: "m1/mcp.json".to_string(),
                hash: Some("abc".to_string()),
                dependencies: None,
            },
        ],
        env_placeholders: vec![],
        validation_rules: vec![],
    })
    .unwrap();

    for resource in &manifest.resources {
        assert!(!resource.resource_type.is_empty(), "type must be set");
        assert!(!resource.name.is_empty(), "name must be set");
        assert!(!resource.source.is_empty(), "source must be set");
    }
}

#[test]
fn test_manifest_env_placeholders() {
    let manifest = Manifest::build(ManifestInput {
        name: "env-pack".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![ManifestResource {
            resource_type: "settings".to_string(),
            name: "settings".to_string(),
            source: "settings/settings.json".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![
            EnvPlaceholder { name: "OPENAI_API_KEY".to_string(), required: Some(true), description: None },
            EnvPlaceholder {
                name: "DATABASE_URL".to_string(),
                required: Some(false),
                description: Some("Database connection".to_string()),
            },
        ],
        validation_rules: vec![],
    })
    .unwrap();

    let env = manifest.env.unwrap();
    assert_eq!(env.len(), 2);
    assert_eq!(env[0].name, "OPENAI_API_KEY");
    assert_eq!(env[0].required, Some(true));
    assert_eq!(env[1].name, "DATABASE_URL");
}

#[test]
fn test_manifest_empty_name_rejected() {
    let result = Manifest::build(ManifestInput {
        name: "".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![ManifestResource {
            resource_type: "rule".to_string(),
            name: "r".to_string(),
            source: "r.md".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    });
    assert!(result.is_err());
}

#[test]
fn test_manifest_name_too_long_rejected() {
    let result = Manifest::build(ManifestInput {
        name: "a".repeat(129),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![ManifestResource {
            resource_type: "rule".to_string(),
            name: "r".to_string(),
            source: "r.md".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    });
    assert!(result.is_err());
}

#[test]
fn test_manifest_description_too_long_rejected() {
    let result = Manifest::build(ManifestInput {
        name: "ok-name".to_string(),
        version: "1.0.0".to_string(),
        description: Some("x".repeat(1025)),
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![ManifestResource {
            resource_type: "rule".to_string(),
            name: "r".to_string(),
            source: "r.md".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    });
    assert!(result.is_err());
}

#[test]
fn test_manifest_invalid_version_rejected() {
    let result = Manifest::build(ManifestInput {
        name: "test".to_string(),
        version: "not-a-version".to_string(),
        description: None,
        pack_type: None,
        source_repo: None,
        source_commit: None,
        resources: vec![ManifestResource {
            resource_type: "rule".to_string(),
            name: "r".to_string(),
            source: "r.md".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    });
    assert!(result.is_err());
}

#[test]
fn test_manifest_to_json() {
    let manifest = Manifest::build(ManifestInput {
        name: "json-test".to_string(),
        version: "1.0.0".to_string(),
        description: Some("JSON output test".to_string()),
        pack_type: Some("project".to_string()),
        source_repo: None,
        source_commit: None,
        resources: vec![ManifestResource {
            resource_type: "skill".to_string(),
            name: "test-skill".to_string(),
            source: "resources/skills/test-skill/SKILL.md".to_string(),
            hash: None,
            dependencies: None,
        }],
        env_placeholders: vec![],
        validation_rules: vec![],
    })
    .unwrap();

    let json = manifest.to_json().unwrap();
    assert!(json.contains("\"schemaVersion\""));
    assert!(json.contains("\"1.0\""));
    assert!(json.contains("\"json-test\""));
}
