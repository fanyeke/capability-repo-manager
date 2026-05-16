use std::fs;
use std::path::PathBuf;

use domain::OperationContext;
use repo_scanner::scanner::{scan_repositories, ScannerConfig};
use storage::event_store::{EventStore, NewOperationEvent};
use storage::repo_store::RepositoryStore;
use storage::resource_store::ResourceStore;
use storage::Database;

/// Create a minimal real Git repo at the given path with an initial commit.
fn init_git_repo(path: &PathBuf) {
    fs::create_dir_all(path).unwrap();
    let repo = git2::Repository::init(path).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    let file_path = path.join("README.md");
    fs::write(&file_path, "# Test").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
}

/// Create Claude Code skill files in the repo.
fn add_claude_skill(repo_path: &PathBuf, name: &str, content: &str) {
    let skill_dir = repo_path.join(".claude").join("skills").join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

#[test]
fn event_store_insert_and_list() {
    let db = Database::open_in_memory().unwrap();
    let store = EventStore::new(db.conn());

    let event = store
        .insert_event(NewOperationEvent {
            operation_id: "op-insert-list".to_string(),
            operation_type: "test".to_string(),
            status: "success".to_string(),
            repo_id: None,
            pack_id: None,
            migration_run_id: None,
            summary: Some("Test event".to_string()),
            detail_json: None,
        })
        .unwrap();

    assert!(!event.id.is_empty(), "id should be a non-empty UUID");
    assert!(!event.created_at.is_empty(), "created_at should be set");
    assert_eq!(event.operation_id, "op-insert-list");
    assert_eq!(event.operation_type, "test");
    assert_eq!(event.status, "success");

    let events = store.list_events(10, 0, None).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].operation_id, "op-insert-list");
}

#[test]
fn event_store_respects_type_filter() {
    let db = Database::open_in_memory().unwrap();
    let store = EventStore::new(db.conn());

    store
        .insert_event(NewOperationEvent {
            operation_id: "op-1".to_string(),
            operation_type: "scan".to_string(),
            status: "success".to_string(),
            repo_id: None,
            pack_id: None,
            migration_run_id: None,
            summary: None,
            detail_json: None,
        })
        .unwrap();

    store
        .insert_event(NewOperationEvent {
            operation_id: "op-2".to_string(),
            operation_type: "export".to_string(),
            status: "failure".to_string(),
            repo_id: None,
            pack_id: Some("pack-1".to_string()),
            migration_run_id: None,
            summary: None,
            detail_json: None,
        })
        .unwrap();

    let scan_events = store.list_events(10, 0, Some("scan")).unwrap();
    assert_eq!(scan_events.len(), 1);
    assert_eq!(scan_events[0].operation_type, "scan");

    let export_events = store.list_events(10, 0, Some("export")).unwrap();
    assert_eq!(export_events.len(), 1);
    assert_eq!(export_events[0].operation_type, "export");

    let all = store.list_events(10, 0, None).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn event_store_pagination() {
    let db = Database::open_in_memory().unwrap();
    let store = EventStore::new(db.conn());

    for i in 0..10 {
        store
            .insert_event(NewOperationEvent {
                operation_id: format!("op-{}", i),
                operation_type: "test".to_string(),
                status: "success".to_string(),
                repo_id: None,
                pack_id: None,
                migration_run_id: None,
                summary: None,
                detail_json: None,
            })
            .unwrap();
    }

    let page1 = store.list_events(3, 0, None).unwrap();
    assert_eq!(page1.len(), 3);

    let page2 = store.list_events(3, 3, None).unwrap();
    assert_eq!(page2.len(), 3);
    // page2 should not overlap with page1 (different operation_id values)
    let page1_ids: Vec<&str> = page1.iter().map(|e| e.operation_id.as_str()).collect();
    for event in &page2 {
        assert!(!page1_ids.contains(&event.operation_id.as_str()));
    }

    let page4 = store.list_events(3, 9, None).unwrap();
    assert_eq!(page4.len(), 1);
}

#[test]
fn event_store_get_by_operation_returns_empty_for_unknown() {
    let db = Database::open_in_memory().unwrap();
    let store = EventStore::new(db.conn());

    let result = store.get_by_operation("non-existent").unwrap();
    assert!(result.is_empty());
}

#[test]
fn event_store_get_by_operation_returns_matching_events() {
    let db = Database::open_in_memory().unwrap();
    let store = EventStore::new(db.conn());

    store
        .insert_event(NewOperationEvent {
            operation_id: "op-shared".to_string(),
            operation_type: "scan".to_string(),
            status: "success".to_string(),
            repo_id: None,
            pack_id: None,
            migration_run_id: None,
            summary: None,
            detail_json: None,
        })
        .unwrap();

    store
        .insert_event(NewOperationEvent {
            operation_id: "op-shared".to_string(),
            operation_type: "parse".to_string(),
            status: "failure".to_string(),
            repo_id: Some("repo-1".to_string()),
            pack_id: None,
            migration_run_id: None,
            summary: None,
            detail_json: None,
        })
        .unwrap();

    let events = store.get_by_operation("op-shared").unwrap();
    assert_eq!(events.len(), 2);
    // Most recent first
    assert_eq!(events[0].operation_type, "parse");
}

#[test]
fn scan_pipeline_propagates_operation_id() {
    // Create fixture repos
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Repo A: real git repo with a skill
    let repo_a = base.join("repo-a");
    init_git_repo(&repo_a);
    add_claude_skill(&repo_a, "code-review", "# Code Review\n\nReview code thoroughly.\n");

    // Repo B: real git repo, no capabilities
    let repo_b = base.join("repo-b");
    init_git_repo(&repo_b);

    // Scan to discover repos
    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };
    let dir_count = config.root_paths.len();
    let repos = scan_repositories(config).unwrap();
    assert_eq!(repos.len(), 2);

    // Sort for deterministic ordering
    let mut repos = repos;
    repos.sort_by(|a, b| a.name.cmp(&b.name));

    // Simulate bridge layer: create OperationContext, store repos, parse caps, record event
    let ctx = OperationContext::new("scan_repositories");
    let db = Database::open_in_memory().unwrap();
    let repo_store = RepositoryStore::new(&db);
    let event_store = EventStore::new(db.conn());

    for repo in &repos {
        let stored = repo_store.upsert_by_path(repo).unwrap();

        // Parse capabilities
        match claude_parser::parse_repo(&repo.path) {
            Ok(inventory) => {
                let resource_store = ResourceStore::new(&db);
                let all_resources = collect_resources(&stored.id, &inventory);
                resource_store.replace_for_repo(&stored.id, &all_resources).unwrap();
                repo_store.update_index_status(&stored.id, "fresh", None).unwrap();
            }
            Err(e) => {
                repo_store.update_index_status(&stored.id, "parse_failed", Some(&e.to_string())).unwrap();
            }
        }
    }

    // Record the operation event (same pattern as repo_commands.rs)
    let summary = format!("Scanned {} directories, found {} repos", dir_count, repos.len(),);
    let recorded = event_store
        .insert_event(NewOperationEvent {
            operation_id: ctx.operation_id.clone(),
            operation_type: "scan_repositories".to_string(),
            status: "success".to_string(),
            repo_id: None,
            pack_id: None,
            migration_run_id: None,
            summary: Some(summary),
            detail_json: None,
        })
        .unwrap();

    // Verify the operation_id is propagated
    assert_eq!(recorded.operation_id, ctx.operation_id);
    assert_eq!(recorded.operation_type, "scan_repositories");
    assert_eq!(recorded.status, "success");
    assert!(recorded.summary.unwrap().contains("2 repos"));

    // Verify we can retrieve by operation_id
    let by_op = event_store.get_by_operation(&ctx.operation_id).unwrap();
    assert_eq!(by_op.len(), 1);
    assert_eq!(by_op[0].operation_type, "scan_repositories");

    // Verify list includes our event
    let all_events = event_store.list_events(10, 0, None).unwrap();
    assert!(all_events.iter().any(|e| e.operation_id == ctx.operation_id));
}

fn collect_resources(repo_id: &str, inv: &claude_parser::CapabilityInventory) -> Vec<domain::CapabilityResource> {
    let mut all = Vec::new();
    for mut r in inv.skills.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.mcp.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.hooks.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.rules.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.agents.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.commands.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.plugins.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    for mut r in inv.settings.iter().cloned() {
        r.repo_id = Some(repo_id.to_string());
        all.push(r);
    }
    all
}
