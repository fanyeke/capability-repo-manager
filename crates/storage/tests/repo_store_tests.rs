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
        remote_url: None,
        current_branch: Some("main".to_string()),
        head_commit: Some("abc123".to_string()),
        dirty_state: "clean".to_string(),
        last_indexed_at: "2026-05-14T10:00:00Z".to_string(),
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
