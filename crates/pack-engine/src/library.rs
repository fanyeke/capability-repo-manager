use std::fs;
use std::path::{Path, PathBuf};

use domain::CapabilityPack;

/// In-memory pack store for managing the pack library.
/// In production, this would be backed by the storage crate's SQLite database.
#[derive(Debug, Default)]
pub struct PackStore {
    packs: Vec<CapabilityPack>,
    #[allow(dead_code)]
    storage_root: PathBuf,
}

impl PackStore {
    /// Create a new PackStore with the given storage root directory.
    pub fn new(storage_root: PathBuf) -> Self {
        PackStore { packs: Vec::new(), storage_root }
    }

    /// Insert a pack into the library. Rejects duplicate (name, version) pairs.
    pub fn insert(&mut self, pack: CapabilityPack) -> Result<(), String> {
        if self.packs.iter().any(|p| p.name == pack.name && p.version == pack.version) {
            return Err(format!("Pack with name '{}' and version '{}' already exists", pack.name, pack.version));
        }

        // Ensure storage directory exists
        if !pack.storage_dir.is_empty() {
            let dir = Path::new(&pack.storage_dir);
            if !dir.exists() {
                fs::create_dir_all(dir).map_err(|e| format!("Failed to create pack storage dir: {}", e))?;
            }
        }

        self.packs.push(pack);
        Ok(())
    }

    /// Get a pack by its ID.
    pub fn get_by_id(&self, id: &str) -> Option<&CapabilityPack> {
        self.packs.iter().find(|p| p.id == id)
    }

    /// List all packs, optionally filtered by pack_type.
    pub fn list(&self, pack_type: Option<&str>) -> Vec<&CapabilityPack> {
        self.packs.iter().filter(|p| pack_type.map(|pt| p.pack_type == pt).unwrap_or(true)).collect()
    }

    /// List all packs matching a search term (matches name and description).
    pub fn search(&self, query: &str) -> Vec<&CapabilityPack> {
        let q = query.to_lowercase();
        self.packs
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q)
                    || p.description.as_ref().map(|d| d.to_lowercase().contains(&q)).unwrap_or(false)
            })
            .collect()
    }

    /// Delete a pack by ID. Returns error if pack not found.
    /// Also removes the pack directory from disk.
    pub fn delete(&mut self, id: &str) -> Result<CapabilityPack, String> {
        let idx = self.packs.iter().position(|p| p.id == id).ok_or_else(|| format!("Pack not found: {}", id))?;

        let pack = self.packs.remove(idx);

        // Remove pack directory from disk
        let pack_dir = Path::new(&pack.storage_dir);
        if pack_dir.exists() {
            fs::remove_dir_all(pack_dir).map_err(|e| format!("Failed to delete pack directory: {}", e))?;
        }

        Ok(pack)
    }

    /// Return the number of packs in the library.
    pub fn len(&self) -> usize {
        self.packs.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.packs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pack(id: &str, name: &str, version: &str, pack_type: &str) -> CapabilityPack {
        CapabilityPack {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: Some(format!("{} pack", name)),
            pack_type: pack_type.to_string(),
            manifest_path: format!("/tmp/{}/pack.manifest.json", name),
            source_repo_id: None,
            source_commit: None,
            created_at: "2026-05-14T00:00:00Z".to_string(),
            storage_dir: format!("/tmp/{}/", name),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = PackStore::new(PathBuf::from("/tmp/test-packs"));
        let pack = make_pack("p1", "test-pack", "1.0.0", "project");
        store.insert(pack).unwrap();
        assert_eq!(store.len(), 1);

        let found = store.get_by_id("p1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-pack");
    }

    #[test]
    fn test_duplicate_name_version_rejected() {
        let mut store = PackStore::new(PathBuf::from("/tmp/test-packs"));
        store.insert(make_pack("p1", "test-pack", "1.0.0", "project")).unwrap();
        let result = store.insert(make_pack("p2", "test-pack", "1.0.0", "blueprint"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_list_all() {
        let mut store = PackStore::new(PathBuf::from("/tmp/test-packs"));
        store.insert(make_pack("p1", "pack-a", "1.0.0", "project")).unwrap();
        store.insert(make_pack("p2", "pack-b", "2.0.0", "blueprint")).unwrap();

        let all = store.list(None);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_list_filter_by_type() {
        let mut store = PackStore::new(PathBuf::from("/tmp/test-packs"));
        store.insert(make_pack("p1", "pack-a", "1.0.0", "project")).unwrap();
        store.insert(make_pack("p2", "pack-b", "2.0.0", "blueprint")).unwrap();

        let projects = store.list(Some("project"));
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "pack-a");
    }

    #[test]
    fn test_search_packs() {
        let mut store = PackStore::new(PathBuf::from("/tmp/test-packs"));
        store.insert(make_pack("p1", "devops-tools", "1.0.0", "project")).unwrap();
        store.insert(make_pack("p2", "frontend-setup", "1.0.0", "blueprint")).unwrap();

        let results = store.search("devops");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "devops-tools");
    }

    #[test]
    fn test_delete_pack() {
        let mut store = PackStore::new(PathBuf::from("/tmp/test-packs"));
        store.insert(make_pack("p1", "test-pack", "1.0.0", "project")).unwrap();
        assert_eq!(store.len(), 1);

        let removed = store.delete("p1").unwrap();
        assert_eq!(removed.name, "test-pack");
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_delete_nonexistent_fails() {
        let mut store = PackStore::new(PathBuf::from("/tmp/test-packs"));
        let result = store.delete("nonexistent");
        assert!(result.is_err());
    }
}
