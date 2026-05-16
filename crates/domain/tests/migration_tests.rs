use domain::{
    MigrationConflict, MigrationPlan, MigrationPlanItem, MigrationReport, MigrationReportItem, MigrationReportSummary,
    MigrationRun, ResourceDependency,
};

#[test]
fn test_migration_plan_with_conflicts() {
    let plan = MigrationPlan {
        plan_id: "plan-1".to_string(),
        source_type: "pack".to_string(),
        source_id: "pack-1".to_string(),
        target_repo_id: "repo-2".to_string(),
        items: vec![
            MigrationPlanItem {
                resource_id: "res-1".to_string(),
                action: "add".to_string(),
                source_path: Some("resources/skills/ui-review".to_string()),
                target_path: Some(".claude/skills/ui-review".to_string()),
                status: "pending".to_string(),
            },
            MigrationPlanItem {
                resource_id: "res-2".to_string(),
                action: "overwrite".to_string(),
                source_path: Some("resources/hooks/pre-commit".to_string()),
                target_path: Some(".claude/settings.json".to_string()),
                status: "pending".to_string(),
            },
        ],
        conflicts: vec![MigrationConflict {
            resource_name: "pre-commit-hook".to_string(),
            resource_type: "hook".to_string(),
            reason: "same_name".to_string(),
            recommended_actions: vec!["skip".to_string(), "overwrite".to_string()],
        }],
        missing_dependencies: vec![ResourceDependency {
            dep_type: "env".to_string(),
            name: "GITHUB_TOKEN".to_string(),
            required: false,
            status: "missing".to_string(),
        }],
    };

    assert_eq!(plan.items.len(), 2);
    assert_eq!(plan.conflicts.len(), 1);
    assert_eq!(plan.missing_dependencies.len(), 1);
    assert_eq!(plan.items[0].action, "add");
    assert_eq!(plan.items[1].action, "overwrite");
}

#[test]
fn test_migration_run_state_transitions() {
    let run = MigrationRun {
        id: "run-1".to_string(),
        source_type: "pack".to_string(),
        source_id: "pack-1".to_string(),
        target_repo_id: "repo-2".to_string(),
        status: "planned".to_string(),
        plan_json: "{}".to_string(),
        report_json: None,
        snapshot_path: None,
        created_at: "2026-05-14T10:00:00Z".to_string(),
        executed_at: None,
    };

    assert_eq!(run.status, "planned");
    assert!(run.executed_at.is_none());
}

#[test]
fn test_migration_report_summary() {
    let summary = MigrationReportSummary { added: 5, overwritten: 2, skipped: 1, failed: 0 };
    assert_eq!(summary.added + summary.overwritten + summary.skipped + summary.failed, 8);
}

#[test]
fn test_migration_report_with_failures() {
    let report = MigrationReport {
        status: "applied".to_string(),
        items: vec![
            MigrationReportItem {
                resource_name: "skill-a".to_string(),
                action: "add".to_string(),
                status: "applied".to_string(),
                error: None,
            },
            MigrationReportItem {
                resource_name: "hook-b".to_string(),
                action: "overwrite".to_string(),
                status: "failed".to_string(),
                error: Some("Permission denied".to_string()),
            },
        ],
        summary: MigrationReportSummary { added: 1, overwritten: 0, skipped: 0, failed: 1 },
    };

    assert_eq!(report.status, "applied");
    assert_eq!(report.items[1].status, "failed");
    assert!(report.items[1].error.is_some());
}

#[test]
fn test_plan_serde_roundtrip() {
    let plan = MigrationPlan {
        plan_id: "p1".to_string(),
        source_type: "pack".to_string(),
        source_id: "s1".to_string(),
        target_repo_id: "t1".to_string(),
        items: vec![],
        conflicts: vec![],
        missing_dependencies: vec![],
    };

    let json = serde_json::to_string(&plan).expect("serialize");
    let deserialized: MigrationPlan = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.plan_id, plan.plan_id);
    assert_eq!(deserialized.source_type, "pack");
}
