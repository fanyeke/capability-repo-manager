use domain::{CapabilityResource, MigrationConflict, MigrationPlan, MigrationPlanItem, ResourceDependency};
use uuid::Uuid;

pub fn build_plan(
    pack_resources: &[CapabilityResource],
    target_resources: &[CapabilityResource],
    source_id: &str,
    target_repo_id: &str,
) -> MigrationPlan {
    let plan_id = Uuid::new_v4().to_string();
    let mut items = Vec::new();
    let mut conflicts = Vec::new();
    let mut missing_dependencies = Vec::new();

    for pack_res in pack_resources {
        let target_match = target_resources
            .iter()
            .find(|t| t.r#type == pack_res.r#type && t.name == pack_res.name);

        let (action, conflict) = classify_resource(pack_res, target_match);
        if let Some(c) = conflict {
            conflicts.push(c);
        }

        items.push(MigrationPlanItem {
            resource_id: pack_res.id.clone(),
            action,
            source_path: pack_res.source_path.clone(),
            target_path: target_match.and_then(|t| t.source_path.clone()),
            status: "pending".to_string(),
        });
    }

    // Check dependencies
    if let Some(deps) = extract_dependencies(pack_resources) {
        for dep in &deps {
            let satisfied = pack_resources
                .iter()
                .any(|r| r.r#type == dep.dep_type && r.name == dep.name)
                || target_resources
                    .iter()
                    .any(|r| r.r#type == dep.dep_type && r.name == dep.name);
            if !satisfied {
                missing_dependencies.push(dep.clone());
            }
        }
    }

    MigrationPlan {
        plan_id,
        source_type: "pack".to_string(),
        source_id: source_id.to_string(),
        target_repo_id: target_repo_id.to_string(),
        items,
        conflicts,
        missing_dependencies,
    }
}

fn classify_resource(
    pack: &CapabilityResource,
    target: Option<&CapabilityResource>,
) -> (String, Option<MigrationConflict>) {
    match target {
        None => ("add".to_string(), None),
        Some(t) => {
            let same_path = pack.source_path == t.source_path;
            if same_path {
                let conflict = MigrationConflict {
                    resource_name: pack.name.clone(),
                    resource_type: pack.r#type.clone(),
                    reason: "overwrite".to_string(),
                    recommended_actions: vec![
                        "skip".to_string(),
                        "overwrite".to_string(),
                        "rename".to_string(),
                    ],
                };
                ("overwrite".to_string(), Some(conflict))
            } else {
                let conflict = MigrationConflict {
                    resource_name: pack.name.clone(),
                    resource_type: pack.r#type.clone(),
                    reason: "path_conflict".to_string(),
                    recommended_actions: vec![
                        "skip".to_string(),
                        "overwrite".to_string(),
                        "rename".to_string(),
                    ],
                };
                ("overwrite".to_string(), Some(conflict))
            }
        }
    }
}

fn extract_dependencies(resources: &[CapabilityResource]) -> Option<Vec<ResourceDependency>> {
    let mut deps = Vec::new();
    for res in resources {
        if let Some(ref meta) = res.metadata_json {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(meta) {
                if let Some(dep_list) = parsed.get("dependencies").and_then(|d| d.as_array()) {
                    for dep in dep_list {
                        if let (Some(dep_type), Some(name)) = (
                            dep.get("type").and_then(|v| v.as_str()),
                            dep.get("name").and_then(|v| v.as_str()),
                        ) {
                            deps.push(ResourceDependency {
                                dep_type: dep_type.to_string(),
                                name: name.to_string(),
                                required: dep
                                    .get("required")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                status: "missing".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    if deps.is_empty() {
        None
    } else {
        Some(deps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_plan_produces_deterministic_classification() {
        let res = |id: &str, t: &str, n: &str| CapabilityResource {
            id: id.to_string(),
            repo_id: None,
            pack_id: Some("p1".to_string()),
            r#type: t.to_string(),
            name: n.to_string(),
            source_path: Some(format!("{}/{}/", t, n)),
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: None,
            metadata_json: None,
            error_message: None,
        };

        let pack = vec![res("r1", "skill", "s1"), res("r2", "rule", "r1")];
        let plan = build_plan(&pack, &[], "p1", "repo-1");

        assert_eq!(plan.items.len(), 2);
        let adds: Vec<_> = plan.items.iter().filter(|i| i.action == "add").collect();
        assert_eq!(adds.len(), 2);
    }
}
