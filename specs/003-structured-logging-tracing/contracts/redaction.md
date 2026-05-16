# Contract: Sensitive Data Redaction

**Purpose**: Define which patterns are redacted from log output and how.

## Redacted Patterns

| Pattern | Example | Replacement |
|---------|---------|-------------|
| `token=...` (alphanumeric after =) | `token=ghp_abc123` | `token=[REDACTED]` |
| `api_key=...` or `apiKey=...` | `api_key=sk-proj-xxx` | `api_key=[REDACTED]` |
| `Authorization: Bearer ...` | `Authorization: Bearer eyJ...` | `Authorization: Bearer [REDACTED]` |
| `secret=...` | `secret=my-password` | `secret=[REDACTED]` |
| Environment variable values | Any `.env` file value | Variable name only, no value |

## Redaction Function

```rust
pub fn redact_sensitive(input: &str) -> String {
    // Apply all redaction patterns via regex
    // Return redacted string
}
```

## Test Vectors

| Input | Expected Output |
|-------|----------------|
| `token=ghp_abc123` | `token=[REDACTED]` |
| `api_key=sk-proj-test-key` | `api_key=[REDACTED]` |
| `Authorization: Bearer eyJhbGciOiJIUzI1NiJ9` | `Authorization: Bearer [REDACTED]` |
| `secret=mysecretpassword` | `secret=[REDACTED]` |
| `GITHUB_TOKEN=ghp_xxx` (env var assignment) | `GITHUB_TOKEN=[REDACTED]` |
| `path=/home/user/repo` (not a token pattern) | `path=/home/user/repo` (unchanged) |
