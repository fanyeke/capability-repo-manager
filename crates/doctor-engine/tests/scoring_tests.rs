use doctor_engine::scoring;
use domain::{DoctorIssue, DoctorReport};

fn issue(severity: &str, code: &str) -> DoctorIssue {
    DoctorIssue {
        severity: severity.to_string(),
        code: code.to_string(),
        message: "test".into(),
        resource_ref: None,
        recommendation: None,
    }
}

#[test]
fn score_perfect_100() {
    let issues: Vec<DoctorIssue> = vec![];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 100);
}

#[test]
fn score_one_critical() {
    let issues = vec![issue("critical", "C1")];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 80); // 100 - 20
}

#[test]
fn score_one_warning() {
    let issues = vec![issue("warning", "W1")];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 95); // 100 - 5
}

#[test]
fn score_one_info() {
    let issues = vec![issue("info", "I1")];
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 99); // 100 - 1
}

#[test]
fn score_mixed_issues() {
    let issues = vec![
        issue("critical", "C1"),
        issue("critical", "C2"),
        issue("warning", "W1"),
        issue("warning", "W2"),
        issue("warning", "W3"),
        issue("info", "I1"),
        issue("info", "I2"),
        issue("info", "I3"),
        issue("info", "I4"),
        issue("info", "I5"),
    ];
    let score = DoctorReport::compute_score(&issues);
    // 2*20 + 3*5 + 5*1 = 40 + 15 + 5 = 60 deduction
    // 100 - 60 = 40
    assert_eq!(score, 40);
}

#[test]
fn score_floor_at_zero() {
    let mut issues = vec![];
    // 10 critical * 20 = 200 deduction → floor at 0
    for i in 0..10 {
        issues.push(issue("critical", &format!("C{}", i)));
    }
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 0, "Score should floor at 0, not go negative");
}

#[test]
fn score_boundary_near_floor() {
    let issues = vec![
        issue("critical", "C1"),
        issue("critical", "C2"),
        issue("critical", "C3"),
        issue("critical", "C4"),
        issue("critical", "C5"),
    ];
    // 5 * 20 = 100 → score = 0
    let score = DoctorReport::compute_score(&issues);
    assert_eq!(score, 0);
}

#[test]
fn score_with_unknown_severity_ignored() {
    let issues = vec![issue("unknown", "U1"), issue("warning", "W1")];
    let score = DoctorReport::compute_score(&issues);
    // Unknown severity ignored; only 5 deducted for warning
    assert_eq!(score, 95);
}

#[test]
fn scoring_module_compute_score_delegates() {
    // Verify the scoring module's compute_score produces same result
    let issues = vec![issue("critical", "C1"), issue("warning", "W1")];
    let score = scoring::compute_score(&issues);
    // 100 - 20 - 5 = 75
    assert_eq!(score, 75);
}
