use domain::Repository;
use git_service::git_cli::extract_metadata;
use std::fs;

fn make_repo(name: &str) -> (tempfile::TempDir, Repository) {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path().join(name);
    fs::create_dir_all(&repo_path).unwrap();

    // Initialize a real git repo
    let git_repo = git2::Repository::init(&repo_path).unwrap();

    // Set up git user config for the test repo (needed for commits)
    let mut config = git_repo.config().unwrap();
    config.set_str("user.name", "test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    let repo = Repository::new(
        String::new(),
        name.to_string(),
        repo_path.to_string_lossy().to_string(),
    );

    (tmp, repo)
}

fn commit_file(repo_path: &std::path::Path, filename: &str, content: &str) {
    let git_repo = git2::Repository::open(repo_path).unwrap();
    let file_path = repo_path.join(filename);
    fs::write(&file_path, content).unwrap();

    let mut index = git_repo.index().unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();

    let sig = git2::Signature::now("test", "test@test.com").unwrap();
    let head = git_repo.head().ok();
    let commit = if let Some(ref head_ref) = head {
        Some(git_repo.find_commit(head_ref.target().unwrap()).unwrap())
    } else {
        None
    };
    let parents: Vec<&git2::Commit> = commit.iter().collect();

    git_repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("commit: {}", filename),
            &tree,
            &parents,
        )
        .unwrap();
}

#[test]
fn extracts_branch_name() {
    let (_tmp, mut repo) = make_repo("branch-test");
    commit_file(&std::path::Path::new(&repo.path), "readme.md", "# Test");

    extract_metadata(&mut repo).unwrap();

    assert!(repo.current_branch.is_some());
    // New repos default to main or master depending on git config
    let branch = repo.current_branch.as_ref().unwrap();
    assert!(branch == "main" || branch == "master");
}

#[test]
fn extracts_head_commit_sha() {
    let (_tmp, mut repo) = make_repo("commit-test");
    commit_file(&std::path::Path::new(&repo.path), "file.txt", "content");

    extract_metadata(&mut repo).unwrap();

    assert!(repo.head_commit.is_some());
    assert_eq!(repo.head_commit.as_ref().unwrap().len(), 40); // Full SHA
}

#[test]
fn detects_clean_working_tree() {
    let (_tmp, mut repo) = make_repo("clean-test");
    commit_file(&std::path::Path::new(&repo.path), "src.txt", "hello");

    extract_metadata(&mut repo).unwrap();

    assert_eq!(repo.dirty_state, "clean");
}

#[test]
fn detects_modified_working_tree() {
    let (_tmp, mut repo) = make_repo("dirty-test");
    let repo_path = std::path::Path::new(&repo.path);
    commit_file(repo_path, "config.txt", "original");

    // Modify the file without committing
    fs::write(repo_path.join("config.txt"), "modified").unwrap();

    extract_metadata(&mut repo).unwrap();

    assert_eq!(repo.dirty_state, "modified");
}

#[test]
fn detects_untracked_files_as_dirty() {
    let (_tmp, mut repo) = make_repo("untracked-test");
    let repo_path = std::path::Path::new(&repo.path);
    commit_file(repo_path, "tracked.txt", "tracked");

    // Create an untracked file
    fs::write(repo_path.join("untracked.txt"), "untracked").unwrap();

    extract_metadata(&mut repo).unwrap();

    assert_eq!(repo.dirty_state, "modified");
}

#[test]
fn handles_repo_without_remote() {
    let (_tmp, mut repo) = make_repo("no-remote");
    commit_file(&std::path::Path::new(&repo.path), "f.txt", "data");

    extract_metadata(&mut repo).unwrap();

    assert!(repo.remote_url.is_none());
}

#[test]
fn handles_non_git_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("not-a-repo");
    fs::create_dir_all(&dir).unwrap();

    let mut repo = Repository::new(
        String::new(),
        "not-a-repo".to_string(),
        dir.to_string_lossy().to_string(),
    );

    let result = extract_metadata(&mut repo);
    assert!(result.is_err());
    match result.unwrap_err() {
        domain::AppError::Git(_) => {} // expected
        e => panic!("Expected Git error, got {:?}", e),
    }
}
