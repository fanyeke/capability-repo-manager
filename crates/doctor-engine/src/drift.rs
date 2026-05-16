use domain::CapabilityResource;

/// Result of comparing two sets of capability resources.
#[derive(Debug, Clone)]
pub struct DriftResult {
    /// Resources identical in both sets (same type+name+hash)
    pub same: Vec<CapabilityResource>,
    /// Resources present in source but missing in target
    pub missing: Vec<CapabilityResource>,
    /// Resources present in target but missing in source
    pub extra: Vec<CapabilityResource>,
    /// Resources with same type+name but different content hash
    pub modified: Vec<ModifiedResource>,
}

/// A resource that exists in both sets but with different content.
#[derive(Debug, Clone)]
pub struct ModifiedResource {
    pub name: String,
    pub r#type: String,
    pub source_resource: CapabilityResource,
    pub target_resource: CapabilityResource,
}

/// Compare two sets of capability resources to detect drift.
///
/// Resources are matched by `(type, name)` tuple.
/// - Same type+name and same content_hash → `same`
/// - Same type+name but different content_hash → `modified`
/// - In source but not in target → `missing`
/// - In target but not in source → `extra`
///
/// # Parameters
/// - `source`: The baseline/expected set of resources (e.g., a pack or reference repo)
/// - `target`: The target/current set of resources (e.g., the repo being compared)
///
/// "missing" = resources in source that target lacks
/// "extra" = resources target has that source doesn't
pub fn compare(source: &[CapabilityResource], target: &[CapabilityResource]) -> DriftResult {
    let source_map = build_index(source);
    let target_map = build_index(target);

    let mut same = Vec::new();
    let mut missing = Vec::new();
    let mut extra = Vec::new();
    let mut modified = Vec::new();

    // Find resources in source: check against target
    for (key, src_res) in &source_map {
        match target_map.get(key) {
            Some(tgt_res) => {
                if src_res.content_hash == tgt_res.content_hash {
                    same.push(tgt_res.clone());
                } else {
                    modified.push(ModifiedResource {
                        name: src_res.name.clone(),
                        r#type: src_res.r#type.clone(),
                        source_resource: src_res.clone(),
                        target_resource: tgt_res.clone(),
                    });
                }
            }
            None => {
                missing.push(src_res.clone());
            }
        }
    }

    // Find resources in target not in source (extra)
    for (key, tgt_res) in &target_map {
        if !source_map.contains_key(key) {
            extra.push(tgt_res.clone());
        }
    }

    DriftResult { same, missing, extra, modified }
}

/// Build a lookup index from (type, name) → CapabilityResource
fn build_index(resources: &[CapabilityResource]) -> std::collections::HashMap<String, CapabilityResource> {
    let mut map = std::collections::HashMap::new();
    for r in resources {
        let key = format!("{}:{}", r.r#type, r.name);
        map.insert(key, r.clone());
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(id: &str, t: &str, name: &str, hash: &str) -> CapabilityResource {
        CapabilityResource {
            id: id.to_string(),
            repo_id: Some("repo".into()),
            pack_id: None,
            r#type: t.to_string(),
            name: name.to_string(),
            source_path: Some(format!("{}/{}.md", t, name)),
            scope: "project".into(),
            tracked_by_git: true,
            content_hash: Some(hash.to_string()),
            metadata_json: None,
            error_message: None,
        }
    }

    #[test]
    fn compare_same_resources() {
        let a = vec![r("1", "skill", "s1", "aaa")];
        let b = vec![r("2", "skill", "s1", "aaa")];
        let result = compare(&a, &b);
        assert_eq!(result.same.len(), 1);
        assert_eq!(result.same[0].id, "2");
    }

    #[test]
    fn compare_modified_resource() {
        let a = vec![r("1", "skill", "s1", "aaa")];
        let b = vec![r("2", "skill", "s1", "bbb")];
        let result = compare(&a, &b);
        assert_eq!(result.modified.len(), 1);
        assert!(result.same.is_empty());
    }

    #[test]
    fn compare_empty_inputs() {
        let result = compare(&[], &[]);
        assert!(result.same.is_empty());
        assert!(result.missing.is_empty());
        assert!(result.extra.is_empty());
        assert!(result.modified.is_empty());
    }
}
