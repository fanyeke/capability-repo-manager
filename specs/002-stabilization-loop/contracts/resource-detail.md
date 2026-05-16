# Contract: get_resource_detail (Fixed)

**Purpose**: Fix get_resource_detail to use direct primary key lookup instead of filtering by empty repo_id.

## Input

```typescript
interface GetResourceDetailInput {
  resource_id: string;
}
```

## Output

```typescript
interface GetResourceDetailOutput {
  resource: CapabilityResource;
  dependencies: DependencyStatus[];
}
```

## Behavioral Changes

- Replaces the current broken implementation that calls `ResourceStore::get_by_repo("")` then filters results in memory with an empty-string repo_id query
- Uses new `ResourceStore::get_by_id(resource_id)` method for direct primary key lookup
- Returns a clear `"Resource not found"` error for non-existent resource IDs instead of silently returning empty or undefined
- Dependencies are resolved after the resource is fetched, using the resource's own repo_id as context

## Internal Flow

```
1. Validate resource_id is non-empty
2. Call ResourceStore::get_by_id(resource_id) — direct PK lookup via SQL
3. If None → return Err("Resource not found")
4. Fetch dependencies for the resource from the dependency graph store
5. Return { resource, dependencies }
```
