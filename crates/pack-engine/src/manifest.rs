use serde::{Deserialize, Serialize};

/// Represents a complete pack manifest (pack.manifest.json).
/// Schema version: 1.0
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "packType")]
    pub pack_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ManifestSource>,
    pub resources: Vec<ManifestResource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<EnvPlaceholder>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ManifestValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<ResourceDependency>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceDependency {
    #[serde(rename = "type")]
    pub dep_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvPlaceholder {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
}

/// Builder input: metadata for creating a manifest.
#[derive(Debug, Clone)]
pub struct ManifestInput {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub pack_type: Option<String>,
    pub source_repo: Option<String>,
    pub source_commit: Option<String>,
    pub resources: Vec<ManifestResource>,
    pub env_placeholders: Vec<EnvPlaceholder>,
    pub validation_rules: Vec<String>,
}

impl Manifest {
    /// Build a Manifest from structured input with validation.
    pub fn build(input: ManifestInput) -> Result<Manifest, String> {
        if input.name.is_empty() {
            return Err("Manifest name cannot be empty".to_string());
        }
        if input.name.len() > 128 {
            return Err("Manifest name exceeds 128 characters".to_string());
        }
        if !is_valid_semver(&input.version) {
            return Err(format!(
                "Invalid semantic version format: {}",
                input.version
            ));
        }
        if input.resources.is_empty() {
            return Err("Manifest must contain at least one resource".to_string());
        }
        if let Some(ref desc) = input.description {
            if desc.len() > 1024 {
                return Err("Description exceeds 1024 characters".to_string());
            }
        }

        let source = if input.source_repo.is_some() || input.source_commit.is_some() {
            Some(ManifestSource {
                repo: input.source_repo,
                commit: input.source_commit,
            })
        } else {
            None
        };

        let env = if input.env_placeholders.is_empty() {
            None
        } else {
            Some(input.env_placeholders)
        };

        let validation = if input.validation_rules.is_empty() {
            None
        } else {
            Some(ManifestValidation {
                rules: Some(input.validation_rules),
            })
        };

        Ok(Manifest {
            schema_version: "1.0".to_string(),
            name: input.name,
            version: input.version,
            description: input.description,
            pack_type: input.pack_type,
            source,
            resources: input.resources,
            env,
            validation,
        })
    }

    /// Serialize manifest to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize manifest from JSON string.
    pub fn from_json(json: &str) -> Result<Manifest, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Basic semver validation: MAJOR.MINOR.PATCH with optional -PRERELEASE
fn is_valid_semver(version: &str) -> bool {
    let semver_re = regex_lite::Regex::new(
        r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$"
    )
    .unwrap();
    semver_re.is_match(version)
}

// Use a simple semver check without adding a regex dependency.
// Re-implement with manual parsing to avoid extra crate.
mod regex_lite {
    pub struct Regex;

    impl Regex {
        pub fn new(_pattern: &str) -> Result<Self, String> {
            Ok(Regex)
        }

        pub fn is_match(&self, text: &str) -> bool {
            // Manual semver parsing: MAJOR.MINOR.PATCH[-PRERELEASE]
            let (base, pre) = if let Some(idx) = text.find('-') {
                (&text[..idx], Some(&text[idx + 1..]))
            } else {
                (text, None)
            };

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
                if !pre_str
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.')
                {
                    return false;
                }
            }

            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_valid() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.0"));
        assert!(is_valid_semver("10.20.30"));
        assert!(is_valid_semver("1.0.0-alpha"));
        assert!(is_valid_semver("1.0.0-alpha.1"));
        assert!(is_valid_semver("1.0.0-beta.2"));
    }

    #[test]
    fn test_semver_invalid() {
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("v1.0.0"));
        assert!(!is_valid_semver("1.0.0-"));
        assert!(!is_valid_semver("abc"));
        assert!(!is_valid_semver(""));
        assert!(!is_valid_semver("1.0.0-alpha@"));
    }
}
