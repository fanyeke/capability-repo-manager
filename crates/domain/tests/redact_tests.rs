use domain::redact_sensitive;

#[test]
fn test_redact_token() {
    assert_eq!(
        redact_sensitive("token=ghp_abc123"),
        "token=[REDACTED]"
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
fn test_redact_env_var_style() {
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
fn test_multiple_patterns() {
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
        redact_sensitive("just a normal log line"),
        "just a normal log line"
    );
}
