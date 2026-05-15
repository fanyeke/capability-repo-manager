# Contract: migration_plan (Modified)

**Purpose**: Modified to store plans with enhanced state tracking.

## Input

```typescript
interface MigrationPlanInput {
  pack_id: string;
  target_repo_id: string;
}
```

## Output

```typescript
interface MigrationPlanOutput {
  plan: MigrationPlan;
}
```

## Behavioral Changes

- Initial status remains `'planned'` (unchanged from current behavior)
- The plan is now persisted via `migration_store::insert_run` instead of being stored only in memory
- The returned `plan_id` is the migration run ID from the database, enabling subsequent commands (`migration_apply`, `migration_rollback`) to reference the same persisted record
- Enables state machine transitions: only a plan with status `'planned'` can proceed to application

## Internal Flow

```
1. Validate pack_id and target_repo_id exist
2. Create MigrationPlan with status = 'planned'
3. Call migration_store::insert_run(plan) to persist to DB
4. Return plan with plan_id set to the inserted run ID
```
