use std::fs;
use std::path::Path;

use crate::manifest::Manifest;
use crate::packer;

/// Individual validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub resource_ref: Option<String>,
    pub message: String,
}

/// Result of a pack validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn success() -> Self {
        ValidationResult { valid: true, errors: vec![] }
    }

    pub fn failure(errors: Vec<ValidationError>) -> Self {
        ValidationResult { valid: false, errors }
    }
}

/// Validate a pack manifest for schema conformance.
pub fn validate_schema(manifest: &Manifest) -> ValidationResult {
    let mut errors = Vec::new();

    if manifest.schema_version != "1.0" {
        errors.push(ValidationError {
            resource_ref: None,
            message: format!("Invalid schemaVersion: '{}'. Only '1.0' is supported.", manifest.schema_version),
        });
    }

    if manifest.name.is_empty() {
        errors.push(ValidationError { resource_ref: None, message: "Manifest name is empty".to_string() });
    }
    if manifest.name.len() > 128 {
        errors.push(ValidationError {
            resource_ref: None,
            message: format!(
                "Manifest name '{}' exceeds 128 characters ({} chars)",
                manifest.name,
                manifest.name.len()
            ),
        });
    }

    if !is_valid_semver(&manifest.version) {
        errors.push(ValidationError {
            resource_ref: None,
            message: format!("Invalid version format: '{}'. Expected MAJOR.MINOR.PATCH[-PRERELEASE]", manifest.version),
        });
    }

    if let Some(ref desc) = manifest.description {
        if desc.len() > 1024 {
            errors.push(ValidationError {
                resource_ref: None,
                message: format!("Description exceeds 1024 characters ({} chars)", desc.len()),
            });
        }
    }

    if let Some(ref pack_type) = manifest.pack_type {
        if !["project", "blueprint", "baseline"].contains(&pack_type.as_str()) {
            errors.push(ValidationError {
                resource_ref: None,
                message: format!("Invalid packType: '{}'. Must be one of: project, blueprint, baseline", pack_type),
            });
        }
    }

    if manifest.resources.is_empty() {
        errors.push(ValidationError {
            resource_ref: None,
            message: "Manifest has no resources (minItems: 1)".to_string(),
        });
    }

    for resource in &manifest.resources {
        if resource.resource_type.is_empty() {
            errors.push(ValidationError {
                resource_ref: Some(resource.name.clone()),
                message: format!("Resource '{}': type is empty", resource.name),
            });
        }
        if resource.name.is_empty() {
            errors.push(ValidationError { resource_ref: None, message: "Resource has empty name".to_string() });
        }
        if resource.source.is_empty() {
            errors.push(ValidationError {
                resource_ref: Some(resource.name.clone()),
                message: format!("Resource '{}': source path is empty", resource.name),
            });
        }

        let valid_types = ["skill", "mcp", "hook", "rule", "agent", "command", "plugin", "settings", "contextDoc"];
        if !valid_types.contains(&resource.resource_type.as_str()) {
            errors.push(ValidationError {
                resource_ref: Some(resource.name.clone()),
                message: format!("Resource '{}': invalid type '{}'", resource.name, resource.resource_type),
            });
        }
    }

    if let Some(ref env_vars) = manifest.env {
        for env_var in env_vars {
            if env_var.name.is_empty() {
                errors.push(ValidationError {
                    resource_ref: None,
                    message: "Env placeholder has empty name".to_string(),
                });
            }
        }
    }

    if let Some(ref validation) = manifest.validation {
        if let Some(ref rules) = validation.rules {
            let valid_rules = [
                "skill-structure-valid",
                "hook-target-exists",
                "mcp-config-parseable",
                "rule-format-valid",
                "env-placeholders-resolved",
            ];
            for rule in rules {
                if !valid_rules.contains(&rule.as_str()) {
                    errors.push(ValidationError {
                        resource_ref: None,
                        message: format!("Invalid validation rule: '{}'", rule),
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        ValidationResult::success()
    } else {
        ValidationResult::failure(errors)
    }
}

/// Validate that all resources referenced in the manifest exist on disk.
pub fn validate_file_existence(pack_dir: &Path, manifest: &Manifest) -> ValidationResult {
    let mut errors = Vec::new();

    for resource in &manifest.resources {
        let file_path = pack_dir.join(&resource.source);
        if !file_path.exists() {
            errors.push(ValidationError {
                resource_ref: Some(resource.name.clone()),
                message: format!("Resource '{}': file not found at '{}'", resource.name, file_path.display()),
            });
        }
    }

    if errors.is_empty() {
        ValidationResult::success()
    } else {
        ValidationResult::failure(errors)
    }
}

/// Validate SHA256 hash integrity for all resources in the manifest.
pub fn validate_hash_integrity(pack_dir: &Path, manifest: &Manifest) -> ValidationResult {
    let mut errors = Vec::new();

    for resource in &manifest.resources {
        let file_path = pack_dir.join(&resource.source);
        if !file_path.exists() {
            continue; // File existence checked separately
        }

        if let Some(ref expected_hash) = resource.hash {
            match packer::compute_file_hash(&file_path) {
                Ok(actual_hash) => {
                    if actual_hash != *expected_hash {
                        errors.push(ValidationError {
                            resource_ref: Some(resource.name.clone()),
                            message: format!(
                                "Resource '{}': hash mismatch. Expected: {}, Actual: {}",
                                resource.name, expected_hash, actual_hash
                            ),
                        });
                    }
                }
                Err(e) => {
                    errors.push(ValidationError {
                        resource_ref: Some(resource.name.clone()),
                        message: format!("Resource '{}': failed to compute hash: {}", resource.name, e),
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        ValidationResult::success()
    } else {
        ValidationResult::failure(errors)
    }
}

/// Check for unresolved environment variable placeholders in resource files.
pub fn validate_env_placeholders(pack_dir: &Path, manifest: &Manifest) -> ValidationResult {
    let mut errors = Vec::new();

    for resource in &manifest.resources {
        let file_path = pack_dir.join(&resource.source);
        if !file_path.exists() {
            continue;
        }

        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let placeholders = packer::detect_env_placeholders(&content);
                if !placeholders.is_empty() {
                    errors.push(ValidationError {
                        resource_ref: Some(resource.name.clone()),
                        message: format!(
                            "Resource '{}' contains unresolved env placeholders: {}",
                            resource.name,
                            placeholders.join(", ")
                        ),
                    });
                }
            }
            Err(e) => {
                errors.push(ValidationError {
                    resource_ref: Some(resource.name.clone()),
                    message: format!("Resource '{}': failed to read for env check: {}", resource.name, e),
                });
            }
        }
    }

    if errors.is_empty() {
        ValidationResult::success()
    } else {
        ValidationResult::failure(errors)
    }
}

/// Run all validators and aggregate results.
pub fn validate_pack(pack_dir: &Path, manifest: &Manifest) -> ValidationResult {
    let mut all_errors = Vec::new();

    let schema_result = validate_schema(manifest);
    all_errors.extend(schema_result.errors);

    let file_result = validate_file_existence(pack_dir, manifest);
    all_errors.extend(file_result.errors);

    let hash_result = validate_hash_integrity(pack_dir, manifest);
    all_errors.extend(hash_result.errors);

    let env_result = validate_env_placeholders(pack_dir, manifest);
    all_errors.extend(env_result.errors);

    if all_errors.is_empty() {
        ValidationResult::success()
    } else {
        ValidationResult::failure(all_errors)
    }
}

fn is_valid_semver(version: &str) -> bool {
    let (base, pre) =
        if let Some(idx) = version.find('-') { (&version[..idx], Some(&version[idx + 1..])) } else { (version, None) };

    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    for part in &parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }

    if let Some(pre_str) = pre {
        if pre_str.is_empty() {
            return false;
        }
        if !pre_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '.') {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, ManifestInput, ManifestResource};

    fn make_manifest(name: &str, version: &str, resources: Vec<ManifestResource>) -> Manifest {
        Manifest::build(ManifestInput {
            name: name.to_string(),
            version: version.to_string(),
            description: None,
            pack_type: None,
            source_repo: None,
            source_commit: None,
            resources,
            env_placeholders: vec![],
            validation_rules: vec![],
        })
        .unwrap()
    }

    fn make_resource(resource_type: &str, name: &str, source: &str) -> ManifestResource {
        ManifestResource {
            resource_type: resource_type.to_string(),
            name: name.to_string(),
            source: source.to_string(),
            hash: None,
            dependencies: None,
        }
    }

    #[test]
    fn test_validate_schema_valid_manifest() {
        let m = make_manifest("test", "1.0.0", vec![make_resource("skill", "my-skill", "skills/my-skill/SKILL.md")]);
        let result = validate_schema(&m);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_schema_missing_name_fails() {
        let mut m = make_manifest("test", "1.0.0", vec![make_resource("skill", "my-skill", "path.md")]);
        // Hack to test: we bypass build() validation to test schema validation directly
        m.name = String::new();
        let result = validate_schema(&m);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.message.contains("name is empty")));
    }

    #[test]
    fn test_validate_schema_invalid_version_fails() {
        let mut m = make_manifest("test", "1.0.0", vec![make_resource("skill", "my-skill", "path.md")]);
        m.version = "bad-version".to_string();
        let result = validate_schema(&m);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.message.contains("Invalid version")));
    }

    #[test]
    fn test_validate_schema_empty_resources_fails() {
        let m = Manifest {
            schema_version: "1.0".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            pack_type: None,
            source: None,
            resources: vec![],
            env: None,
            validation: None,
        };
        let result = validate_schema(&m);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.message.contains("no resources")));
    }

    #[test]
    fn test_validate_schema_invalid_resource_type() {
        let mut m = make_manifest("test", "1.0.0", vec![make_resource("invalid_type", "rsrc", "path.md")]);
        // Fix the resource to have invalid type directly
        m.resources[0].resource_type = "invalid_type".to_string();
        let result = validate_schema(&m);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.message.contains("invalid type")));
    }

    #[test]
    fn test_validate_schema_invalid_pack_type() {
        let mut m = make_manifest("test", "1.0.0", vec![make_resource("skill", "rsrc", "path.md")]);
        m.pack_type = Some("invalid".to_string());
        let result = validate_schema(&m);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.message.contains("Invalid packType")));
    }
}
