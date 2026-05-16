# Contract: pack_delete (Fixed)

**Purpose**: Fixed to also clean up DB records, not just disk files.

## Input

```typescript
interface PackDeleteInput {
  pack_id: string;
}
```

## Output

```typescript
void
```

## Behavioral Changes

- Previously only deleted the pack directory from disk, leaving orphaned rows in the database
- Now performs a coordinated cleanup that removes both disk artifacts and database records
- Adds security validation: the pack's `storage_dir` must be within the configured pack root directory
- Disk deletion must succeed before DB rows are deleted; if disk delete fails, DB rows are preserved and a clear error is returned (no half-state)

## Internal Flow

```
1. Call storage::pack_store::delete_pack(pack_id)
2. Inside delete_pack:
   a. Fetch pack metadata from DB; verify it exists (else return "Pack not found")
   b. Validate storage_dir is within the pack root (path traversal check)
   c. Delete pack directory from disk
   d. If disk delete fails → return Err with descriptive message; do NOT touch DB rows
   e. Delete pack row from packs table
   f. Delete all rows from capability_resources WHERE pack_id = pack_id
   g. Commit
3. Return void
```
