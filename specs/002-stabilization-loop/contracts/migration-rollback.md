# Contract: migration_rollback (Modified)

**Purpose**: Modified to use scoped snapshot for precise per-file restore.

## Input

```typescript
interface MigrationRollbackInput {
  run_id: string;
}
```

## Output

```typescript
interface MigrationRollbackOutput {
  success: boolean;
  restored: number;
  message: string;
}
```

## Behavioral Changes

- Previously had no snapshot support; rollback was manual or undefined
- Now uses the scoped snapshot created during `migration_apply` for precise per-file restoration
- Only allows rollback from `'success'` or `'partial_failure'` statuses (not from `'failed'`, where no destructive writes were committed)
- Distinguishes between files that existed before the migration (restore from backup) and files that were created by migration (delete them)
- Updates status to `'rolled_back'` in DB after successful restoration

## Internal Flow

```
1. Fetch migration run from DB by run_id
2. Verify status is 'success' or 'partial_failure' (else return error)
3. Load snapshot manifest from snapshot_path stored on the migration run
4. For each SnapshotItem in the manifest:
   a. If file.existed_before == true → restore from backup_path
   b. If file.existed_before == false → delete the file (it was created by migration)
5. Update migration run status to 'rolled_back' in DB
6. Return { success: true, restored: <count>, message: "Rollback completed" }
```
