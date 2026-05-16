# Contract: repo_remove (Fixed)

**Purpose**: Fixed to cascade-delete associated resources and doctor reports.

## Input

```typescript
interface RepoRemoveInput {
  repo_id: string;
}
```

## Output

```typescript
void
```

## Behavioral Changes

- Previously only removed the repository row from the `repositories` table, leaving orphaned `capability_resources` and `doctor_reports` rows
- Now performs a transactional cascade delete to maintain referential integrity
- Migration history (`migration_runs`) is intentionally preserved for audit trail purposes; the UI should handle missing repos gracefully when displaying migration history

## Internal Flow

```
1. Validate repo_id is non-empty
2. Begin transaction
3. Verify repository exists in repositories table (else return "Repository not found")
4. DELETE FROM capability_resources WHERE repo_id = repo_id
5. DELETE FROM doctor_reports WHERE repo_id = repo_id
6. DELETE FROM repositories WHERE id = repo_id
7. Commit transaction
8. Return void
```
