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
