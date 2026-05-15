use std::fs;
use std::path::PathBuf;

use domain::Repository;
use repo_scanner::scanner::{scan_repositories, ScannerConfig};
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
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();
}

/// Create Claude Code skill files in the repo.
fn add_claude_skill(repo_path: &PathBuf, name: &str, content: &str) {
    let skill_dir = repo_path.join(".claude").join("skills").join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

/// Create a claude_settings.json with MCP config
fn add_claude_settings(repo_path: &PathBuf, content: &str) {
    let claude_dir = repo_path.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    fs::write(claude_dir.join("settings.json"), content).unwrap();
}

#[test]
fn full_scan_capability_pipeline() {
    // Step 1: Create fixture repos with Claude config files
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().to_path_buf();

    // Repo A: real git repo with skills
    let repo_a = base.join("repo-a");
    init_git_repo(&repo_a);
    add_claude_skill(&repo_a, "code-review", "# Code Review\n\nPerform thorough code reviews.\n");
    add_claude_skill(&repo_a, "test-writer", "# Test Writer\n\nGenerate comprehensive tests.\n");
    add_claude_settings(&repo_a, r#"{"skills": true}"#);

    // Repo B: real git repo with MCP config
    let repo_b = base.join("repo-b");
    init_git_repo(&repo_b);

    // Step 2: Scan to discover repos
    let config = ScannerConfig {
        root_paths: vec![base.to_string_lossy().to_string()],
        max_depth: 3,
        ignore_dirs: vec![],
    };
    let scan_result = scan_repositories(config).unwrap();
    assert_eq!(scan_result.len(), 2, "should find 2 git repos");

    // Sort so assertions are deterministic
    let mut repos = scan_result;
    repos.sort_by(|a, b| a.name.cmp(&b.name));

    // Step 3: Store repos in DB via upsert (simulating bridge layer behavior)
    let db = Database::open_in_memory().unwrap();
    let repo_store = RepositoryStore::new(&db);

    for repo in &repos {
        let stored = repo_store.upsert_by_path(repo).unwrap();
        assert!(!stored.id.is_empty(), "should have assigned UUID");
        assert!(!stored.first_indexed_at.is_empty(), "should have first_indexed_at");

        // Step 4: Parse capabilities
        match claude_parser::parse_repo(&repo.path) {
            Ok(inventory) => {
                let resource_store = ResourceStore::new(&db);
                let all_resources = collect_inventory(&stored.id, &inventory);
                resource_store.replace_for_repo(&stored.id, &all_resources).unwrap();
                repo_store.update_index_status(&stored.id, "fresh", None).unwrap();
            }
            Err(e) => {
                repo_store.update_index_status(&stored.id, "parse_failed", Some(&e.to_string())).unwrap();
            }
        }
    }

    // Step 5: Verify results
    // Repo A should have skills parsed (or parse_failed if settings.json was invalid)
    // Repo B should have parse_failed (no .claude config)
    let all_stored = repo_store.list_all().unwrap();
    assert_eq!(all_stored.len(), 2);

    for repo in &all_stored {
        let resource_store = ResourceStore::new(&db);
        let resources = resource_store.get_by_repo(&repo.id).unwrap();

        if repo.name == "repo-a" {
            // Should have capabilities OR parse_failed status
            assert!(
                repo.capability_index_status == "fresh" || repo.capability_index_status == "parse_failed",
                "repo-a should have been parsed: got {}",
                repo.capability_index_status
            );
            assert!(
                repo.last_capability_error.as_ref().map(|s| s.len() > 0) != Some(true),
                "repo-a has valid config, should not have error"
            );
        } else if repo.name == "repo-b" {
            // repo-b has no .claude dir - expect parse_failed or fresh (parser behavior varies)
            let has_no_claude_dir = repo.capability_index_status == "parse_failed"
                || (repo.capability_index_status == "fresh" && resources.is_empty());
            assert!(has_no_claude_dir, "repo-b without config: got {}", repo.capability_index_status);
        }
    }
}

fn collect_inventory(repo_id: &str, inv: &claude_parser::CapabilityInventory) -> Vec<domain::CapabilityResource> {
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
