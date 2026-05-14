use domain::{DoctorIssue, DoctorReport};

fn make_issue(severity: &str, code: &str, message: &str) -> DoctorIssue {
    DoctorIssue {
        severity: severity.to_string(),
        code: code.to_string(),
        message: message.to_string(),
        resource_ref: None,
        recommendation: None,
    }
}

#[test]
fn test_perfect_score_no_issues() {
    let score = DoctorReport::compute_score(&[]);
    assert_eq!(score, 100);
}

#[test]
fn test_critical_deduction() {
    let issues = vec![
        make_issue("critical", "E001", "Hook script not found"),
        make_issue("critical", "E002", "MCP config broken"),
    ];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 60); // 100 - 2*20
}

#[test]
fn test_warning_deduction() {
    let issues = vec![
        make_issue("warning", "W001", "Skill structure incomplete"),
        make_issue("warning", "W002", "Missing optional env var"),
        make_issue("warning", "W003", "Rule format deprecated"),
    ];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 85); // 100 - 3*5
}

#[test]
fn test_info_deduction() {
    let issues = vec![
        make_issue("info", "I001", "Consider adding a README"),
        make_issue("info", "I002", "New version available"),
    ];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 98); // 100 - 2*1
}

#[test]
fn test_mixed_severity_scoring() {
    let issues = vec![
        make_issue("critical", "E001", "Hook script not found"),
        make_issue("critical", "E002", "MCP config broken"),
        make_issue("warning", "W001", "Skill structure incomplete"),
        make_issue("warning", "W002", "Missing optional env var"),
        make_issue("warning", "W003", "Rule format deprecated"),
        make_issue("info", "I001", "Consider adding a README"),
        make_issue("info", "I002", "New version available"),
        make_issue("info", "I003", "Update agent description"),
        make_issue("info", "I004", "Add pack description"),
        make_issue("info", "I005", "Consider Doctor check"),
    ];
    let score = DoctorReport::compute_score(&issues);
    // 100 - 2*20 - 3*5 - 5*1 = 100 - 40 - 15 - 5 = 40
    assert_eq!(score, 40);
}

#[test]
fn test_score_floor_at_zero() {
    let issues = vec![make_issue("critical", "E", "error"); 10];
    let score = DoctorReport::compute_score(&issues);
    // 100 - 10*20 = -100, but floor is 0
    assert_eq!(score, 0);
}

#[test]
fn test_doctor_report_serde() {
    let report = DoctorReport {
        id: "r1".to_string(),
        repo_id: "repo1".to_string(),
        score: 85,
        issues: vec![
            DoctorIssue {
                severity: "warning".to_string(),
                code: "SKILL_INCOMPLETE".to_string(),
                message: "Skill 'ui-review' missing SKILL.md".to_string(),
                resource_ref: Some("res-123".to_string()),
                recommendation: Some("Add SKILL.md to skills/ui-review/".to_string()),
            },
        ],
        created_at: "2026-05-14T10:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&report).expect("serialize");
    let deserialized: DoctorReport = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.score, 85);
    assert_eq!(deserialized.issues.len(), 1);
    assert_eq!(deserialized.issues[0].code, "SKILL_INCOMPLETE");
}

#[test]
fn test_unknown_severity_ignored() {
    let issues = vec![
        make_issue("critical", "E001", "error"),
        DoctorIssue {
            severity: "unknown".to_string(),
            code: "X".to_string(),
            message: "mystery".to_string(),
            resource_ref: None,
            recommendation: None,
        },
    ];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 80); // only critical deducted
}
