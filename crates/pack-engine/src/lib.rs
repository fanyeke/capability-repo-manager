pub mod library;
pub mod manifest;
pub mod packer;
pub mod validator;

use std::path::{Path, PathBuf};

use domain::{CapabilityPack, CapabilityResource};

use crate::library::PackStore;
use crate::manifest::{EnvPlaceholder, Manifest, ManifestInput, ManifestResource};
use crate::packer::{detect_env_placeholders, PackResourceInput};
use crate::validator::validate_pack;

/// Minimum required disk space for pack and migration operations (50 MB).
pub const MIN_DISK_SPACE: u64 = 50 * 1024 * 1024;

/// Check that the filesystem containing `path` has at least `min_bytes` available.
///
/// Uses the system `df` command on Linux to query available space. If the check
/// cannot be performed (e.g., platform not supported), it returns Ok(()) since
/// the operation can proceed with a best-effort basis.
pub fn check_disk_space(path: &Path, min_bytes: u64) -> Result<(), String> {
    match std::process::Command::new("df")
        .arg("-k")
        .arg(path)
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let last_line = stdout.lines().last().unwrap_or("");
            let fields: Vec<&str> = last_line.split_whitespace().collect();
            if fields.len() >= 4 {
                if let Ok(avail_kb) = fields[3].parse::<u64>() {
                    let avail_bytes = avail_kb * 1024;
                    if avail_bytes < min_bytes {
                        let mb = min_bytes / (1024 * 1024);
                        return Err(format!(
                            "Insufficient disk space: {} MB required but only {} MB available on {}",
                            mb,
                            avail_bytes / (1024 * 1024),
                            path.display()
                        ));
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()), // Can't check; proceed optimistically
    }
}

/// High-level input for exporting a capability pack from a repository.
#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub pack_name: String,
    pub version: String,
    pub description: Option<String>,
    pub pack_type: String,
    pub source_repo_id: String,
    pub source_commit: Option<String>,
    pub source_repo_path: PathBuf,
    pub selected_resources: Vec<CapabilityResource>,
    pub output_base_dir: PathBuf,
}

/// Result of a successful pack export.
#[derive(Debug)]
pub struct ExportResult {
    pub pack: CapabilityPack,
    pub manifest: Manifest,
    pub pack_dir: PathBuf,
    pub warnings: Vec<String>,
}

/// Orchestrates the full pack export flow:
/// 1. Validate selection (must have at least 1 resource)
/// 2. Build manifest
/// 3. Detect env placeholders in resource files
/// 4. Pack files into directory structure
/// 5. Validate the resulting pack
/// 6. Add to pack library
pub fn export_pack(
    request: ExportRequest,
    library: &mut PackStore,
) -> Result<ExportResult, String> {
    let mut warnings: Vec<String> = Vec::new();

    // Step 0: Check available disk space
    if let Err(e) = check_disk_space(&request.output_base_dir, MIN_DISK_SPACE) {
        return Err(e);
    }

    // Step 1: Validate selection
    if request.selected_resources.is_empty() {
        return Err("Cannot export pack: no resources selected".to_string());
    }

    // Step 2: Build manifest resources from selected capabilities
    let mut manifest_resources: Vec<ManifestResource> = Vec::new();
    let mut pack_inputs: Vec<PackResourceInput> = Vec::new();
    let mut env_placeholders: Vec<EnvPlaceholder> = Vec::new();
    let mut seen_env_vars: std::collections::HashSet<String> = std::collections::HashSet::new();

    for resource in &request.selected_resources {
        let source_rel = resource
            .source_path
            .as_ref()
            .map(|p| p.clone())
            .unwrap_or_else(|| format!("{}/resource", resource.name));

        // Compute content hash
        let source_abs = request.source_repo_path.join(&source_rel);
        let hash = if source_abs.exists() {
            match packer::compute_file_hash(&source_abs) {
                Ok(h) => Some(h),
                Err(e) => {
                    warnings.push(format!(
                        "Warning: could not hash {}: {}",
                        resource.name, e
                    ));
                    None
                }
            }
        } else {
            warnings.push(format!(
                "Warning: resource file not found: {}",
                source_abs.display()
            ));
            None
        };

        // Detect env placeholders in resource content
        if source_abs.exists() {
            if let Ok(content) = std::fs::read_to_string(&source_abs) {
                for var in detect_env_placeholders(&content) {
                    if seen_env_vars.insert(var.clone()) {
                        env_placeholders.push(EnvPlaceholder {
                            name: var,
                            required: None,
                            description: None,
                        });
                    }
                }
            }
        }

        // Determine destination path based on resource type
        let relative_dest = determine_pack_path(&resource.r#type, &resource.name, &source_rel);

        manifest_resources.push(ManifestResource {
            resource_type: resource.r#type.clone(),
            name: resource.name.clone(),
            source: relative_dest
                .to_str()
                .unwrap_or(&source_rel)
                .to_string(),
            hash,
            dependencies: None,
        });

        pack_inputs.push(PackResourceInput {
            source_path: source_abs,
            relative_dest,
        });
    }

    // Step 3: Build manifest
    let manifest = Manifest::build(ManifestInput {
        name: request.pack_name.clone(),
        version: request.version.clone(),
        description: request.description.clone(),
        pack_type: Some(request.pack_type.clone()),
        source_repo: Some(request.source_repo_path.display().to_string()),
        source_commit: request.source_commit.clone(),
        resources: manifest_resources,
        env_placeholders,
        validation_rules: vec![],
    })?;

    // Step 4: Pack files into directory
    let pack_result = packer::create_pack(&request.output_base_dir, &pack_inputs, &manifest)?;

    // Step 5: Validate the created pack
    let validation = validate_pack(&pack_result.pack_dir, &manifest);
    if !validation.valid {
        for err in &validation.errors {
            warnings.push(format!("Validation: {}", err.message));
        }
    }

    // Step 6: Create CapabilityPack record and add to library
    let pack_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let capability_pack = CapabilityPack {
        id: pack_id,
        name: request.pack_name,
        version: request.version,
        description: request.description,
        pack_type: request.pack_type,
        manifest_path: pack_result
            .pack_dir
            .join("pack.manifest.json")
            .display()
            .to_string(),
        source_repo_id: Some(request.source_repo_id),
        source_commit: request.source_commit,
        created_at: now,
        storage_dir: pack_result.pack_dir.display().to_string(),
    };

    library.insert(capability_pack.clone())?;

    Ok(ExportResult {
        pack: capability_pack,
        manifest,
        pack_dir: pack_result.pack_dir,
        warnings,
    })
}

/// Determine the pack-relative destination path for a resource based on its type.
fn determine_pack_path(resource_type: &str, name: &str, source_path: &str) -> PathBuf {
    match resource_type {
        "skill" => PathBuf::from("resources").join("skills").join(name).join(
            Path::new(source_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("SKILL.md"),
        ),
        "mcp" => PathBuf::from("mcp").join(
            Path::new(source_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("mcp.json"),
        ),
        "hook" | "rule" | "agent" | "command" => PathBuf::from("resources").join(
            Path::new(source_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("file.md"),
        ),
        "settings" => PathBuf::from("settings").join(
            Path::new(source_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("settings.json"),
        ),
        _ => PathBuf::from("resources").join(
            Path::new(source_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("resource"),
        ),
    }
}

/// Validate a pack on disk. Wraps the validator module.
pub fn validate_existing_pack(pack_dir: &Path, manifest: &Manifest) -> validator::ValidationResult {
    validate_pack(pack_dir, manifest)
}

/// Build a manifest from input (used by tauri-bridge and for testing).
pub fn build_manifest(input: ManifestInput) -> Result<Manifest, String> {
    Manifest::build(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::CapabilityResource;
    use std::fs;

    #[test]
    fn test_export_pack_full_flow() {
        let temp = tempfile::TempDir::new().unwrap();
        let repo_dir = temp.path().join("repo");
        let output_dir = temp.path().join("packs");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create a dummy skill file in the repo
        let skill_dir = repo_dir.join("skills").join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# My Skill\nAPI_KEY=${MY_SECRET}").unwrap();

        let resource = CapabilityResource {
            id: "r1".to_string(),
            repo_id: Some("repo-1".to_string()),
            pack_id: None,
            r#type: "skill".to_string(),
            name: "my-skill".to_string(),
            source_path: Some("skills/my-skill/SKILL.md".to_string()),
            scope: "project".to_string(),
            tracked_by_git: true,
            content_hash: None,
            metadata_json: None,
            error_message: None,
        };

        let request = ExportRequest {
            pack_name: "test-pack".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test pack".to_string()),
            pack_type: "project".to_string(),
            source_repo_id: "repo-1".to_string(),
            source_commit: Some("abc123".to_string()),
            source_repo_path: repo_dir,
            selected_resources: vec![resource],
            output_base_dir: output_dir.clone(),
        };

        let mut library = PackStore::new(output_dir);
        let result = export_pack(request, &mut library).unwrap();

        assert_eq!(result.pack.name, "test-pack");
        assert_eq!(result.pack.version, "1.0.0");
        assert!(result.pack_dir.exists());
        assert!(result.pack_dir.join("pack.manifest.json").exists());
        assert!(result
            .pack_dir
            .join("resources/skills/my-skill/SKILL.md")
            .exists());
        assert_eq!(library.len(), 1);

        // Verify manifest contains env placeholder detection
        let manifest_json = fs::read_to_string(result.pack_dir.join("pack.manifest.json")).unwrap();
        assert!(manifest_json.contains("MY_SECRET"));
    }

    #[test]
    fn test_export_pack_empty_selection_fails() {
        let temp = tempfile::TempDir::new().unwrap();
        let output_dir = temp.path().join("packs");
        fs::create_dir_all(&output_dir).unwrap();

        let request = ExportRequest {
            pack_name: "empty-pack".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            pack_type: "project".to_string(),
            source_repo_id: "repo-1".to_string(),
            source_commit: None,
            source_repo_path: temp.path().to_path_buf(),
            selected_resources: vec![],
            output_base_dir: output_dir.clone(),
        };

        let mut library = PackStore::new(output_dir);
        let result = export_pack(request, &mut library);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no resources"));
    }

    #[test]
    fn test_export_pack_invalid_version_fails() {
        let temp = tempfile::TempDir::new().unwrap();
        let repo_dir = temp.path().join("repo");
        let output_dir = temp.path().join("packs");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        let resource = CapabilityResource {
            id: "r1".to_string(),
            repo_id: Some("repo-1".to_string()),
            pack_id: None,
            r#type: "skill".to_string(),
            name: "my-skill".to_string(),
            source_path: Some("skills/my-skill/SKILL.md".to_string()),
            scope: "project".to_string(),
            tracked_by_git: true,
            content_hash: None,
            metadata_json: None,
            error_message: None,
        };

        let request = ExportRequest {
            pack_name: "test-pack".to_string(),
            version: "not.valid".to_string(),
            description: None,
            pack_type: "project".to_string(),
            source_repo_id: "repo-1".to_string(),
            source_commit: None,
            source_repo_path: repo_dir,
            selected_resources: vec![resource],
            output_base_dir: output_dir.clone(),
        };

        let mut library = PackStore::new(output_dir);
        let result = export_pack(request, &mut library);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("version"));
    }

    #[test]
    fn test_export_pack_duplicate_name_version_rejected() {
        let temp = tempfile::TempDir::new().unwrap();
        let repo_dir = temp.path().join("repo");
        let output_dir = temp.path().join("packs");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        fs::write(repo_dir.join("test.md"), "# test").unwrap();

        let resource = CapabilityResource {
            id: "r1".to_string(),
            repo_id: Some("repo-1".to_string()),
            pack_id: None,
            r#type: "rule".to_string(),
            name: "test-resource".to_string(),
            source_path: Some("test.md".to_string()),
            scope: "project".to_string(),
            tracked_by_git: true,
            content_hash: None,
            metadata_json: None,
            error_message: None,
        };

        let make_request = || ExportRequest {
            pack_name: "dup-pack".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            pack_type: "project".to_string(),
            source_repo_id: "repo-1".to_string(),
            source_commit: None,
            source_repo_path: repo_dir.clone(),
            selected_resources: vec![resource.clone()],
            output_base_dir: output_dir.clone(),
        };

        let mut library = PackStore::new(output_dir.clone());
        export_pack(make_request(), &mut library).unwrap();

        // Second export with same name+version should fail
        let result = export_pack(make_request(), &mut library);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }
}
