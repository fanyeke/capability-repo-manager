use std::path::PathBuf;

use domain::CapabilityPack;
use pack_engine::library::PackStore;

fn make_pack(id: &str, name: &str, version: &str, pack_type: &str) -> CapabilityPack {
    CapabilityPack {
        id: id.to_string(),
        name: name.to_string(),
        version: version.to_string(),
        description: Some(format!("{} description", name)),
        pack_type: pack_type.to_string(),
        manifest_path: format!("/tmp/{}/pack.manifest.json", name),
        source_repo_id: None,
        source_commit: None,
        created_at: "2026-05-14T00:00:00Z".to_string(),
        storage_dir: String::new(),
    }
}

#[test]
fn test_library_insert_pack() {
    let mut store = PackStore::new(PathBuf::from("/tmp/packs"));
    let pack = make_pack("id-1", "test-pack", "1.0.0", "project");
    store.insert(pack).unwrap();
    assert_eq!(store.len(), 1);

    let found = store.get_by_id("id-1").unwrap();
    assert_eq!(found.name, "test-pack");
    assert_eq!(found.version, "1.0.0");
}

#[test]
fn test_library_duplicate_name_version_rejected() {
    let mut store = PackStore::new(PathBuf::from("/tmp/packs"));
    store
        .insert(make_pack("id-1", "same-name", "1.0.0", "project"))
        .unwrap();
    let result = store.insert(make_pack("id-2", "same-name", "1.0.0", "blueprint"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_library_get_by_id() {
    let mut store = PackStore::new(PathBuf::from("/tmp/packs"));
    store
        .insert(make_pack("abc-123", "my-pack", "2.0.0", "blueprint"))
        .unwrap();

    let found = store.get_by_id("abc-123");
    assert!(found.is_some());
    assert_eq!(found.unwrap().pack_type, "blueprint");

    let missing = store.get_by_id("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_library_list_packs() {
    let mut store = PackStore::new(PathBuf::from("/tmp/packs"));
    store
        .insert(make_pack("p1", "pack-a", "1.0.0", "project"))
        .unwrap();
    store
        .insert(make_pack("p2", "pack-b", "1.0.0", "blueprint"))
        .unwrap();
    store
        .insert(make_pack("p3", "pack-c", "2.0.0", "baseline"))
        .unwrap();

    let all = store.list(None);
    assert_eq!(all.len(), 3);
}

#[test]
fn test_library_delete_pack() {
    let mut store = PackStore::new(PathBuf::from("/tmp/packs"));
    store
        .insert(make_pack("to-delete", "temp-pack", "1.0.0", "project"))
        .unwrap();
    assert_eq!(store.len(), 1);

    let removed = store.delete("to-delete").unwrap();
    assert_eq!(removed.name, "temp-pack");
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
}

#[test]
fn test_library_list_packs_filtered_by_type() {
    let mut store = PackStore::new(PathBuf::from("/tmp/packs"));
    store
        .insert(make_pack("p1", "proj-pack", "1.0.0", "project"))
        .unwrap();
    store
        .insert(make_pack("p2", "blue-pack", "1.0.0", "blueprint"))
        .unwrap();
    store
        .insert(make_pack("p3", "proj-pack-2", "2.0.0", "project"))
        .unwrap();

    let projects = store.list(Some("project"));
    assert_eq!(projects.len(), 2);

    let blueprints = store.list(Some("blueprint"));
    assert_eq!(blueprints.len(), 1);
    assert_eq!(blueprints[0].name, "blue-pack");

    let baselines = store.list(Some("baseline"));
    assert_eq!(baselines.len(), 0);
}

#[test]
fn test_library_search() {
    let mut store = PackStore::new(PathBuf::from("/tmp/packs"));
    store
        .insert(make_pack("p1", "devops-setup", "1.0.0", "project"))
        .unwrap();
    store
        .insert(make_pack("p2", "frontend-toolkit", "1.0.0", "blueprint"))
        .unwrap();
    store
        .insert(make_pack("p3", "backend-utils", "1.0.0", "project"))
        .unwrap();

    let results = store.search("devops");
    assert_eq!(results.len(), 1);

    let results = store.search("end");
    assert_eq!(results.len(), 2); // frontend, backend

    let results = store.search("nonexistent");
    assert_eq!(results.len(), 0);
}
