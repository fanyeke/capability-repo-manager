/// Sensitive data redaction utility.
///
/// Regex-based pattern matching for tokens, API keys, bearer auth, and secrets.
/// Designed to be used before writing log output so sensitive values never
/// appear in log files.

/// Redact sensitive patterns from a string, replacing matched values with `[REDACTED]`.
///
/// Currently redacts:
/// - `token=<alphanumeric value>` → `token=[REDACTED]`
/// - `api_key=<value>` or `apiKey=<value>` → `api_key=[REDACTED]`
/// - `Authorization: Bearer <value>` → `Authorization: Bearer [REDACTED]`
/// - `secret=<value>` → `secret=[REDACTED]`
/// - `GITHUB_TOKEN=<value>` (env-var-style assignment) → `GITHUB_TOKEN=[REDACTED]`
///
/// Non-sensitive strings pass through unchanged.
pub fn redact_sensitive(input: &str) -> String {
    // Token patterns: `token=<value>`, `api_key=<value>`, `apiKey=<value>`, `secret=<value>`
    let token_pattern = regex_lite::Regex::new(
        r"(?i)(token|api_key|apiKey|secret)\s*=\s*\S+"
    ).expect("valid token pattern");

    // Bearer authorization pattern
    let bearer_pattern = regex_lite::Regex::new(
        r"(?i)(Authorization:\s*Bearer\s+)\S+"
    ).expect("valid bearer pattern");

    // Env-var-style assignments: GITHUB_TOKEN=..., ANTHROPIC_API_KEY=..., etc.
    let env_var_pattern = regex_lite::Regex::new(
        r"(?i)((?:GITHUB|ANTHROPIC|OPENAI|AWS|AZURE|GOOGLE)_[A-Z_]+)\s*=\s*\S+"
    ).expect("valid env var pattern");

    let mut result = input.to_string();
    result = token_pattern.replace_all(&result, "${1}=[REDACTED]").to_string();
    result = bearer_pattern.replace_all(&result, "${1}[REDACTED]").to_string();
    result = env_var_pattern.replace_all(&result, "${1}=[REDACTED]").to_string();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_token() {
        assert_eq!(
            redact_sensitive("token=ghp_abc123"),
            "token=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_token_in_sentence() {
        assert_eq!(
            redact_sensitive("using token=ghp_abc123 in request"),
            "using token=[REDACTED] in request"
        );
    }

    #[test]
    fn test_redact_api_key() {
        assert_eq!(
            redact_sensitive("api_key=sk-proj-test-key"),
            "api_key=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_api_key_camelcase() {
        assert_eq!(
            redact_sensitive("apiKey=sk-proj-test-key"),
            "apiKey=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_bearer_authorization() {
        assert_eq!(
            redact_sensitive("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9"),
            "Authorization: Bearer [REDACTED]"
        );
    }

    #[test]
    fn test_redact_secret() {
        assert_eq!(
            redact_sensitive("secret=mysecretpassword"),
            "secret=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_github_token_env_var() {
        assert_eq!(
            redact_sensitive("GITHUB_TOKEN=ghp_xxx"),
            "GITHUB_TOKEN=[REDACTED]"
        );
    }

    #[test]
    fn test_non_sensitive_string_unchanged() {
        assert_eq!(
            redact_sensitive("path=/home/user/repo"),
            "path=/home/user/repo"
        );
    }

    #[test]
    fn test_multiple_patterns_in_one_string() {
        let input = "token=abc123 and api_key=secret-key and path=/safe";
        let expected = "token=[REDACTED] and api_key=[REDACTED] and path=/safe";
        assert_eq!(redact_sensitive(input), expected);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(redact_sensitive(""), "");
    }

    #[test]
    fn test_no_sensitive_content() {
        assert_eq!(
            redact_sensitive("just a normal log line with path=/tmp/foo"),
            "just a normal log line with path=/tmp/foo"
        );
    }
}
