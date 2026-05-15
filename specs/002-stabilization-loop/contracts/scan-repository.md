# Contract: scan_repositories (Modified)

**Purpose**: Scan filesystem paths for Git repositories, parse capabilities, persist to DB.

## Input

```typescript
{
  paths: string[]  // Filesystem paths to scan
}
```

## Output

```typescript
{
  repos_found: number,       // Total Git repos discovered
  repos_added: number,       // New repos (not previously indexed)
  repos_updated: number,     // Existing repos (previously indexed)
  repos_parsed: number,      // Repos where capability parsing succeeded
  repos_parse_failed: number, // Repos where capability parsing failed (old resources preserved)
  errors: ScanError[]
}

ScanError { path: string, message: string }
```

## Behavioral Changes

1. **Identity**: Uses canonical path lookup to reuse existing repo IDs instead of always generating new UUIDs
2. **Capability Indexing**: After persisting repo metadata, immediately parses capability inventory and stores resources
3. **Transaction**: Each repo's upsert + resource replace is wrapped in a single SQLite transaction
4. **Parse Error Handling**: Parse failure preserves old resources, records error in `last_capability_error`, sets status to `parse_failed`
5. **New Output Fields**: `repos_parsed`, `repos_parse_failed` added for transparency

## Internal Flow

```
for each repo discovered:
  1. canonical_path = resolve_symlinks(repo.path)
  2. existing = repo_store.get_by_canonical_path(canonical_path)
  3. if existing: repo.id = existing.id; repos_updated++
     else: repo.id = new_uuid(); repos_added++
  4. repo_store.upsert(repo)  // INSERT OR UPDATE
  5. match parse_repo(repo.path):
       Ok(inventory) -> resource_store.replace_for_repo(repo.id, resources); repo_parsed++
       Err(e) -> repo_store.record_parse_error(repo.id, e); repo_parse_failed++
  6. commit
```
