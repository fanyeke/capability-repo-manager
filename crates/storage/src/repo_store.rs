use crate::Database;
use domain::Repository;
use rusqlite::{params, Result};

pub struct RepositoryStore<'a> {
    db: &'a Database,
}

impl<'a> RepositoryStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, repo: &Repository) -> Result<()> {
        self.db.conn().execute(
            "INSERT OR REPLACE INTO repositories (id, name, path, remote_url, current_branch, head_commit, dirty_state, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                repo.id,
                repo.name,
                repo.path,
                repo.remote_url,
                repo.current_branch,
                repo.head_commit,
                repo.dirty_state,
                repo.last_indexed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Repository>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, path, remote_url, current_branch, head_commit, dirty_state, last_indexed_at
             FROM repositories WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Repository {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                remote_url: row.get(3)?,
                current_branch: row.get(4)?,
                head_commit: row.get(5)?,
                dirty_state: row.get(6)?,
                last_indexed_at: row.get(7)?,
            })
        })?;

        match rows.next() {
            Some(Ok(repo)) => Ok(Some(repo)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn get_by_path(&self, path: &str) -> Result<Option<Repository>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, path, remote_url, current_branch, head_commit, dirty_state, last_indexed_at
             FROM repositories WHERE path = ?1"
        )?;

        let mut rows = stmt.query_map(params![path], |row| {
            Ok(Repository {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                remote_url: row.get(3)?,
                current_branch: row.get(4)?,
                head_commit: row.get(5)?,
                dirty_state: row.get(6)?,
                last_indexed_at: row.get(7)?,
            })
        })?;

        match rows.next() {
            Some(Ok(repo)) => Ok(Some(repo)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn list_all(&self) -> Result<Vec<Repository>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, path, remote_url, current_branch, head_commit, dirty_state, last_indexed_at
             FROM repositories ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Repository {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                remote_url: row.get(3)?,
                current_branch: row.get(4)?,
                head_commit: row.get(5)?,
                dirty_state: row.get(6)?,
                last_indexed_at: row.get(7)?,
            })
        })?;

        let mut repos = Vec::new();
        for row in rows {
            repos.push(row?);
        }
        Ok(repos)
    }

    pub fn update(&self, repo: &Repository) -> Result<()> {
        self.insert(repo) // INSERT OR REPLACE handles updates
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.db.conn().execute(
            "DELETE FROM repositories WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
}
