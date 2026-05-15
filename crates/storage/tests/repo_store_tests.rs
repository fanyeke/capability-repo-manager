use domain::Repository;
use storage::repo_store::RepositoryStore;
use storage::Database;

fn setup_db() -> Database {
    let db = Database::open_in_memory().unwrap();
    db
}

fn make_repo(id: &str, name: &str, path: &str) -> Repository {
    Repository {
        id: id.to_string(),
        name: name.to_string(),
        path: path.to_string(),
        canonical_path: path.to_string(),
        remote_url: None,
        current_branch: Some("main".to_string()),
        head_commit: Some("abc123".to_string()),
        dirty_state: "clean".to_string(),
        first_indexed_at: "2026-05-14T10:00:00Z".to_string(),
        last_indexed_at: "2026-05-14T10:00:00Z".to_string(),
        capability_index_status: "never_indexed".to_string(),
        last_capability_indexed_at: None,
        last_capability_error: None,
    }
}

#[test]
fn test_insert_and_get_by_id() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = make_repo("r1", "test-project", "/home/user/test-project");
    store.insert(&repo).unwrap();

    let fetched = store.get_by_id("r1").unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "test-project");
}

#[test]
fn test_get_by_path() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    store.insert(&make_repo("r1", "a", "/tmp/a")).unwrap();
    store.insert(&make_repo("r2", "b", "/tmp/b")).unwrap();

    let fetched = store.get_by_path("/tmp/a").unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().id, "r1");
}

#[test]
fn test_get_not_found() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let result = store.get_by_id("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_list_all() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    store.insert(&make_repo("r1", "a", "/tmp/a")).unwrap();
    store.insert(&make_repo("r2", "b", "/tmp/b")).unwrap();
    store.insert(&make_repo("r3", "c", "/tmp/c")).unwrap();

    let all = store.list_all().unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_delete() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    store.insert(&make_repo("r1", "a", "/tmp/a")).unwrap();
    store.delete("r1").unwrap();

    let result = store.get_by_id("r1").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    store.insert(&make_repo("r1", "old-name", "/tmp/r1")).unwrap();

    let mut updated = make_repo("r1", "new-name", "/tmp/r1");
    updated.dirty_state = "modified".to_string();
    store.update(&updated).unwrap();

    let fetched = store.get_by_id("r1").unwrap().unwrap();
    assert_eq!(fetched.name, "new-name");
    assert_eq!(fetched.dirty_state, "modified");
}

#[test]
fn test_get_by_canonical_path() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let mut repo = make_repo("r1", "proj-a", "/real/path/proj-a");
    repo.canonical_path = "/canonical/path/proj-a".to_string();
    store.insert(&repo).unwrap();

    let fetched = store.get_by_canonical_path("/canonical/path/proj-a").unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().id, "r1");

    let not_found = store.get_by_canonical_path("/nonexistent").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_upsert_by_path_inserts_new() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = make_repo("new-id", "new-project", "/tmp/new-project");

    let result = store.upsert_by_path(&repo).unwrap();
    assert_eq!(result.id, "new-id", "should keep provided id");
    assert_eq!(result.name, "new-project");
    assert!(!result.first_indexed_at.is_empty(), "should set first_indexed_at");

    // Verify it was persisted
    let fetched = store.get_by_canonical_path("/tmp/new-project").unwrap();
    assert!(fetched.is_some());
}

#[test]
fn test_upsert_by_path_inserts_generates_uuid_when_empty() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = Repository::new(String::new(), "auto-uuid".to_string(), "/tmp/auto-uuid".to_string());

    let result = store.upsert_by_path(&repo).unwrap();
    assert!(!result.id.is_empty(), "should generate UUID when id is empty");
    assert_eq!(result.name, "auto-uuid");
}

#[test]
fn test_upsert_by_path_updates_existing() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = make_repo("existing-id", "original-name", "/tmp/existing");
    store.insert(&repo).unwrap();

    // Now upsert with same canonical_path but different name
    let mut updated = Repository::new("different-id".to_string(), "updated-name".to_string(), "/tmp/existing".to_string());
    updated.canonical_path = "/tmp/existing".to_string();
    updated.current_branch = Some("develop".to_string());

    let result = store.upsert_by_path(&updated).unwrap();
    assert_eq!(result.id, "existing-id", "should reuse existing id");
    assert_eq!(result.name, "updated-name", "should update name");
    assert_eq!(result.current_branch, Some("develop".to_string()), "should update branch");

    // Verify stored correctly
    let fetched = store.get_by_id("existing-id").unwrap().unwrap();
    assert_eq!(fetched.name, "updated-name");
    assert_eq!(fetched.current_branch, Some("develop".to_string()));
}

#[test]
fn test_upsert_by_path_preserves_first_indexed_at() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = make_repo("preserve-id", "first", "/tmp/preserve");
    store.insert(&repo).unwrap();

    let original_first_indexed = store.get_by_id("preserve-id").unwrap().unwrap().first_indexed_at;

    // Wait a moment, then upsert
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut updated = Repository::new(String::new(), "second".to_string(), "/tmp/preserve".to_string());
    updated.canonical_path = "/tmp/preserve".to_string();

    let result = store.upsert_by_path(&updated).unwrap();
    assert_eq!(result.id, "preserve-id", "should reuse id");
    assert_eq!(
        result.first_indexed_at,
        original_first_indexed,
        "should preserve original first_indexed_at"
    );
    assert_ne!(
        result.last_indexed_at,
        result.first_indexed_at,
        "last_indexed_at should differ from first_indexed_at after re-scan"
    );
}

#[test]
fn test_update_index_status_to_fresh() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = make_repo("status-1", "status-test", "/tmp/status-test");
    store.insert(&repo).unwrap();

    // Update to fresh
    store.update_index_status("status-1", "fresh", None).unwrap();

    let fetched = store.get_by_id("status-1").unwrap().unwrap();
    assert_eq!(fetched.capability_index_status, "fresh");
    assert!(fetched.last_capability_indexed_at.is_some(), "last_capability_indexed_at should be set for fresh status");
    assert!(fetched.last_capability_error.is_none());
}

#[test]
fn test_update_index_status_to_parse_failed_with_error() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = make_repo("status-2", "fail-test", "/tmp/fail-test");
    store.insert(&repo).unwrap();

    // Update to parse_failed with error message
    store.update_index_status("status-2", "parse_failed", Some("Invalid config: missing skills")).unwrap();

    let fetched = store.get_by_id("status-2").unwrap().unwrap();
    assert_eq!(fetched.capability_index_status, "parse_failed");
    assert_eq!(fetched.last_capability_error, Some("Invalid config: missing skills".to_string()));
    // last_capability_indexed_at should NOT be updated for parse_failed
    assert!(fetched.last_capability_indexed_at.is_none(), "should not update indexed_at on failure");
}

#[test]
fn test_update_index_status_transitions() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    let repo = make_repo("status-3", "transitions", "/tmp/transitions");
    store.insert(&repo).unwrap();

    // Default state
    let fetched = store.get_by_id("status-3").unwrap().unwrap();
    assert_eq!(fetched.capability_index_status, "never_indexed");

    // fresh
    store.update_index_status("status-3", "fresh", None).unwrap();
    let fetched = store.get_by_id("status-3").unwrap().unwrap();
    assert_eq!(fetched.capability_index_status, "fresh");

    // stale
    store.update_index_status("status-3", "stale", None).unwrap();
    let fetched = store.get_by_id("status-3").unwrap().unwrap();
    assert_eq!(fetched.capability_index_status, "stale");

    // parse_failed (from stale)
    store.update_index_status("status-3", "parse_failed", Some("error")).unwrap();
    let fetched = store.get_by_id("status-3").unwrap().unwrap();
    assert_eq!(fetched.capability_index_status, "parse_failed");
    assert_eq!(fetched.last_capability_error, Some("error".to_string()));

    // fresh again (from parse_failed) — should clear error and update indexed_at
    store.update_index_status("status-3", "fresh", None).unwrap();
    let fetched = store.get_by_id("status-3").unwrap().unwrap();
    assert_eq!(fetched.capability_index_status, "fresh");
    assert!(fetched.last_capability_error.is_none(), "error should be cleared on success");
    assert!(fetched.last_capability_indexed_at.is_some(), "indexed_at should be set");
}

#[test]
fn test_update_index_status_nonexistent_repo() {
    let db = setup_db();
    let store = RepositoryStore::new(&db);

    // Should not crash — 0 rows updated is OK
    let result = store.update_index_status("does-not-exist", "fresh", None);
    assert!(result.is_ok(), "update_index_status on nonexistent repo should not error");
}
