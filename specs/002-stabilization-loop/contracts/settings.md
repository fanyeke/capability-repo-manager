# Contract: settings (Modified)

**Purpose**: Modified to persist settings to disk (currently a no-op).

## Input

```typescript
interface UpdateSettingsInput {
  new_settings: AppSettings;
}
```

## Output

```typescript
void
```

## Behavioral Changes

- Previously `update_settings` was a no-op that accepted settings but never wrote them; `get_settings` always returned defaults
- Now writes settings to `~/.capability-repo-manager/settings.json` on every update
- On app startup, `get_settings` reads from this file; if the file does not exist, defaults are used and a new file is created

## AppSettings Type

```typescript
interface AppSettings {
  theme?: 'light' | 'dark' | 'system';
  language?: string;
  scan_interval_minutes?: number;
  auto_discover?: boolean;
  max_concurrent_tasks?: number;
  [key: string]: unknown;
}
```

## Internal Flow

```
get_settings:
  1. Check if ~/.capability-repo-manager/settings.json exists
  2. If yes → read and deserialize as AppSettings
  3. If no → return default AppSettings (do NOT create file on read)

update_settings:
  1. Serialize new_settings to JSON
  2. Ensure ~/.capability-repo-manager/ directory exists (create if missing)
  3. Write JSON to ~/.capability-repo-manager/settings.json
  4. Return void
```
