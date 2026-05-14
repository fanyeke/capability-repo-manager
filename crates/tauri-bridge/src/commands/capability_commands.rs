use std::collections::HashMap;

use tauri::State;

use crate::state::AppState;
use domain::CapabilityResource;
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
pub fn get_capability_inventory(
    repo_id: String,
    state: State<AppState>,
) -> Result<CapabilityInventory, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let resource_store = ResourceStore::new(&db);

    let resources = resource_store
        .get_by_repo(&repo_id)
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(group_resources(resources))
}

#[tauri::command]
pub fn get_resource_detail(
    resource_id: String,
    state: State<AppState>,
) -> Result<ResourceDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let resource_store = ResourceStore::new(&db);

    let resources = resource_store
        .get_by_repo("")
        .map_err(|e| format!("Database error: {}", e))?;

    let resource = resources
        .into_iter()
        .find(|r| r.id == resource_id)
        .ok_or_else(|| format!("Resource not found: {}", resource_id))?;

    let deps = resolve_dependencies(&resource);

    Ok(ResourceDetail {
        resource,
        dependencies: deps,
    })
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

    CapabilityInventory {
        skills,
        mcp,
        hooks,
        rules,
        agents,
        commands,
        plugins,
        settings,
    }
}
