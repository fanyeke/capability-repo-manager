//! Sensitive data redaction utility.
//!
//! Regex-based pattern matching for tokens, API keys, bearer auth, secrets, and
//! generic KEY=VALUE env-var assignments. Designed to be used before writing
//! log output so sensitive values never appear in log files.

/// The result of a redaction operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedactResult {
    /// The original input string.
    pub input: String,
    /// The redacted output string.
    pub output: String,
    /// Number of redacted values.
    pub redacted_count: usize,
}

/// Redact sensitive patterns from a string, replacing matched values with `[REDACTED]`.
///
/// Currently redacts:
/// - `token=<value>` → `token=[REDACTED]`
/// - `api_key=<value>` or `apiKey=<value>` → `api_key=[REDACTED]`
/// - `Authorization: Bearer <value>` → `Authorization: Bearer [REDACTED]`
/// - `secret=<value>` → `secret=[REDACTED]`
/// - Known env-var patterns (`GITHUB_TOKEN=`, `ANTHROPIC_API_KEY=`, etc.)
/// - Generic `ALL_CAPS_KEY=value` env-var-style assignments
///
/// Returns a `RedactResult` containing the original input, redacted output,
/// and a count of how many values were redacted.
pub fn redact_sensitive(input: &str) -> RedactResult {
    // Token patterns: `token=<value>`, `api_key=<value>`, `apiKey=<value>`, `secret=<value>`
    let token_pattern = regex_lite::Regex::new(
        r"(?i)(token|api_key|apiKey|secret)\s*=\s*\S+"
    ).expect("valid token pattern");

    // Bearer authorization pattern
    let bearer_pattern = regex_lite::Regex::new(
        r"(?i)(Authorization:\s*Bearer\s+)\S+"
    ).expect("valid bearer pattern");

    // Known env-var-style assignments: GITHUB_TOKEN=..., ANTHROPIC_API_KEY=..., etc.
    let known_env_var = regex_lite::Regex::new(
        r"(?i)((?:GITHUB|ANTHROPIC|OPENAI|AWS|AZURE|GOOGLE)_[A-Z_]+)\s*=\s*\S+"
    ).expect("valid known env var pattern");

    // Generic env-var-style pattern: any UPPERCASE_KEY=value not caught above
    // Uppercase-only (no (?i) flag) to avoid matching lowercase words like "path"
    let generic_env_var = regex_lite::Regex::new(
        r"\b([A-Z][A-Z_0-9]{2,})\s*=\s*\S+"
    ).expect("valid generic env var pattern");

    let mut result = input.to_string();
    result = token_pattern.replace_all(&result, "${1}=[REDACTED]").to_string();
    result = bearer_pattern.replace_all(&result, "${1}[REDACTED]").to_string();
    result = known_env_var.replace_all(&result, "${1}=[REDACTED]").to_string();
    result = generic_env_var.replace_all(&result, "${1}=[REDACTED]").to_string();

    let redacted_count = result.matches("[REDACTED]").count();

    RedactResult {
        input: input.to_string(),
        output: result,
        redacted_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_token() {
        let result = redact_sensitive("token=ghp_abc123");
        assert_eq!(result.output, "token=[REDACTED]");
        assert_eq!(result.redacted_count, 1);
    }

    #[test]
    fn test_redact_token_in_sentence() {
        assert_eq!(
            redact_sensitive("using token=ghp_abc123 in request").output,
            "using token=[REDACTED] in request"
        );
    }

    #[test]
    fn test_redact_api_key() {
        assert_eq!(
            redact_sensitive("api_key=sk-proj-test-key").output,
            "api_key=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_api_key_camelcase() {
        assert_eq!(
            redact_sensitive("apiKey=sk-proj-test-key").output,
            "apiKey=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_bearer_authorization() {
        assert_eq!(
            redact_sensitive("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9").output,
            "Authorization: Bearer [REDACTED]"
        );
    }

    #[test]
    fn test_redact_secret() {
        assert_eq!(
            redact_sensitive("secret=mysecretpassword").output,
            "secret=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_github_token_env_var() {
        assert_eq!(
            redact_sensitive("GITHUB_TOKEN=ghp_xxx").output,
            "GITHUB_TOKEN=[REDACTED]"
        );
    }

    #[test]
    fn test_non_sensitive_string_unchanged() {
        let result = redact_sensitive("path=/home/user/repo");
        assert_eq!(result.output, "path=/home/user/repo");
        assert_eq!(result.redacted_count, 0);
    }

    #[test]
    fn test_multiple_patterns_in_one_string() {
        let result = redact_sensitive("token=abc123 and api_key=secret-key and path=/safe");
        assert_eq!(result.output, "token=[REDACTED] and api_key=[REDACTED] and path=/safe");
        assert_eq!(result.redacted_count, 2);
    }

    #[test]
    fn test_empty_string() {
        let result = redact_sensitive("");
        assert_eq!(result.output, "");
        assert_eq!(result.redacted_count, 0);
    }

    #[test]
    fn test_no_sensitive_content() {
        assert_eq!(
            redact_sensitive("just a normal log line with path=/tmp/foo").output,
            "just a normal log line with path=/tmp/foo"
        );
    }

    #[test]
    fn test_generic_env_var() {
        let result = redact_sensitive("MY_CUSTOM_SECRET=supersecretvalue");
        assert_eq!(result.output, "MY_CUSTOM_SECRET=[REDACTED]");
        assert_eq!(result.redacted_count, 1);
    }

    #[test]
    fn test_redact_result_counts() {
        let result = redact_sensitive("token=a and GITHUB_TOKEN=b and api_key=c and safe_path=/tmp");
        assert_eq!(result.redacted_count, 3);
        assert!(result.output.contains("[REDACTED]"));
    }
}