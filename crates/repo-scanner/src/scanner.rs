use domain::Repository;
use std::path::Path;

const DEFAULT_MAX_DEPTH: usize = 5;
const DEFAULT_IGNORE_DIRS: &[&str] = &["node_modules", ".venv", "vendor", ".cache", "build", "target"];

pub struct ScannerConfig {
    pub root_paths: Vec<String>,
    pub max_depth: usize,
    pub ignore_dirs: Vec<String>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            root_paths: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
            ignore_dirs: DEFAULT_IGNORE_DIRS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

fn is_skipped_dir(name: &str, ignore_dirs: &[String]) -> bool {
    if name.starts_with('.') && name != ".git" {
        return true;
    }
    ignore_dirs.iter().any(|ignored| name == ignored.as_str())
}

/// Check if any path component between root and entry is a skipped directory
fn has_skipped_ancestor(entry_path: &Path, root: &Path, ignore_dirs: &[String]) -> bool {
    for ancestor in entry_path.ancestors() {
        if ancestor == root || ancestor.parent().is_none() {
            break;
        }
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
            if is_skipped_dir(name, ignore_dirs) {
                return true;
            }
        }
    }
    false
}

pub fn scan_repositories(config: ScannerConfig) -> Result<Vec<Repository>, domain::AppError> {
    let mut repos = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for root in &config.root_paths {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(domain::AppError::NotFound(format!("Root path does not exist: {}", root)));
        }
        if !root_path.is_dir() {
            return Err(domain::AppError::Scan(format!("Root path is not a directory: {}", root)));
        }

        let ignore_dirs = &config.ignore_dirs;

        for entry in walkdir::WalkDir::new(root).max_depth(config.max_depth).follow_links(false).into_iter() {
            match entry {
                Ok(entry) => {
                    if has_skipped_ancestor(entry.path(), root_path, ignore_dirs) {
                        continue;
                    }

                    if entry.file_name() == ".git" && (entry.file_type().is_dir() || entry.file_type().is_file()) {
                        let repo_path = entry
                            .path()
                            .parent()
                            .ok_or_else(|| domain::AppError::Scan("Invalid .git path".to_string()))?;

                        let canonical = repo_path.canonicalize().unwrap_or(repo_path.to_path_buf());
                        if !seen_paths.insert(canonical.clone()) {
                            continue;
                        }

                        let path_str = canonical.to_string_lossy().to_string();
                        let name = canonical
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path_str.clone());

                        repos.push(Repository::new(String::new(), name, path_str));
                    }
                }
                Err(e) => {
                    if let Some(inner) = e.io_error() {
                        if inner.kind() == std::io::ErrorKind::PermissionDenied {
                            continue;
                        }
                    }
                    return Err(domain::AppError::Scan(format!("Walk error: {}", e)));
                }
            }
        }
    }

    Ok(repos)
}
