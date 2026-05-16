# Contract: migration_apply (Modified)

**Purpose**: Modified to include snapshot creation, strategy validation, and state machine transitions.

## Input

```typescript
interface MigrationApplyInput {
  plan_id: string;
  strategies: ConflictStrategy[];
}
```

## Output

```typescript
interface MigrationApplyOutput {
  report: MigrationReport;
}
```

## Behavioral Changes

- Previously did not validate strategy actions, create snapshots, or enforce a state machine; it simply attempted file writes
- Now enforces a strict state machine: `planned → ready → executing → {success | partial_failure | failed}`
- Validates all `ConflictStrategy` actions against the `ConflictAction` enum; rejects any invalid actions before any writes occur
- Verifies all conflicts are resolved before proceeding (no unresolved conflicts allowed)
- Creates a scoped snapshot: backs up each file that will be overwritten or deleted, storing a snapshot manifest for rollback
- Per-item result tracking: each file write records success/failure independently
- Final status is determined by aggregate results (all success → `success`, partial → `partial_failure`, all failed → `failed`)

## Internal Flow

```
1. Fetch migration run from DB by plan_id
2. Verify status == 'planned' (else return error)
3. Validate all ConflictStrategy actions are valid ConflictAction enum values
4. Verify all conflicts have a corresponding strategy (no unresolved conflicts)
5. Update status to 'ready' in DB
6. Update status to 'executing' in DB
7. Build scoped snapshot:
   a. For each file that will be overwritten/deleted, copy to snapshot directory
   b. Store snapshot manifest (list of SnapshotItems with existed_before flag)
8. For each file in the migration:
   a. Attempt the write operation (copy/overwrite/delete)
   b. Record per-item success or failure
9. Determine final status:
   - All succeeded → 'success'
   - Some succeeded, some failed → 'partial_failure'
   - All failed → 'failed'
10. Store MigrationReport with per-item results
11. Update status and report reference in DB
12. Return MigrationReport
```
