# Contract: Log Event Schemas

**Purpose**: Define machine-readable event names and structured fields for all instrumented flows.

## Format

```
{timestamp} {level} {event_name} {field1}={value1} {field2}={value2}
```

## Scan Events

| Event | Level | Fields |
|-------|-------|--------|
| scan_started | INFO | operation_id, roots, max_depth |
| repo_discovered | DEBUG | operation_id, path, canonical_path |
| repo_deduplicated | DEBUG | operation_id, canonical_path |
| git_metadata_failed | WARN | operation_id, repo_path, error |
| capability_parse_started | INFO | operation_id, repo_id, repo_path |
| skill_found | DEBUG | operation_id, repo_id, name, source |
| mcp_entry_found | DEBUG | operation_id, repo_id, name |
| hook_entry_found | DEBUG | operation_id, repo_id, name |
| rule_found | DEBUG | operation_id, repo_id, name |
| agent_found | DEBUG | operation_id, repo_id, name |
| capability_parse_finished | INFO | operation_id, repo_id, skills, mcp, hooks, rules, agents |
| parse_failed | WARN | operation_id, repo_id, error |
| scan_finished | INFO | operation_id, repos_found, repos_added, repos_updated, repos_parsed, repos_parse_failed |

## Pack Events

| Event | Level | Fields |
|-------|-------|--------|
| pack_export_started | INFO | operation_id, repo_id, pack_name, version, selected_count |
| pack_resource_selected | DEBUG | operation_id, resource_id, type, source |
| pack_manifest_written | DEBUG | operation_id, manifest_path |
| pack_export_finished | INFO | operation_id, pack_id, resources, warnings |
| pack_delete_started | INFO | operation_id, pack_id |
| pack_delete_safety_checked | DEBUG | operation_id, pack_id, storage_dir, allowed_base |
| pack_delete_finished | INFO | operation_id, pack_id |

## Migration Events

| Event | Level | Fields |
|-------|-------|--------|
| migration_plan_started | INFO | operation_id, pack_id, target_repo_id |
| migration_item_classified | DEBUG | operation_id, resource_id, action, target_path |
| migration_plan_finished | INFO | operation_id, plan_id, items, conflicts, missing_deps |
| migration_apply_started | INFO | operation_id, plan_id |
| migration_strategy_validated | DEBUG | operation_id, resource_id, action |
| migration_snapshot_created | INFO | operation_id, plan_id, snapshot_dir, items |
| migration_copy_started | DEBUG | operation_id, resource_id, from, to |
| migration_copy_failed | ERROR | operation_id, resource_id, error |
| migration_apply_finished | INFO | operation_id, plan_id, status, added, overwritten, skipped, failed |
| rollback_started | INFO | operation_id, run_id |
| rollback_snapshot_loaded | DEBUG | operation_id, items |
| rollback_restored | DEBUG | operation_id, target, mode |
| rollback_removed_new_file | DEBUG | operation_id, target |
| rollback_finished | INFO | operation_id, run_id, restored |

## Doctor Events

| Event | Level | Fields |
|-------|-------|--------|
| doctor_started | INFO | operation_id, repo_id |
| doctor_check_started | DEBUG | operation_id, check |
| doctor_check_finished | DEBUG | operation_id, check, issues |
| doctor_finished | INFO | operation_id, repo_id, score, critical, warning, info |

## App Events

| Event | Level | Fields |
|-------|-------|--------|
| app_start | INFO | version, platform, settings_path, db_path, log_dir |
| settings_loaded | INFO | path |
| database_opened | INFO | path |
| logging_initialized | INFO | level, dir |
