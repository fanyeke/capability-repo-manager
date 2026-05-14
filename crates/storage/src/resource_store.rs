use crate::Database;
use domain::CapabilityResource;
use rusqlite::{params, Result};

pub struct ResourceStore<'a> {
    db: &'a Database,
}

impl<'a> ResourceStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert_batch(&self, resources: &[CapabilityResource]) -> Result<()> {
        let mut stmt = self.db.conn().prepare(
            "INSERT OR REPLACE INTO capability_resources
             (id, repo_id, pack_id, type, name, source_path, scope, tracked_by_git, content_hash, metadata_json, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        )?;

        for r in resources {
            stmt.execute(params![
                r.id,
                r.repo_id,
                r.pack_id,
                r.r#type,
                r.name,
                r.source_path,
                r.scope,
                r.tracked_by_git as i32,
                r.content_hash,
                r.metadata_json,
                r.error_message,
            ])?;
        }
        Ok(())
    }

    pub fn get_by_repo(&self, repo_id: &str) -> Result<Vec<CapabilityResource>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, repo_id, pack_id, type, name, source_path, scope, tracked_by_git, content_hash, metadata_json, error_message
             FROM capability_resources WHERE repo_id = ?1 ORDER BY type, name"
        )?;

        let rows = stmt.query_map(params![repo_id], |row| {
            Ok(CapabilityResource {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                pack_id: row.get(2)?,
                r#type: row.get(3)?,
                name: row.get(4)?,
                source_path: row.get(5)?,
                scope: row.get(6)?,
                tracked_by_git: row.get::<_, i32>(7)? != 0,
                content_hash: row.get(8)?,
                metadata_json: row.get(9)?,
                error_message: row.get(10)?,
            })
        })?;

        let mut resources = Vec::new();
        for row in rows {
            resources.push(row?);
        }
        Ok(resources)
    }

    pub fn get_by_pack(&self, pack_id: &str) -> Result<Vec<CapabilityResource>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, repo_id, pack_id, type, name, source_path, scope, tracked_by_git, content_hash, metadata_json, error_message
             FROM capability_resources WHERE pack_id = ?1 ORDER BY type, name"
        )?;

        let rows = stmt.query_map(params![pack_id], |row| {
            Ok(CapabilityResource {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                pack_id: row.get(2)?,
                r#type: row.get(3)?,
                name: row.get(4)?,
                source_path: row.get(5)?,
                scope: row.get(6)?,
                tracked_by_git: row.get::<_, i32>(7)? != 0,
                content_hash: row.get(8)?,
                metadata_json: row.get(9)?,
                error_message: row.get(10)?,
            })
        })?;

        let mut resources = Vec::new();
        for row in rows {
            resources.push(row?);
        }
        Ok(resources)
    }

    pub fn delete_by_repo(&self, repo_id: &str) -> Result<()> {
        self.db.conn().execute(
            "DELETE FROM capability_resources WHERE repo_id = ?1",
            params![repo_id],
        )?;
        Ok(())
    }
}
