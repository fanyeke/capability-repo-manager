pub mod scanner;

use domain::AppError;
pub use scanner::{scan_repositories, ScannerConfig};

/// Result of a scan operation, containing discovered repos and any non-fatal errors
pub struct ScanResult {
    pub repos: Vec<domain::Repository>,
    pub errors: Vec<ScanError>,
}

pub struct ScanError {
    pub repo_path: String,
    pub message: String,
}

/// The RepoCatalog orchestrates repository discovery and metadata enrichment.
///
/// It first uses the scanner to find Git repositories, then enriches each
/// discovered repo with Git metadata (branch, commit, dirty state, remote URL)
/// via the git-service crate.
pub struct RepoCatalog;

impl RepoCatalog {
    /// Scan for repositories using the given config, enriching each with Git metadata.
    ///
    /// Returns a `ScanResult` containing successfully processed repos and any
    /// non-fatal errors encountered during metadata extraction.
    pub fn scan(config: ScannerConfig) -> Result<ScanResult, AppError> {
        let mut repos = scan_repositories(config)?;
        let mut errors = Vec::new();

        for repo in &mut repos {
            // Set canonical_path from the (already canonical) path
            repo.canonical_path = repo.path.clone();

            // Set last_indexed_at timestamp
            repo.last_indexed_at = chrono::Utc::now().to_rfc3339();

            // Enrich with Git metadata
            match git_service::git_cli::extract_metadata(repo) {
                Ok(()) => {
                    log::debug!("Extracted metadata for repo: {}", repo.name);
                }
                Err(e) => {
                    log::warn!("Failed to extract git metadata for {}: {}", repo.name, e);
                    errors.push(ScanError {
                        repo_path: repo.path.clone(),
                        message: format!("Git metadata extraction failed: {}", e),
                    });
                    // Keep the repo even if metadata extraction fails
                }
            }
        }

        Ok(ScanResult { repos, errors })
    }

    /// Refresh Git metadata for all previously indexed repositories.
    ///
    /// This is used at startup to bring stale metadata up to date without
    /// re-scanning the filesystem for new repos. Returns the list of
    /// refreshed repos and any non-fatal errors.
    pub fn refresh_all(repos: &mut [domain::Repository]) -> Vec<ScanError> {
        let mut errors = Vec::new();

        for repo in repos.iter_mut() {
            // Ensure canonical_path is set (existing DB records may have NULL)
            if repo.canonical_path.is_empty() {
                repo.canonical_path = repo.path.clone();
            }
            repo.last_indexed_at = chrono::Utc::now().to_rfc3339();

            match git_service::git_cli::extract_metadata(repo) {
                Ok(()) => {
                    log::debug!("Refreshed metadata for repo: {}", repo.name);
                }
                Err(e) => {
                    log::warn!("Failed to refresh metadata for {}: {}", repo.name, e);
                    errors.push(ScanError {
                        repo_path: repo.path.clone(),
                        message: format!("Git metadata refresh failed: {}", e),
                    });
                }
            }
        }

        errors
    }
}
