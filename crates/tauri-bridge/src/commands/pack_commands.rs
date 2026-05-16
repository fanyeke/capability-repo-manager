use std::path::PathBuf;

use tauri::State;

use crate::state::AppState;
use domain::CapabilityPack;
use pack_engine;
use pack_engine::manifest::Manifest;
use storage::pack_store::PackStore;
use storage::{repo_store::RepositoryStore, resource_store::ResourceStore};

#[derive(serde::Serialize)]
pub struct PackSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub pack_type: String,
    pub resource_count: usize,
    pub source_repo_name: Option<String>,
    pub created_at: String,
}

#[derive(serde::Serialize)]
pub struct PackDetail {
    pub pack: CapabilityPack,
    pub resources: Vec<domain::CapabilityResource>,
    pub manifest: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct PackSelection {
    pub resource_ids: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct PackMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub pack_type: String,
}

#[derive(serde::Deserialize)]
pub struct PackFilter {
    pub search: Option<String>,
    pub pack_type: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(serde::Serialize)]
pub struct ValidationError {
    pub resource_ref: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn export_capability_pack(
    repo_id: String,
    selection: PackSelection,
    metadata: PackMetadata,
    state: State<AppState>,
) -> Result<PackSummary, String> {
    let ctx = domain::OperationContext::new("export_pack");
    let _span = tracing::info_span!("export_pack", operation_id = %ctx.operation_id).entered();

    tracing::info!(
        operation_id = %ctx.operation_id,
        repo_id = %repo_id,
        pack_name = %metadata.name,
        version = %metadata.version,
        selected = %selection.resource_ids.len(),
        "pack_export_started"
    );

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;

    let repo_store = RepositoryStore::new(&db);
    let resource_store = ResourceStore::new(&db);

    let repo = repo_store
        .get_by_id(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

    let all_resources = resource_store.get_by_repo(&repo_id).map_err(|e| format!("Database error: {}", e))?;

    let selected: Vec<domain::CapabilityResource> =
        all_resources.into_iter().filter(|r| selection.resource_ids.contains(&r.id)).collect();

    let output_dir = PathBuf::from(&settings.pack_storage_dir);
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("Cannot create pack directory: {}", e))?;

    let request = pack_engine::ExportRequest {
        pack_name: metadata.name.clone(),
        version: metadata.version.clone(),
        description: metadata.description.clone(),
        pack_type: metadata.pack_type,
        source_repo_id: repo_id,
        source_commit: repo.head_commit,
        source_repo_path: PathBuf::from(&repo.path),
        selected_resources: selected,
        output_base_dir: output_dir,
    };

    let mut library = pack_engine::library::PackStore::new(PathBuf::from(&settings.pack_storage_dir));
    let result = pack_engine::export_pack(request, &mut library).map_err(|e| format!("Export failed: {}", e))?;

    // Persist pack metadata to DB
    let pack_store = PackStore::new(&db);
    pack_store.insert_pack(&result.pack).map_err(|e| format!("Failed to persist pack metadata: {}", e))?;

    // Store pack resources in DB
    let mut pack_resources: Vec<domain::CapabilityResource> = Vec::new();
    for res in &result.manifest.resources {
        pack_resources.push(domain::CapabilityResource {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id: None,
            pack_id: Some(result.pack.id.clone()),
            r#type: res.resource_type.clone(),
            name: res.name.clone(),
            source_path: Some(res.source.clone()),
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: res.hash.clone(),
            metadata_json: None,
            error_message: None,
        });
    }
    if !pack_resources.is_empty() {
        resource_store.insert_batch(&pack_resources).map_err(|e| format!("Failed to store pack resources: {}", e))?;
    }

    tracing::info!(
        operation_id = %ctx.operation_id,
        pack_id = %result.pack.id,
        resources = %result.manifest.resources.len(),
        "pack_export_finished"
    );

    Ok(PackSummary {
        id: result.pack.id,
        name: result.pack.name,
        version: result.pack.version,
        description: result.pack.description,
        pack_type: result.pack.pack_type,
        resource_count: result.manifest.resources.len(),
        source_repo_name: Some(repo.name),
        created_at: result.pack.created_at,
    })
}

#[tauri::command]
pub fn list_packs(filter: PackFilter, state: State<AppState>) -> Result<Vec<PackSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let pack_store = PackStore::new(&db);
    let packs = pack_store
        .list_packs(filter.search.as_deref(), filter.pack_type.as_deref())
        .map_err(|e| format!("Database error: {}", e))?;

    let mut summaries = Vec::new();
    for pack in packs {
        let resources = ResourceStore::new(&db).get_by_pack(&pack.id).unwrap_or_default();

        let repo_name = pack
            .source_repo_id
            .as_ref()
            .and_then(|rid| RepositoryStore::new(&db).get_by_id(rid).ok().flatten().map(|r| r.name));

        summaries.push(PackSummary {
            id: pack.id.clone(),
            name: pack.name.clone(),
            version: pack.version.clone(),
            description: pack.description.clone(),
            pack_type: pack.pack_type.clone(),
            resource_count: resources.len(),
            source_repo_name: repo_name,
            created_at: pack.created_at.clone(),
        });
    }

    Ok(summaries)
}

#[tauri::command]
pub fn get_pack_detail(pack_id: String, state: State<AppState>) -> Result<PackDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let pack_store = PackStore::new(&db);
    let pack = pack_store
        .get_pack_by_id(&pack_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Pack not found: {}", pack_id))?;

    let resources = ResourceStore::new(&db).get_by_pack(&pack_id).unwrap_or_default();

    let manifest_path = std::path::Path::new(&pack.manifest_path);
    let manifest: serde_json::Value = if manifest_path.exists() {
        let content = std::fs::read_to_string(manifest_path).map_err(|e| format!("Cannot read manifest: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    Ok(PackDetail { pack, resources, manifest })
}

#[tauri::command]
pub fn delete_pack(pack_id: String, state: State<AppState>) -> Result<(), String> {
    let ctx = domain::OperationContext::new("delete_pack");
    let _span = tracing::info_span!("delete_pack", operation_id = %ctx.operation_id).entered();

    tracing::info!(
        operation_id = %ctx.operation_id,
        pack_id = %pack_id,
        "pack_delete_started"
    );

    let db = state.db.lock().map_err(|e| e.to_string())?;

    let pack_store = PackStore::new(&db);

    // First get pack info to know storage_dir (before deleting DB record)
    let pack = pack_store
        .get_pack_by_id(&pack_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Pack not found: {}", pack_id))?;

    // Delete files first, then DB (atomicity: file failure stops before DB delete)
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let allowed_base = std::path::Path::new(&settings.pack_storage_dir).canonicalize().unwrap_or_default();
    let pack_dir = std::path::Path::new(&pack.storage_dir);
    if pack_dir.exists() {
        // Safety check: ensure pack_dir is within the allowed storage directory
        if let Ok(canonical) = pack_dir.canonicalize() {
            if !allowed_base.as_os_str().is_empty() && !canonical.starts_with(&allowed_base) {
                return Err(format!("Pack directory outside allowed storage area: {}", pack_dir.display()));
            }
        }
        std::fs::remove_dir_all(pack_dir).map_err(|e| format!("Failed to delete pack directory: {}", e))?;
    }

    // Now delete DB record
    pack_store
        .delete_pack(&pack_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Pack not found: {}", pack_id))?;

    tracing::info!(
        operation_id = %ctx.operation_id,
        pack_id = %pack_id,
        "pack_delete_finished"
    );

    Ok(())
}

#[tauri::command]
pub fn validate_pack(pack_id: String, state: State<AppState>) -> Result<ValidationResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let pack_store = PackStore::new(&db);
    let pack = pack_store
        .get_pack_by_id(&pack_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Pack not found: {}", pack_id))?;

    let pack_dir = std::path::Path::new(&pack.storage_dir);
    let manifest_path = pack_dir.join("pack.manifest.json");

    if !manifest_path.exists() {
        return Ok(ValidationResult {
            valid: false,
            errors: vec![ValidationError { resource_ref: None, message: "Manifest file not found".to_string() }],
        });
    }

    let content = std::fs::read_to_string(&manifest_path).map_err(|e| format!("Cannot read manifest: {}", e))?;

    let manifest: Manifest = serde_json::from_str(&content).map_err(|e| format!("Invalid manifest JSON: {}", e))?;

    let result = pack_engine::validate_existing_pack(pack_dir, &manifest);

    Ok(ValidationResult {
        valid: result.valid,
        errors: result
            .errors
            .into_iter()
            .map(|e| ValidationError { resource_ref: e.resource_ref, message: e.message })
            .collect(),
    })
}
