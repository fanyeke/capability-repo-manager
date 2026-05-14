use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub id: String,
    pub repo_id: String,
    pub score: i32,
    pub issues: Vec<DoctorIssue>,
    pub created_at: String,
}

impl DoctorReport {
    pub fn compute_score(issues: &[DoctorIssue]) -> i32 {
        let mut critical = 0i32;
        let mut warning = 0i32;
        let mut info = 0i32;

        for issue in issues {
            match issue.severity.as_str() {
                "critical" => critical += 1,
                "warning" => warning += 1,
                "info" => info += 1,
                _ => {}
            }
        }

        let deduction = 20 * critical + 5 * warning + info;
        let score = 100i32.saturating_sub(deduction);
        score.max(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorIssue {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub resource_ref: Option<String>,
    pub recommendation: Option<String>,
}
