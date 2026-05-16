use repo_scanner::scanner::{scan_repositories, ScannerConfig};
use std::fs;
use std::path::PathBuf;

/// Create a fake git repo directory structure at the given path
fn create_fake_repo(base: &PathBuf, name: &str) -> PathBuf {
    let repo = base.join(name);
    fs::create_dir_all(repo.join(".git")).unwrap();
    repo
}

#[test]
fn discovers_single_git_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    create_fake_repo(&base, "my-project");

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let repos = scan_repositories(config).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "my-project");
    assert!(repos[0].path.ends_with("my-project"));
    assert_eq!(repos[0].dirty_state, "unknown");
    assert!(repos[0].current_branch.is_none());
}

#[test]
fn discovers_multiple_repos() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    create_fake_repo(&base, "repo-a");
    create_fake_repo(&base, "repo-b");
    create_fake_repo(&base, "repo-c");

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let repos = scan_repositories(config).unwrap();
    assert_eq!(repos.len(), 3);
    let names: Vec<&str> = repos.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"repo-a"));
    assert!(names.contains(&"repo-b"));
    assert!(names.contains(&"repo-c"));
}

#[test]
fn respects_max_depth_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Create a deep directory structure: base/d1/d2/d3/.git
    let deep = base.join("d1").join("d2").join("d3");
    fs::create_dir_all(deep.join(".git")).unwrap();

    // With depth 2, should NOT find the repo at depth 3
    let config_shallow =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 2, ignore_dirs: vec![] };
    let repos = scan_repositories(config_shallow).unwrap();
    assert_eq!(repos.len(), 0);

    // With depth 4, SHOULD find it
    let config_deep =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 4, ignore_dirs: vec![] };
    let repos = scan_repositories(config_deep).unwrap();
    assert_eq!(repos.len(), 1);
}

#[test]
fn skips_ignored_directories() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Create repos inside ignored dirs
    create_fake_repo(&base.join("node_modules"), "pkg-a");
    create_fake_repo(&base.join(".venv"), "pkg-b");
    create_fake_repo(&base.join("vendor"), "pkg-c");

    // Also create a normal repo
    create_fake_repo(&base, "real-repo");

    let config = ScannerConfig {
        root_paths: vec![base.to_string_lossy().to_string()],
        max_depth: 4,
        ignore_dirs: vec!["node_modules".to_string(), ".venv".to_string(), "vendor".to_string()],
    };

    let repos = scan_repositories(config).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "real-repo");
}

#[test]
fn deduplicates_duplicate_canonical_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    create_fake_repo(&base, "my-project");

    // Create a symlink to the same repo inside the scan root
    #[cfg(unix)]
    {
        let link_path = base.join("symlink-project");
        std::os::unix::fs::symlink(base.join("my-project"), &link_path).unwrap();
    }

    // With follow_links(false), symlinks to dirs are not followed, so
    // only the real directory is found. This verifies correct behavior.
    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let repos = scan_repositories(config).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "my-project");
}

#[test]
fn handles_non_existent_root_path() {
    let config =
        ScannerConfig { root_paths: vec!["/nonexistent/path/12345".to_string()], max_depth: 3, ignore_dirs: vec![] };

    let result = scan_repositories(config);
    assert!(result.is_err());
    match result.unwrap_err() {
        domain::AppError::NotFound(_) => {} // expected
        e => panic!("Expected NotFound, got {:?}", e),
    }
}

#[test]
fn handles_root_path_is_file_not_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let file_path = tmp.path().join("not-a-dir.txt");
    fs::write(&file_path, "hello").unwrap();

    let config =
        ScannerConfig { root_paths: vec![file_path.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let result = scan_repositories(config);
    assert!(result.is_err());
    match result.unwrap_err() {
        domain::AppError::Scan(msg) => assert!(msg.contains("not a directory")),
        e => panic!("Expected Scan error, got {:?}", e),
    }
}

#[test]
fn skips_hidden_directories_except_git() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Repo inside .hidden dir should be skipped
    let hidden = base.join(".hidden-dir");
    fs::create_dir_all(hidden.join("secret-repo").join(".git")).unwrap();

    // Normal repo should be found
    create_fake_repo(&base, "visible-repo");

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 4, ignore_dirs: vec![] };

    let repos = scan_repositories(config).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "visible-repo");
}

#[test]
fn empty_root_paths_returns_empty() {
    let config = ScannerConfig { root_paths: vec![], max_depth: 3, ignore_dirs: vec![] };

    let repos = scan_repositories(config).unwrap();
    assert!(repos.is_empty());
}

#[test]
fn empty_directory_with_no_git_repos() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Create some non-git directories
    fs::create_dir_all(base.join("src").join("components")).unwrap();
    fs::create_dir_all(base.join("docs")).unwrap();

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let repos = scan_repositories(config).unwrap();
    assert!(repos.is_empty());
}

#[test]
fn default_ignore_list_includes_common_dirs() {
    let config = ScannerConfig::default();
    assert!(config.ignore_dirs.contains(&"node_modules".to_string()));
    assert!(config.ignore_dirs.contains(&".venv".to_string()));
    assert!(config.ignore_dirs.contains(&"vendor".to_string()));
    assert!(config.ignore_dirs.contains(&".cache".to_string()));
    assert!(config.ignore_dirs.contains(&"build".to_string()));
    assert!(config.ignore_dirs.contains(&"target".to_string()));
    assert_eq!(config.max_depth, 5);
    assert!(config.root_paths.is_empty());
}

// RepoCatalog integration tests below

fn make_real_repo(dir: &PathBuf, name: &str) -> PathBuf {
    let repo_path = dir.join(name);
    fs::create_dir_all(&repo_path).unwrap();
    let git_repo = git2::Repository::init(&repo_path).unwrap();

    let mut config = git_repo.config().unwrap();
    config.set_str("user.name", "test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    // Create initial commit so HEAD exists
    let file_path = repo_path.join("README.md");
    fs::write(&file_path, "# Test Repo").unwrap();
    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    git_repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[]).unwrap();

    repo_path
}

#[test]
fn repo_catalog_scan_enriches_with_uuid_and_timestamp() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();
    make_real_repo(&base, "enrich-test");

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let result = repo_scanner::RepoCatalog::scan(config).unwrap();
    assert_eq!(result.repos.len(), 1);
    assert!(result.errors.is_empty());

    let repo = &result.repos[0];
    // UUID is assigned by the store layer (upsert_by_path), RepoCatalog no longer generates it
    assert!(repo.id.is_empty(), "UUID should be empty at scan stage");
    assert!(!repo.last_indexed_at.is_empty(), "timestamp should be set");
    assert_eq!(repo.dirty_state, "clean", "fresh repo should be clean");
    assert!(repo.head_commit.is_some(), "commit SHA should be set");
    assert!(repo.current_branch.is_some(), "branch should be set");
}

#[test]
fn repo_catalog_scan_handles_non_git_directory_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Create a fake repo without real git (empty .git dir, not a real git repo)
    fs::create_dir_all(base.join("fake-repo").join(".git")).unwrap();

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let result = repo_scanner::RepoCatalog::scan(config).unwrap();
    // The repo IS discovered (it has .git dir) but metadata extraction fails
    assert_eq!(result.repos.len(), 1);
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].message.contains("Git metadata extraction failed"));
    // Repo should still be included with defaults
    assert_eq!(result.repos[0].name, "fake-repo");
    assert_eq!(result.repos[0].dirty_state, "unknown");
}

#[test]
fn repo_catalog_scan_sets_canonical_path() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();
    make_real_repo(&base, "canonical-test");

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let result = repo_scanner::RepoCatalog::scan(config).unwrap();
    assert_eq!(result.repos.len(), 1);

    let repo = &result.repos[0];
    assert!(!repo.canonical_path.is_empty(), "canonical_path should be set");
    assert_eq!(repo.canonical_path, repo.path, "canonical_path should match path (already canonical)");
}

#[test]
fn repo_catalog_scan_identity_stable_across_calls() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();
    make_real_repo(&base, "stable-id");

    let cfg1 =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let cfg2 =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let r1 = repo_scanner::RepoCatalog::scan(cfg1).unwrap();
    assert_eq!(r1.repos.len(), 1);

    let r2 = repo_scanner::RepoCatalog::scan(cfg2).unwrap();
    assert_eq!(r2.repos.len(), 1);

    assert_eq!(r1.repos[0].path, r2.repos[0].path, "paths should match across scans");
    assert_eq!(r1.repos[0].canonical_path, r2.repos[0].canonical_path, "canonical_paths should match across scans");
}

#[test]
fn discovers_git_worktree_with_dotgit_file() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Create a real git repo (the main repo)
    let main_repo = make_real_repo(&base, "main-repo");

    // Create a worktree checkout: has a .git FILE (not directory) pointing to the main repo
    let worktree = base.join("worktree");
    fs::create_dir_all(&worktree).unwrap();
    let gitdir_path = main_repo.join(".git");
    fs::write(worktree.join(".git"), format!("gitdir: {}", gitdir_path.display())).unwrap();

    let config =
        ScannerConfig { root_paths: vec![base.to_string_lossy().to_string()], max_depth: 3, ignore_dirs: vec![] };

    let repos = scan_repositories(config).unwrap();
    let worktree_found = repos.iter().any(|r| r.name == "worktree");
    assert!(worktree_found, "worktree with .git file should be discovered");
}
