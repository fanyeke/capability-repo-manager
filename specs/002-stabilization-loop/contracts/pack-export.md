# Contract: pack_export (Modified)

**Purpose**: Modified to write pack metadata to SQLite DB (not just in-memory PackStore).

## Input

```typescript
interface PackExportInput {
  repo_id: string;
  selection: {
    resource_ids: string[];
  };
  metadata: {
    name: string;
    version: string;
    description?: string;
    pack_type: string;
  };
}
```

## Output

```typescript
interface PackExportOutput {
  summary: PackSummary;
}
```

## Behavioral Changes

- Previously only wrote artifacts to disk via `pack_engine::export_pack` and stored metadata in an in-memory `PackStore` (lost on restart)
- Now persists pack metadata to a `packs` table in the SQLite database after disk artifacts are written
- Inserts capability resources into `capability_resources` with the new `pack_id` set, linking resources to the pack
- Entire flow is wrapped in an atomic transaction to prevent half-written state on failure

## Internal Flow

```
1. Validate repo_id, selection, and metadata
2. Create pack directory on disk
3. Call pack_engine::export_pack to write disk artifacts
4. Insert pack metadata row into DB (packs table)
5. For each resource in selection:
   - Insert or update capability_resources row with pack_id set
6. Commit DB transaction
7. On any failure:
   - Clean up half-created pack directory
   - Roll back DB transaction (no orphan rows)
   - Return error to caller
```
