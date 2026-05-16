# Contracts: Stabilization Loop

**Date**: 2026-05-15

## Overview

This feature introduces changes to existing Tauri command contracts and Rust internal APIs. Each contract document describes the interface contract (input/output) and the behavioral guarantees.

### Contract List

| Contract | Type | File |
|----------|------|------|
| scan_repositories | Tauri Command (modified) | [scan-repository.md](./scan-repository.md) |
| get_resource_detail | Tauri Command (fixed) | [resource-detail.md](./resource-detail.md) |
| export_capability_pack | Tauri Command (modified) | [pack-export.md](./pack-export.md) |
| delete_pack | Tauri Command (modified) | [pack-delete.md](./pack-delete.md) |
| remove_repository | Tauri Command (modified) | [repo-remove.md](./repo-remove.md) |
| build_migration_plan | Tauri Command (modified) | [migration-plan.md](./migration-plan.md) |
| apply_migration_plan | Tauri Command (modified) | [migration-apply.md](./migration-apply.md) |
| rollback_migration | Tauri Command (modified) | [migration-rollback.md](./migration-rollback.md) |
| update_settings | Tauri Command (fixed) | [settings.md](./settings.md) |

### Rust Internal API Changes

| Crate | Change | Details |
|-------|--------|---------|
| `storage::repo_store` | New methods | `upsert_by_path()`, `get_by_canonical_path()`, `update_index_status()`, `delete_cascade()` |
| `storage::resource_store` | New methods | `get_by_id()`, `replace_for_repo()`, `replace_for_pack()`, `delete_by_pack()` |
| `storage::pack_store` | **New module** | `insert_pack()`, `get_pack_by_id()`, `list_packs()`, `delete_pack()` |
| `storage::migration_store` | **New module** | `insert_run()`, `get_run()`, `update_status()`, `update_snapshot()`, `update_report()`, `list_by_repo()` |
| `domain::repository` | New fields | `canonical_path`, `first_indexed_at`, `capability_index_status`, `last_capability_indexed_at`, `last_capability_error` |
| `domain::migration` | New type | `MigrationSnapshot` with `SnapshotItem` |
| `domain::migration` | New enum | `ConflictAction { Skip, Overwrite, Rename, Merge }` |
| `pack_engine::library::PackStore` | Deprecated | Replaced by `storage::pack_store` |
