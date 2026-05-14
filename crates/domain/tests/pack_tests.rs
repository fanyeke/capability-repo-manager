use domain::CapabilityPack;
use uuid::Uuid;

#[test]
fn test_create_project_pack() {
    let pack = CapabilityPack {
        id: Uuid::new_v4().to_string(),
        name: "frontend-baseline".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Frontend team baseline config".to_string()),
        pack_type: "baseline".to_string(),
        manifest_path: "/home/user/.capability-manager/packs/frontend-baseline/pack.manifest.json".to_string(),
        source_repo_id: Some(Uuid::new_v4().to_string()),
        source_commit: Some("abc123def".to_string()),
        created_at: "2026-05-14T10:00:00Z".to_string(),
        storage_dir: "/home/user/.capability-manager/packs/frontend-baseline".to_string(),
    };

    assert_eq!(pack.name, "frontend-baseline");
    assert_eq!(pack.version, "1.0.0");
    assert_eq!(pack.pack_type, "baseline");
    assert!(pack.description.is_some());
}

#[test]
fn test_pack_type_valid_values() {
    let valid_types = vec!["project", "blueprint", "baseline"];
    for pt in valid_types {
        let pack = CapabilityPack {
            id: Uuid::new_v4().to_string(),
            name: format!("pack-{}", pt),
            version: "0.1.0".to_string(),
            description: None,
            pack_type: pt.to_string(),
            manifest_path: format!("/tmp/{}/pack.manifest.json", pt),
            source_repo_id: None,
            source_commit: None,
            created_at: "2026-05-14T10:00:00Z".to_string(),
            storage_dir: format!("/tmp/{}", pt),
        };
        assert_eq!(pack.pack_type, pt);
    }
}

#[test]
fn test_version_format_semver() {
    let valid_versions = vec!["1.0.0", "0.1.0", "2.3.1", "10.20.30", "1.0.0-alpha.1"];
    for ver in valid_versions {
        let pack = CapabilityPack {
            id: Uuid::new_v4().to_string(),
            name: "version-test".to_string(),
            version: ver.to_string(),
            description: None,
            pack_type: "project".to_string(),
            manifest_path: "/tmp/version-test/pack.manifest.json".to_string(),
            source_repo_id: None,
            source_commit: None,
            created_at: "2026-05-14T10:00:00Z".to_string(),
            storage_dir: "/tmp/version-test".to_string(),
        };
        assert_eq!(pack.version, ver);
    }
}

#[test]
fn test_pack_source_reference() {
    let source_repo_id = Uuid::new_v4().to_string();
    let pack = CapabilityPack {
        id: Uuid::new_v4().to_string(),
        name: "from-repo-pack".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        pack_type: "project".to_string(),
        manifest_path: "/tmp/pack/pack.manifest.json".to_string(),
        source_repo_id: Some(source_repo_id.clone()),
        source_commit: Some("deadbeef".to_string()),
        created_at: "2026-05-14T10:00:00Z".to_string(),
        storage_dir: "/tmp/pack".to_string(),
    };

    assert_eq!(pack.source_repo_id.unwrap(), source_repo_id);
    assert_eq!(pack.source_commit.unwrap(), "deadbeef");
}

#[test]
fn test_blueprint_pack_no_source_repo() {
    let pack = CapabilityPack {
        id: Uuid::new_v4().to_string(),
        name: "empty-blueprint".to_string(),
        version: "0.1.0".to_string(),
        description: Some("Reusable blueprint".to_string()),
        pack_type: "blueprint".to_string(),
        manifest_path: "/tmp/blueprint/pack.manifest.json".to_string(),
        source_repo_id: None,
        source_commit: None,
        created_at: "2026-05-14T10:00:00Z".to_string(),
        storage_dir: "/tmp/blueprint".to_string(),
    };

    assert_eq!(pack.pack_type, "blueprint");
    assert!(pack.source_repo_id.is_none());
    assert!(pack.source_commit.is_none());
}

#[test]
fn test_pack_serde_roundtrip() {
    let pack = CapabilityPack {
        id: Uuid::new_v4().to_string(),
        name: "serde-pack".to_string(),
        version: "2.0.0".to_string(),
        description: Some("Test serialization".to_string()),
        pack_type: "project".to_string(),
        manifest_path: "/tmp/serde/pack.manifest.json".to_string(),
        source_repo_id: Some(Uuid::new_v4().to_string()),
        source_commit: Some("1234567".to_string()),
        created_at: "2026-05-14T12:00:00Z".to_string(),
        storage_dir: "/tmp/serde".to_string(),
    };

    let json = serde_json::to_string(&pack).expect("serialize");
    let deserialized: CapabilityPack = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.id, pack.id);
    assert_eq!(deserialized.name, pack.name);
    assert_eq!(deserialized.version, pack.version);
    assert_eq!(deserialized.pack_type, pack.pack_type);
    assert_eq!(deserialized.storage_dir, pack.storage_dir);
}
