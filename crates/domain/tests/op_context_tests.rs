use domain::OperationContext;

#[test]
fn test_new_operation_context() {
    let ctx = OperationContext::new("scan_repositories");
    assert!(!ctx.operation_id.is_empty());
    assert_eq!(ctx.operation_type, "scan_repositories");
    assert!(ctx.repo_id.is_none());
    assert!(ctx.pack_id.is_none());
    assert!(ctx.migration_run_id.is_none());
}

#[test]
fn test_builder_pattern() {
    let ctx = OperationContext::new("export_pack")
        .with_repo_id("repo-123")
        .with_pack_id("pack-456")
        .with_migration_run_id("run-789");

    assert_eq!(ctx.repo_id, Some("repo-123".to_string()));
    assert_eq!(ctx.pack_id, Some("pack-456".to_string()));
    assert_eq!(ctx.migration_run_id, Some("run-789".to_string()));
}

#[test]
fn test_chained_operations() {
    let ctx = OperationContext::new("apply_migration")
        .with_repo_id("repo-abc")
        .with_migration_run_id("run-xyz");

    assert_eq!(ctx.operation_type, "apply_migration");
    assert_eq!(ctx.repo_id, Some("repo-abc".to_string()));
    assert_eq!(ctx.migration_run_id, Some("run-xyz".to_string()));
    assert!(ctx.pack_id.is_none());
}

#[test]
fn test_validation() {
    let ctx = OperationContext::new("scan_repositories");
    assert!(ctx.validate().is_ok());
}

#[test]
fn test_unique_operation_ids() {
    let ctx1 = OperationContext::new("scan");
    let ctx2 = OperationContext::new("scan");
    assert_ne!(ctx1.operation_id, ctx2.operation_id);
}

#[test]
fn test_serialization_roundtrip() {
    let ctx = OperationContext::new("run_doctor")
        .with_repo_id("repo-1");
    let json = serde_json::to_string(&ctx).unwrap();
    let deserialized: OperationContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.operation_type, "run_doctor");
    assert_eq!(deserialized.repo_id, Some("repo-1".to_string()));
    assert_eq!(deserialized.pack_id, None);
}
