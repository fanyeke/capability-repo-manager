use domain::redact_sensitive;

#[test]
fn test_redact_token() {
    let result = redact_sensitive("token=ghp_abc123");
    assert_eq!(result.output, "token=[REDACTED]");
    assert!(result.redacted_count >= 1);
}

#[test]
fn test_redact_api_key() {
    let result = redact_sensitive("api_key=sk-proj-test-key");
    assert_eq!(result.output, "api_key=[REDACTED]");
}

#[test]
fn test_redact_api_key_camelcase() {
    let result = redact_sensitive("apiKey=sk-proj-test-key");
    assert_eq!(result.output, "apiKey=[REDACTED]");
}

#[test]
fn test_redact_bearer_authorization() {
    let result = redact_sensitive("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9");
    assert_eq!(result.output, "Authorization: Bearer [REDACTED]");
}

#[test]
fn test_redact_secret() {
    let result = redact_sensitive("secret=mysecretpassword");
    assert_eq!(result.output, "secret=[REDACTED]");
}

#[test]
fn test_redact_env_var_style() {
    let result = redact_sensitive("GITHUB_TOKEN=ghp_xxx");
    assert_eq!(result.output, "GITHUB_TOKEN=[REDACTED]");
}

#[test]
fn test_non_sensitive_string_unchanged() {
    let result = redact_sensitive("path=/home/user/repo");
    assert_eq!(result.output, "path=/home/user/repo");
    assert_eq!(result.redacted_count, 0);
}

#[test]
fn test_multiple_patterns() {
    let result = redact_sensitive("token=abc123 and api_key=secret-key and path=/safe");
    assert_eq!(result.output, "token=[REDACTED] and api_key=[REDACTED] and path=/safe");
    assert!(result.redacted_count >= 2);
}

#[test]
fn test_empty_string() {
    let result = redact_sensitive("");
    assert_eq!(result.output, "");
    assert_eq!(result.redacted_count, 0);
}

#[test]
fn test_no_sensitive_content() {
    let result = redact_sensitive("just a normal log line");
    assert_eq!(result.output, "just a normal log line");
}

#[test]
fn test_generic_env_var() {
    let result = redact_sensitive("MY_CUSTOM_VAR=supersecret");
    assert_eq!(result.output, "MY_CUSTOM_VAR=[REDACTED]");
    assert_eq!(result.redacted_count, 1);
}