use domain::{CapabilityResource, MigrationConflict};
use std::collections::HashMap;

pub fn detect_conflicts(
    pack_resources: &[CapabilityResource],
    target_resources: &[CapabilityResource],
) -> Vec<MigrationConflict> {
    let mut conflicts = Vec::new();

    // Index target resources by (type, name) for lookup
    let mut target_by_type_name: HashMap<(String, String), &CapabilityResource> = HashMap::new();
    for res in target_resources {
        target_by_type_name.insert((res.r#type.clone(), res.name.clone()), res);
    }

    // Index target resources by source_path for path conflict detection
    let mut target_by_path: HashMap<String, Vec<&CapabilityResource>> = HashMap::new();
    for res in target_resources {
        if let Some(ref path) = res.source_path {
            target_by_path
                .entry(path.clone())
                .or_default()
                .push(res);
        }
    }

    for pack_res in pack_resources {
        let key = (pack_res.r#type.clone(), pack_res.name.clone());

        if let Some(target_res) = target_by_type_name.get(&key) {
            let same_path = pack_res.source_path == target_res.source_path;

            if same_path {
                // Check if content is identical (idempotent)
                let same_content = pack_res.content_hash.is_some()
                    && target_res.content_hash.is_some()
                    && pack_res.content_hash == target_res.content_hash;

                if !same_content {
                    conflicts.push(MigrationConflict {
                        resource_name: pack_res.name.clone(),
                        resource_type: pack_res.r#type.clone(),
                        reason: "overwrite".to_string(),
                        recommended_actions: vec![
                            "skip".to_string(),
                            "overwrite".to_string(),
                            "rename".to_string(),
                        ],
                    });
                }
            } else {
                conflicts.push(MigrationConflict {
                    resource_name: pack_res.name.clone(),
                    resource_type: pack_res.r#type.clone(),
                    reason: "path_conflict".to_string(),
                    recommended_actions: vec![
                        "skip".to_string(),
                        "overwrite".to_string(),
                        "rename".to_string(),
                    ],
                });
            }
        } else if let Some(ref pack_path) = pack_res.source_path {
            // Check for path conflict with different-named resources
            if let Some(occupants) = target_by_path.get(pack_path) {
                for occupant in occupants {
                    // Avoid duplicate conflicts
                    if !conflicts.iter().any(|c| {
                        c.resource_name == occupant.name && c.resource_type == occupant.r#type
                    }) {
                        conflicts.push(MigrationConflict {
                            resource_name: pack_res.name.clone(),
                            resource_type: pack_res.r#type.clone(),
                            reason: "path_conflict".to_string(),
                            recommended_actions: vec![
                                "skip".to_string(),
                                "rename".to_string(),
                            ],
                        });
                    }
                }
            }
        }
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_both_sides_no_conflicts() {
        let conflicts = detect_conflicts(&[], &[]);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn different_types_same_name_no_conflict() {
        let pack = vec![CapabilityResource {
            id: "1".to_string(),
            repo_id: None,
            pack_id: Some("p1".to_string()),
            r#type: "skill".to_string(),
            name: "test".to_string(),
            source_path: Some("skills/test/".to_string()),
            scope: "project".to_string(),
            tracked_by_git: false,
            content_hash: Some("hash1".to_string()),
            metadata_json: None,
            error_message: None,
        }];
        let target = vec![CapabilityResource {
            id: "2".to_string(),
            repo_id: Some("r1".to_string()),
            pack_id: None,
            r#type: "rule".to_string(),
            name: "test".to_string(),
            source_path: Some(".claude/rules/test.md".to_string()),
            scope: "project".to_string(),
            tracked_by_git: true,
            content_hash: Some("hash2".to_string()),
            metadata_json: None,
            error_message: None,
        }];

        let conflicts = detect_conflicts(&pack, &target);
        assert!(conflicts.is_empty());
    }
}
