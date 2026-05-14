use domain::Repository;
use uuid::Uuid;

#[test]
fn test_create_repository_with_required_fields() {
    let id = Uuid::new_v4().to_string();
    let repo = Repository::new(
        id.clone(),
        "my-project".to_string(),
        "/home/user/code/my-project".to_string(),
    );

    assert_eq!(repo.id, id);
    assert_eq!(repo.name, "my-project");
    assert_eq!(repo.path, "/home/user/code/my-project");
    assert_eq!(repo.remote_url, None);
    assert_eq!(repo.current_branch, None);
    assert_eq!(repo.head_commit, None);
    assert_eq!(repo.dirty_state, "unknown");
    assert!(repo.last_indexed_at.is_empty());
}

#[test]
fn test_repository_with_full_metadata() {
    let repo = Repository {
        id: Uuid::new_v4().to_string(),
        name: "full-project".to_string(),
        path: "/home/user/code/full-project".to_string(),
        remote_url: Some("git@github.com:user/full-project.git".to_string()),
        current_branch: Some("main".to_string()),
        head_commit: Some("abc123def456".to_string()),
        dirty_state: "clean".to_string(),
        last_indexed_at: "2026-05-14T10:00:00Z".to_string(),
    };

    assert_eq!(repo.remote_url.unwrap(), "git@github.com:user/full-project.git");
    assert_eq!(repo.current_branch.unwrap(), "main");
    assert_eq!(repo.head_commit.unwrap(), "abc123def456");
    assert_eq!(repo.dirty_state, "clean");
}

#[test]
fn test_dirty_state_valid_values() {
    let valid_states = vec!["clean", "modified", "unknown"];
    for state in valid_states {
        let repo = Repository {
            id: Uuid::new_v4().to_string(),
            name: "test".to_string(),
            path: format!("/tmp/{}", state),
            remote_url: None,
            current_branch: None,
            head_commit: None,
            dirty_state: state.to_string(),
            last_indexed_at: String::new(),
        };
        assert_eq!(repo.dirty_state, state);
    }
}

#[test]
fn test_repository_dirty_state_transition() {
    let mut repo = Repository::new(
        Uuid::new_v4().to_string(),
        "transition-test".to_string(),
        "/tmp/transition-test".to_string(),
    );
    assert_eq!(repo.dirty_state, "unknown");

    repo.dirty_state = "clean".to_string();
    assert_eq!(repo.dirty_state, "clean");

    repo.dirty_state = "modified".to_string();
    assert_eq!(repo.dirty_state, "modified");

    repo.dirty_state = "clean".to_string();
    assert_eq!(repo.dirty_state, "clean");
}

#[test]
fn test_repository_serde_roundtrip() {
    let repo = Repository {
        id: Uuid::new_v4().to_string(),
        name: "serde-test".to_string(),
        path: "/tmp/serde-test".to_string(),
        remote_url: Some("https://github.com/test/repo.git".to_string()),
        current_branch: Some("develop".to_string()),
        head_commit: Some("fedcba987654".to_string()),
        dirty_state: "modified".to_string(),
        last_indexed_at: "2026-05-14T12:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&repo).expect("serialize");
    let deserialized: Repository = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.id, repo.id);
    assert_eq!(deserialized.name, repo.name);
    assert_eq!(deserialized.path, repo.path);
    assert_eq!(deserialized.remote_url, repo.remote_url);
    assert_eq!(deserialized.current_branch, repo.current_branch);
    assert_eq!(deserialized.head_commit, repo.head_commit);
    assert_eq!(deserialized.dirty_state, repo.dirty_state);
    assert_eq!(deserialized.last_indexed_at, repo.last_indexed_at);
}
