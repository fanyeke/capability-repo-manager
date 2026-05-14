use domain::DoctorIssue;

/// Compute the health score from a collection of doctor issues.
///
/// Formula: score = max(0, 100 - 20*critical - 5*warning - info)
///
/// This is a convenience wrapper around DoctorReport::compute_score
/// for use when a full report hasn't been constructed yet.
pub fn compute_score(issues: &[DoctorIssue]) -> i32 {
    domain::DoctorReport::compute_score(issues)
}
