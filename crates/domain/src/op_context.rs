use crate::error::AppError;

/// Tracks the context of a user-initiated operation through the system.
///
/// Created at the Tauri command entry point and propagated through all
/// engine and storage calls so log entries carry a shared operation_id.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationContext {
    pub operation_id: String,
    pub operation_type: String,
    pub repo_id: Option<String>,
    pub pack_id: Option<String>,
    pub migration_run_id: Option<String>,
}

impl OperationContext {
    /// Create a new OperationContext with a generated UUID and the given operation_type.
    pub fn new(operation_type: impl Into<String>) -> Self {
        Self {
            operation_id: uuid::Uuid::new_v4().to_string(),
            operation_type: operation_type.into(),
            repo_id: None,
            pack_id: None,
            migration_run_id: None,
        }
    }

    /// Builder: set the repo_id.
    pub fn with_repo_id(mut self, repo_id: impl Into<String>) -> Self {
        self.repo_id = Some(repo_id.into());
        self
    }

    /// Builder: set the pack_id.
    pub fn with_pack_id(mut self, pack_id: impl Into<String>) -> Self {
        self.pack_id = Some(pack_id.into());
        self
    }

    /// Builder: set the migration_run_id.
    pub fn with_migration_run_id(mut self, migration_run_id: impl Into<String>) -> Self {
        self.migration_run_id = Some(migration_run_id.into());
        self
    }

    /// Validate that the operation_id is a non-empty UUID.
    pub fn validate(&self) -> Result<(), AppError> {
        if self.operation_id.is_empty() {
            return Err(AppError::Validation("operation_id must not be empty".into()));
        }
        if self.operation_type.is_empty() {
            return Err(AppError::Validation("operation_type must not be empty".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_valid_operation_context() {
        let ctx = OperationContext::new("scan_repositories");
        assert!(!ctx.operation_id.is_empty());
        assert_eq!(ctx.operation_type, "scan_repositories");
        assert!(ctx.repo_id.is_none());
        assert!(ctx.pack_id.is_none());
        assert!(ctx.migration_run_id.is_none());
    }

    #[test]
    fn test_with_repo_id() {
        let ctx = OperationContext::new("scan_repositories").with_repo_id("repo-123");
        assert_eq!(ctx.repo_id, Some("repo-123".to_string()));
    }

    #[test]
    fn test_with_pack_id() {
        let ctx = OperationContext::new("export_pack").with_pack_id("pack-456");
        assert_eq!(ctx.pack_id, Some("pack-456".to_string()));
    }

    #[test]
    fn test_with_migration_run_id() {
        let ctx = OperationContext::new("apply_migration").with_migration_run_id("run-789");
        assert_eq!(ctx.migration_run_id, Some("run-789".to_string()));
    }

    #[test]
    fn test_validate_valid_context() {
        let ctx = OperationContext::new("scan_repositories");
        assert!(ctx.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_on_empty_operation_id() {
        let ctx = OperationContext {
            operation_id: "".to_string(),
            ..OperationContext::new("scan")
        };
        assert!(ctx.validate().is_err());
    }

    #[test]
    fn test_validate_fails_on_empty_operation_type() {
        let ctx = OperationContext {
            operation_type: "".to_string(),
            ..OperationContext::new("")
        };
        assert!(ctx.validate().is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let ctx = OperationContext::new("scan_repositories")
            .with_repo_id("repo-123")
            .with_pack_id("pack-456");
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: OperationContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.operation_type, "scan_repositories");
        assert_eq!(deserialized.repo_id, Some("repo-123".to_string()));
        assert_eq!(deserialized.pack_id, Some("pack-456".to_string()));
    }
}
