use crate::Database;
use domain::MigrationRun;
use rusqlite::{params, Result};

pub struct MigrationStore<'a> {
    db: &'a Database,
}

impl<'a> MigrationStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert_run(&self, run: &MigrationRun) -> Result<()> {
        self.db.conn().execute(
            "INSERT OR REPLACE INTO migration_runs (id, source_type, source_id, target_repo_id, status, plan_json, report_json, snapshot_path, created_at, executed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                run.id,
                run.source_type,
                run.source_id,
                run.target_repo_id,
                run.status,
                run.plan_json,
                run.report_json,
                run.snapshot_path,
                run.created_at,
                run.executed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_run(&self, run_id: &str) -> Result<Option<MigrationRun>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, source_type, source_id, target_repo_id, status, plan_json, report_json, snapshot_path, created_at, executed_at
             FROM migration_runs WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![run_id], |row| {
            Ok(MigrationRun {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_id: row.get(2)?,
                target_repo_id: row.get(3)?,
                status: row.get(4)?,
                plan_json: row.get(5)?,
                report_json: row.get(6)?,
                snapshot_path: row.get(7)?,
                created_at: row.get(8)?,
                executed_at: row.get(9)?,
            })
        })?;

        match rows.next() {
            Some(Ok(run)) => Ok(Some(run)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn update_status(&self, run_id: &str, status: &str) -> Result<()> {
        self.db.conn().execute("UPDATE migration_runs SET status = ?1 WHERE id = ?2", params![status, run_id])?;
        Ok(())
    }

    pub fn update_snapshot(&self, run_id: &str, snapshot_json: &str) -> Result<()> {
        self.db
            .conn()
            .execute("UPDATE migration_runs SET snapshot_path = ?1 WHERE id = ?2", params![snapshot_json, run_id])?;
        Ok(())
    }

    pub fn update_report(&self, run_id: &str, report_json: &str) -> Result<()> {
        self.db
            .conn()
            .execute("UPDATE migration_runs SET report_json = ?1 WHERE id = ?2", params![report_json, run_id])?;
        Ok(())
    }

    pub fn list_by_repo(&self, repo_id: &str) -> Result<Vec<MigrationRun>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, source_type, source_id, target_repo_id, status, plan_json, report_json, snapshot_path, created_at, executed_at
             FROM migration_runs WHERE target_repo_id = ?1 ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map(params![repo_id], |row| {
            Ok(MigrationRun {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_id: row.get(2)?,
                target_repo_id: row.get(3)?,
                status: row.get(4)?,
                plan_json: row.get(5)?,
                report_json: row.get(6)?,
                snapshot_path: row.get(7)?,
                created_at: row.get(8)?,
                executed_at: row.get(9)?,
            })
        })?;

        let mut runs = Vec::new();
        for row in rows {
            runs.push(row?);
        }
        Ok(runs)
    }
}
