# Contract: Debug Bundle

**Purpose**: Define the debug bundle export format and behavior.

## Export Command

```typescript
Input:  { redact_paths?: boolean }
Output: { bundle_path: string; size_bytes: number; log_count: number; event_count: number }
```

## Bundle Contents (.zip)

```
debug-bundle-{timestamp}.zip
├── metadata.json                    # App version, platform, timestamp
├── logs/
│   ├── app-{date}.log               # Last 7 days of app logs
│   └── app-{date-1}.log
├── settings.json                    # App settings (redacted)
├── recent-operations.json           # Last 50 operation events
├── recent-errors.json               # Last 20 ERROR-level log events
├── migration-runs.json              # Recent migration run summaries
└── doctor-reports.json              # Recent doctor report summaries
```

## Redaction

When `redact_paths: true`:
- All `/home/<username>` paths are replaced with `~`
- All matching sensitive patterns are replaced with `[REDACTED]`

When `redact_paths: false`:
- Only sensitive patterns (tokens, API keys) are redacted
- File paths remain as-is (useful for developer self-export)
