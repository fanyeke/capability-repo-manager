use std::fs;
use std::path::Path;

use tauri::State;

use crate::state::AppState;
use domain::CapabilityResource;
use storage::repo_store::RepositoryStore;
use storage::resource_store::ResourceStore;

#[derive(serde::Serialize)]
pub struct CapabilityInventory {
    pub skills: Vec<CapabilityResource>,
    pub mcp: Vec<CapabilityResource>,
    pub hooks: Vec<CapabilityResource>,
    pub rules: Vec<CapabilityResource>,
    pub agents: Vec<CapabilityResource>,
    pub commands: Vec<CapabilityResource>,
    pub plugins: Vec<CapabilityResource>,
    pub settings: Vec<CapabilityResource>,
}

#[derive(serde::Serialize)]
pub struct ResourceDetail {
    pub resource: CapabilityResource,
    pub dependencies: Vec<DependencyStatus>,
}

#[derive(serde::Serialize)]
pub struct DependencyStatus {
    pub dep_type: String,
    pub name: String,
    pub required: bool,
    pub status: String,
}

#[tauri::command]
pub fn get_capability_inventory(repo_id: String, state: State<AppState>) -> Result<CapabilityInventory, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let resource_store = ResourceStore::new(&db);

    let resources = resource_store.get_by_repo(&repo_id).map_err(|e| format!("Database error: {}", e))?;

    Ok(group_resources(resources))
}

#[tauri::command]
pub fn get_resource_detail(resource_id: String, state: State<AppState>) -> Result<ResourceDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let resource_store = ResourceStore::new(&db);

    let resource = resource_store
        .get_by_id(&resource_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Resource not found: {}", resource_id))?;

    let deps = resolve_dependencies(&resource);

    Ok(ResourceDetail { resource, dependencies: deps })
}

#[derive(serde::Serialize)]
pub struct ResourceContent {
    pub content: Option<String>,
    pub language: String,
    pub size_bytes: u64,
    pub exists: bool,
    pub is_binary: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub fn get_resource_content(resource_id: String, state: State<AppState>) -> Result<ResourceContent, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let resource_store = ResourceStore::new(&db);
    let resource = resource_store
        .get_by_id(&resource_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Resource not found: {}", resource_id))?;

    let source_path = match resource.source_path {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => return Ok(ResourceContent { content: None, language: "text".to_string(), size_bytes: 0, exists: false, is_binary: false, error: Some("No source path".to_string()) }),
    };

    // Resolve base directory from repo_id
    let base_dir = if let Some(ref repo_id) = resource.repo_id {
        let repo_store = RepositoryStore::new(&db);
        match repo_store.get_by_id(repo_id) {
            Ok(Some(repo)) => repo.path,
            _ => return Ok(ResourceContent { content: None, language: "text".to_string(), size_bytes: 0, exists: false, is_binary: false, error: Some("Repository not found".to_string()) }),
        }
    } else {
        return Ok(ResourceContent { content: None, language: "text".to_string(), size_bytes: 0, exists: false, is_binary: false, error: Some("Content not available for pack resources".to_string()) });
    };

    // Resolve path and prevent traversal
    let base = Path::new(&base_dir).canonicalize().map_err(|e| format!("Cannot resolve base: {}", e))?;
    let full_path = base.join(&source_path);
    let resolved = full_path.canonicalize().map_err(|_| format!("File not found: {}", source_path))?;

    if !resolved.starts_with(&base) {
        return Ok(ResourceContent { content: None, language: "text".to_string(), size_bytes: 0, exists: false, is_binary: false, error: Some("Path traversal".to_string()) });
    }

    let metadata = fs::metadata(&resolved).map_err(|_| format!("Cannot read file"))?;
    let size_bytes = metadata.len();
    let language = language_from_extension(&resolved);
    let is_binary = is_binary_file(&resolved);

    if is_binary {
        return Ok(ResourceContent { content: None, language, size_bytes, exists: true, is_binary: true, error: None });
    }

    if size_bytes > 1024 * 1024 {
        return Ok(ResourceContent { content: None, language, size_bytes, exists: true, is_binary: false, error: Some(format!("File too large ({} bytes)", size_bytes)) });
    }

    let content = fs::read_to_string(&resolved).map_err(|e| format!("Read error: {}", e))?;
    Ok(ResourceContent { content: Some(content), language, size_bytes, exists: true, is_binary: false, error: None })
}

fn language_from_extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| match e {
            "md" | "mdx" => "markdown",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "sh" | "bash" | "zsh" => "shell",
            "py" => "python",
            "rs" => "rust",
            "ts" | "tsx" => "typescript",
            "js" | "jsx" => "javascript",
            "css" => "css",
            "html" | "svelte" => "html",
            _ => "text",
        })
        .unwrap_or("text")
        .to_string()
}

fn is_binary_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e, "png" | "jpg" | "jpeg" | "gif" | "ico" | "pdf" | "zip" | "gz"))
        .unwrap_or(false)
}

fn resolve_dependencies(_resource: &CapabilityResource) -> Vec<DependencyStatus> {
    Vec::new()
}

fn group_resources(resources: Vec<CapabilityResource>) -> CapabilityInventory {
    let mut skills = Vec::new();
    let mut mcp = Vec::new();
    let mut hooks = Vec::new();
    let mut rules = Vec::new();
    let mut agents = Vec::new();
    let mut commands = Vec::new();
    let mut plugins = Vec::new();
    let mut settings = Vec::new();

    for r in resources {
        match r.r#type.as_str() {
            "skill" => skills.push(r),
            "mcp" => mcp.push(r),
            "hook" => hooks.push(r),
            "rule" => rules.push(r),
            "agent" => agents.push(r),
            "command" => commands.push(r),
            "plugin" => plugins.push(r),
            "settings" => settings.push(r),
            _ => {}
        }
    }

    CapabilityInventory { skills, mcp, hooks, rules, agents, commands, plugins, settings }
}
