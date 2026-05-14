use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::manifest::Manifest;

/// Result of a pack operation: the pack directory path and the written manifest.
#[derive(Debug)]
pub struct PackResult {
    pub pack_dir: PathBuf,
    pub manifest: Manifest,
}

/// Copy selected resource files into a pack directory structure and write the manifest.
pub fn create_pack(
    base_dir: &Path,
    resources: &[PackResourceInput],
    manifest: &Manifest,
) -> Result<PackResult, String> {
    let pack_dir = base_dir.join(&manifest.name);
    if pack_dir.exists() {
        return Err(format!(
            "Pack directory already exists: {}",
            pack_dir.display()
        ));
    }

    fs::create_dir_all(&pack_dir).map_err(|e| format!("Failed to create pack directory: {}", e))?;

    for resource in resources {
        let dest = pack_dir.join(&resource.relative_dest);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        fs::copy(&resource.source_path, &dest).map_err(|e| {
            format!(
                "Failed to copy {} -> {}: {}",
                resource.source_path.display(),
                dest.display(),
                e
            )
        })?;
    }

    let manifest_path = pack_dir.join("pack.manifest.json");
    let manifest_json = manifest
        .to_json()
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    fs::write(&manifest_path, manifest_json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(PackResult {
        pack_dir,
        manifest: manifest.clone(),
    })
}

/// Input for a single resource to be packed.
#[derive(Debug, Clone)]
pub struct PackResourceInput {
    pub source_path: PathBuf,
    pub relative_dest: PathBuf,
}

/// Compute SHA256 hash of a file's contents.
pub fn compute_file_hash(path: &Path) -> Result<String, String> {
    let contents = fs::read(path)
        .map_err(|e| format!("Failed to read {} for hashing: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Detect environment variable placeholders (${VAR_NAME}) in a file's contents.
pub fn detect_env_placeholders(content: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '}' {
                    chars.next(); // consume '}'
                    break;
                }
                var_name.push(nc);
                chars.next();
            }
            if !var_name.is_empty()
                && var_name
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
            {
                placeholders.push(var_name);
            }
        }
    }

    placeholders
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_manifest() -> Manifest {
        Manifest::build(crate::manifest::ManifestInput {
            name: "test-pack".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test pack".to_string()),
            pack_type: Some("project".to_string()),
            source_repo: Some("/test/repo".to_string()),
            source_commit: Some("abc123".to_string()),
            resources: vec![crate::manifest::ManifestResource {
                resource_type: "skill".to_string(),
                name: "test-skill".to_string(),
                source: "resources/skills/test-skill/SKILL.md".to_string(),
                hash: None,
                dependencies: None,
            }],
            env_placeholders: vec![],
            validation_rules: vec![],
        })
        .unwrap()
    }

    #[test]
    fn test_create_pack_writes_manifest() {
        let temp = TempDir::new().unwrap();
        let manifest = setup_manifest();

        // Create a dummy source file to pack
        let src_file = temp.path().join("SKILL.md");
        fs::write(&src_file, "# Test Skill").unwrap();

        let resources = vec![PackResourceInput {
            source_path: src_file,
            relative_dest: PathBuf::from("resources/skills/test-skill/SKILL.md"),
        }];

        let result = create_pack(temp.path(), &resources, &manifest).unwrap();
        assert!(result.pack_dir.exists());
        assert!(result.pack_dir.join("pack.manifest.json").exists());
        assert!(result
            .pack_dir
            .join("resources/skills/test-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn test_create_pack_duplicate_dir_rejected() {
        let temp = TempDir::new().unwrap();
        let manifest = setup_manifest();
        let pack_dir = temp.path().join(&manifest.name);
        fs::create_dir_all(&pack_dir).unwrap();

        let result = create_pack(temp.path(), &[], &manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_compute_file_hash() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        fs::write(&file, "hello world").unwrap();

        let hash = compute_file_hash(&file).unwrap();
        // SHA256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_detect_env_placeholders() {
        let content = "api_key: ${API_KEY}\ndb_url: ${DATABASE_URL}\nno_var here";
        let vars = detect_env_placeholders(content);
        assert_eq!(vars, vec!["API_KEY", "DATABASE_URL"]);
    }

    #[test]
    fn test_detect_env_placeholders_none() {
        let content = "no variables here";
        let vars = detect_env_placeholders(content);
        assert!(vars.is_empty());
    }
}
