use crate::Database;
use domain::CapabilityPack;
use rusqlite::{params, Result};

pub struct PackStore<'a> {
    db: &'a Database,
}

impl<'a> PackStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_pack(row: &rusqlite::Row) -> rusqlite::Result<CapabilityPack> {
        Ok(CapabilityPack {
            id: row.get(0)?,
            name: row.get(1)?,
            version: row.get(2)?,
            description: row.get(3)?,
            pack_type: row.get(4)?,
            manifest_path: row.get(5)?,
            source_repo_id: row.get(6)?,
            source_commit: row.get(7)?,
            created_at: row.get(8)?,
            storage_dir: row.get(9)?,
        })
    }

    const SELECT_COLS: &'static str =
        "id, name, version, description, pack_type, manifest_path, \
         source_repo_id, source_commit, created_at, storage_dir";

    pub fn insert_pack(&self, pack: &CapabilityPack) -> Result<()> {
        let sql = format!(
            "INSERT INTO packs ({}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            Self::SELECT_COLS
        );
        self.db.conn().execute(
            &sql,
            params![
                pack.id,
                pack.name,
                pack.version,
                pack.description,
                pack.pack_type,
                pack.manifest_path,
                pack.source_repo_id,
                pack.source_commit,
                pack.created_at,
                pack.storage_dir,
            ],
        )?;
        Ok(())
    }

    pub fn get_pack_by_id(&self, pack_id: &str) -> Result<Option<CapabilityPack>> {
        let sql = format!(
            "SELECT {} FROM packs WHERE id = ?1",
            Self::SELECT_COLS
        );
        let mut stmt = self.db.conn().prepare(&sql)?;
        let mut rows = stmt.query_map(params![pack_id], Self::row_to_pack)?;
        match rows.next() {
            Some(Ok(pack)) => Ok(Some(pack)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn get_pack_by_name_version(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<CapabilityPack>> {
        let sql = format!(
            "SELECT {} FROM packs WHERE name = ?1 AND version = ?2",
            Self::SELECT_COLS
        );
        let mut stmt = self.db.conn().prepare(&sql)?;
        let mut rows = stmt.query_map(params![name, version], Self::row_to_pack)?;
        match rows.next() {
            Some(Ok(pack)) => Ok(Some(pack)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// List packs with optional search and pack_type filters.
    ///
    /// When `search` is provided, matches against both name and description (case-insensitive LIKE).
    /// When `pack_type` is provided, filters to that type only.
    /// Results are ordered by created_at descending (newest first).
    pub fn list_packs(
        &self,
        search: Option<&str>,
        pack_type: Option<&str>,
    ) -> Result<Vec<CapabilityPack>> {
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(pt) = pack_type {
            conditions.push(format!("pack_type = ?{}", param_values.len() + 1));
            param_values.push(Box::new(pt.to_string()));
        }

        if let Some(s) = search {
            conditions.push(format!(
                "(name LIKE ?{} OR description LIKE ?{})",
                param_values.len() + 1,
                param_values.len() + 2
            ));
            let pattern = format!("%{}%", s);
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT {} FROM packs {} ORDER BY created_at DESC",
            Self::SELECT_COLS,
            where_clause
        );

        let mut stmt = self.db.conn().prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), Self::row_to_pack)?;

        let mut packs = Vec::new();
        for row in rows {
            packs.push(row?);
        }
        Ok(packs)
    }

    /// Delete a pack by ID.
    ///
    /// Deletes the pack record from the `packs` table and all associated
    /// rows from `capability_resources` in a single transaction.
    /// Returns the deleted pack metadata on success, or `None` if not found.
    pub fn delete_pack(&self, pack_id: &str) -> Result<Option<CapabilityPack>> {
        let pack = self.get_pack_by_id(pack_id)?;
        if pack.is_none() {
            return Ok(None);
        }
        let pack = pack.unwrap();

        let conn = self.db.conn();
        conn.execute_batch("BEGIN")?;
        conn.execute(
            "DELETE FROM capability_resources WHERE pack_id = ?1",
            params![pack_id],
        )?;
        conn.execute("DELETE FROM packs WHERE id = ?1", params![pack_id])?;
        conn.execute_batch("COMMIT")?;

        Ok(Some(pack))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn make_pack(id: &str, name: &str, version: &str, pack_type: &str) -> CapabilityPack {
        CapabilityPack {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: Some(format!("{} pack", name)),
            pack_type: pack_type.to_string(),
            manifest_path: format!("/tmp/packs/{}/pack.manifest.json", name),
            source_repo_id: Some("repo-1".to_string()),
            source_commit: Some("abc123".to_string()),
            created_at: "2026-05-14T00:00:00Z".to_string(),
            storage_dir: format!("/tmp/packs/{}/", name),
        }
    }

    #[test]
    fn test_insert_and_get_by_id() {
        let db = setup_db();
        let store = PackStore::new(&db);

        let pack = make_pack("p1", "test-pack", "1.0.0", "project");
        store.insert_pack(&pack).unwrap();

        let found = store.get_pack_by_id("p1").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-pack");
    }

    #[test]
    fn test_get_by_id_not_found() {
        let db = setup_db();
        let store = PackStore::new(&db);

        let found = store.get_pack_by_id("nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_get_pack_by_name_version() {
        let db = setup_db();
        let store = PackStore::new(&db);

        store
            .insert_pack(&make_pack("p1", "my-pack", "1.0.0", "project"))
            .unwrap();
        store
            .insert_pack(&make_pack("p2", "other-pack", "2.0.0", "blueprint"))
            .unwrap();

        let found = store
            .get_pack_by_name_version("my-pack", "1.0.0")
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "p1");

        let not_found = store
            .get_pack_by_name_version("my-pack", "2.0.0")
            .unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_duplicate_name_version_rejected() {
        let db = setup_db();
        let store = PackStore::new(&db);

        store
            .insert_pack(&make_pack("p1", "dup-pack", "1.0.0", "project"))
            .unwrap();

        let result = store.insert_pack(&make_pack("p2", "dup-pack", "1.0.0", "project"));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_packs_no_filter() {
        let db = setup_db();
        let store = PackStore::new(&db);

        store
            .insert_pack(&make_pack("p1", "pack-a", "1.0.0", "project"))
            .unwrap();
        store
            .insert_pack(&make_pack("p2", "pack-b", "2.0.0", "blueprint"))
            .unwrap();

        let all = store.list_packs(None, None).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_list_packs_filter_by_type() {
        let db = setup_db();
        let store = PackStore::new(&db);

        store
            .insert_pack(&make_pack("p1", "pack-a", "1.0.0", "project"))
            .unwrap();
        store
            .insert_pack(&make_pack("p2", "pack-b", "2.0.0", "blueprint"))
            .unwrap();

        let projects = store.list_packs(None, Some("project")).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "pack-a");
    }

    #[test]
    fn test_list_packs_search() {
        let db = setup_db();
        let store = PackStore::new(&db);

        store
            .insert_pack(&make_pack("p1", "devops-tools", "1.0.0", "project"))
            .unwrap();
        store
            .insert_pack(&make_pack("p2", "frontend-setup", "1.0.0", "blueprint"))
            .unwrap();

        let results = store.list_packs(Some("devops"), None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "devops-tools");
    }

    #[test]
    fn test_delete_pack() {
        let db = setup_db();
        let store = PackStore::new(&db);

        store
            .insert_pack(&make_pack("p1", "delete-me", "1.0.0", "project"))
            .unwrap();

        // Insert a resource linked to the pack
        db.conn()
            .execute(
                "INSERT INTO capability_resources (id, pack_id, type, name, scope) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["res-1", "p1", "skill", "packed-skill", "project"],
            )
            .unwrap();

        let deleted = store.delete_pack("p1").unwrap();
        assert!(deleted.is_some());
        assert_eq!(deleted.unwrap().name, "delete-me");

        // Pack should no longer exist
        let found = store.get_pack_by_id("p1").unwrap();
        assert!(found.is_none());

        // Resources should also be deleted
        let count: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM capability_resources WHERE pack_id = ?1",
                params!["p1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_delete_nonexistent_pack() {
        let db = setup_db();
        let store = PackStore::new(&db);

        let result = store.delete_pack("nonexistent").unwrap();
        assert!(result.is_none());
    }
}
