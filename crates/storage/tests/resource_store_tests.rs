use domain::CapabilityResource;
use storage::resource_store::ResourceStore;
use storage::Database;

fn setup_db() -> Database {
    Database::open_in_memory().unwrap()
}

fn make_resource(id: &str, repo_id: &str, rtype: &str, name: &str) -> CapabilityResource {
    CapabilityResource {
        id: id.to_string(),
        repo_id: Some(repo_id.to_string()),
        pack_id: None,
        r#type: rtype.to_string(),
        name: name.to_string(),
        source_path: Some(format!(".claude/{}/{}", rtype, name)),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: Some("hash123".to_string()),
        metadata_json: None,
        error_message: None,
    }
}

#[test]
fn test_insert_batch_and_get_by_repo() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    let resources = vec![
        make_resource("res-1", "repo-1", "skill", "ui-review"),
        make_resource("res-2", "repo-1", "hook", "pre-commit"),
        make_resource("res-3", "repo-2", "skill", "backend-code"),
    ];

    store.insert_batch(&resources).unwrap();

    let repo1_resources = store.get_by_repo("repo-1").unwrap();
    assert_eq!(repo1_resources.len(), 2);

    let repo2_resources = store.get_by_repo("repo-2").unwrap();
    assert_eq!(repo2_resources.len(), 1);
}

#[test]
fn test_get_by_pack() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    let resources = vec![
        CapabilityResource {
            id: "res-p1".to_string(),
            repo_id: None,
            pack_id: Some("pack-1".to_string()),
            r#type: "skill".to_string(),
            name: "packed-skill".to_string(),
            source_path: Some("resources/skills/packed-skill".to_string()),
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: Some("abc".to_string()),
            metadata_json: None,
            error_message: None,
        },
    ];

    store.insert_batch(&resources).unwrap();

    let pack_resources = store.get_by_pack("pack-1").unwrap();
    assert_eq!(pack_resources.len(), 1);
    assert_eq!(pack_resources[0].name, "packed-skill");
}

#[test]
fn test_delete_by_repo() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    store.insert_batch(&vec![
        make_resource("r1", "repo-A", "skill", "s1"),
        make_resource("r2", "repo-A", "hook", "h1"),
        make_resource("r3", "repo-B", "rule", "r1"),
    ]).unwrap();

    store.delete_by_repo("repo-A").unwrap();

    let remaining = store.get_by_repo("repo-A").unwrap();
    assert_eq!(remaining.len(), 0);

    let repo_b = store.get_by_repo("repo-B").unwrap();
    assert_eq!(repo_b.len(), 1);
}

#[test]
fn test_replace_for_pack() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    let resources = vec![
        CapabilityResource {
            id: "r1".to_string(),
            repo_id: None,
            pack_id: Some("pack-1".to_string()),
            r#type: "skill".to_string(),
            name: "packed-skill".to_string(),
            source_path: Some("resources/skills/packed-skill".to_string()),
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: Some("abc".to_string()),
            metadata_json: None,
            error_message: None,
        },
        CapabilityResource {
            id: "r2".to_string(),
            repo_id: None,
            pack_id: Some("pack-1".to_string()),
            r#type: "hook".to_string(),
            name: "packed-hook".to_string(),
            source_path: Some("resources/hooks/packed-hook".to_string()),
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: Some("def".to_string()),
            metadata_json: None,
            error_message: None,
        },
    ];

    // Initial insert
    store.replace_for_pack("pack-1", &resources).unwrap();
    let found = store.get_by_pack("pack-1").unwrap();
    assert_eq!(found.len(), 2);

    // Replace with a subset
    let subset = vec![resources[0].clone()];
    store.replace_for_pack("pack-1", &subset).unwrap();
    let found = store.get_by_pack("pack-1").unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "packed-skill");
}

#[test]
fn test_delete_by_pack() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    let resources = vec![
        CapabilityResource {
            id: "rp1".to_string(),
            repo_id: None,
            pack_id: Some("pack-X".to_string()),
            r#type: "skill".to_string(),
            name: "skill-a".to_string(),
            source_path: None,
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: None,
            metadata_json: None,
            error_message: None,
        },
        CapabilityResource {
            id: "rp2".to_string(),
            repo_id: None,
            pack_id: Some("pack-X".to_string()),
            r#type: "skill".to_string(),
            name: "skill-b".to_string(),
            source_path: None,
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: None,
            metadata_json: None,
            error_message: None,
        },
    ];

    store.insert_batch(&resources).unwrap();
    assert_eq!(store.get_by_pack("pack-X").unwrap().len(), 2);

    store.delete_by_pack("pack-X").unwrap();
    assert_eq!(store.get_by_pack("pack-X").unwrap().len(), 0);
}

#[test]
fn test_get_by_id_found() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    let resources = vec![
        make_resource("target-id", "repo-1", "skill", "find-me"),
        make_resource("other-id", "repo-1", "hook", "ignore-me"),
    ];
    store.insert_batch(&resources).unwrap();

    let result = store.get_by_id("target-id").unwrap();
    assert!(result.is_some());
    let resource = result.unwrap();
    assert_eq!(resource.id, "target-id");
    assert_eq!(resource.name, "find-me");
    assert_eq!(resource.r#type, "skill");
}

#[test]
fn test_get_by_id_not_found() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    let result = store.get_by_id("non-existent-id").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_by_id_returns_none_after_delete() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    store.insert_batch(&vec![make_resource("to-delete", "repo-1", "skill", "ephemeral")]).unwrap();

    let before = store.get_by_id("to-delete").unwrap();
    assert!(before.is_some());

    store.delete_by_repo("repo-1").unwrap();

    let after = store.get_by_id("to-delete").unwrap();
    assert!(after.is_none());
}

#[test]
fn test_roundtrip_with_error_message() {
    let db = setup_db();
    let store = ResourceStore::new(&db);

    let broken = CapabilityResource {
        id: "broken-1".to_string(),
        repo_id: Some("repo-1".to_string()),
        pack_id: None,
        r#type: "mcp".to_string(),
        name: "broken-mcp".to_string(),
        source_path: Some(".claude/mcp.json".to_string()),
        scope: "project".to_string(),
        tracked_by_git: true,
        content_hash: None,
        metadata_json: None,
        error_message: Some("Parse error: unexpected token".to_string()),
    };

    store.insert_batch(&vec![broken]).unwrap();

    let fetched = store.get_by_repo("repo-1").unwrap();
    assert_eq!(fetched.len(), 1);
    assert!(fetched[0].error_message.is_some());
}
