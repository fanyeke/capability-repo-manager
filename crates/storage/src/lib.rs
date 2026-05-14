pub mod repo_store;
pub mod resource_store;

use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS repositories (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                remote_url TEXT,
                current_branch TEXT,
                head_commit TEXT,
                dirty_state TEXT NOT NULL DEFAULT 'unknown',
                last_indexed_at TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS capability_resources (
                id TEXT PRIMARY KEY NOT NULL,
                repo_id TEXT,
                pack_id TEXT,
                type TEXT NOT NULL,
                name TEXT NOT NULL,
                source_path TEXT,
                scope TEXT NOT NULL DEFAULT 'project',
                tracked_by_git INTEGER NOT NULL DEFAULT 0,
                content_hash TEXT,
                metadata_json TEXT,
                error_message TEXT
            );

            CREATE TABLE IF NOT EXISTS packs (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT,
                pack_type TEXT NOT NULL DEFAULT 'project',
                manifest_path TEXT NOT NULL,
                source_repo_id TEXT,
                source_commit TEXT,
                created_at TEXT NOT NULL,
                storage_dir TEXT NOT NULL,
                UNIQUE(name, version)
            );

            CREATE TABLE IF NOT EXISTS migration_runs (
                id TEXT PRIMARY KEY NOT NULL,
                source_type TEXT NOT NULL,
                source_id TEXT NOT NULL,
                target_repo_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'planned',
                plan_json TEXT NOT NULL,
                report_json TEXT,
                snapshot_path TEXT,
                created_at TEXT NOT NULL,
                executed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS doctor_reports (
                id TEXT PRIMARY KEY NOT NULL,
                repo_id TEXT NOT NULL,
                score INTEGER NOT NULL,
                issues_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_repos_path ON repositories(path);
            CREATE INDEX IF NOT EXISTS idx_repos_name ON repositories(name);
            CREATE INDEX IF NOT EXISTS idx_resources_repo ON capability_resources(repo_id);
            CREATE INDEX IF NOT EXISTS idx_resources_pack ON capability_resources(pack_id);
            CREATE INDEX IF NOT EXISTS idx_resources_type ON capability_resources(type);
            CREATE INDEX IF NOT EXISTS idx_packs_name_version ON packs(name, version);
            CREATE INDEX IF NOT EXISTS idx_migrations_target ON migration_runs(target_repo_id);
            CREATE INDEX IF NOT EXISTS idx_doctor_repo ON doctor_reports(repo_id);
            "
        )?;
        Ok(())
    }
}
