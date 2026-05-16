use domain::Repository;

pub fn extract_metadata(repo: &mut Repository) -> Result<(), domain::AppError> {
    let repo_path = std::path::Path::new(&repo.path);
    let git_repo = git2::Repository::open(repo_path)
        .map_err(|e| domain::AppError::Git(format!("Failed to open git repo at {}: {}", repo.path, e)))?;

    // Branch name
    if let Ok(head) = git_repo.head() {
        if head.is_branch() {
            repo.current_branch = Some(head.shorthand().unwrap_or("HEAD").to_string());
        } else {
            repo.current_branch = Some("HEAD (detached)".to_string());
        }

        // HEAD commit SHA
        if let Some(oid) = head.target() {
            repo.head_commit = Some(oid.to_string());
        }
    }

    // Remote URL
    if let Ok(remote) = git_repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            repo.remote_url = Some(url.to_string());
        }
    }

    // Dirty state detection via status
    let dirty = check_dirty_state(&git_repo)?;
    repo.dirty_state = dirty;

    Ok(())
}

fn check_dirty_state(repo: &git2::Repository) -> Result<String, domain::AppError> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true);

    let statuses =
        repo.statuses(Some(&mut opts)).map_err(|e| domain::AppError::Git(format!("Failed to get status: {}", e)))?;

    for status in statuses.iter() {
        let flags = status.status();
        if flags.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE
                | git2::Status::WT_NEW
                | git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        ) && !flags.contains(git2::Status::IGNORED)
        {
            return Ok("modified".to_string());
        }
    }

    Ok("clean".to_string())
}
