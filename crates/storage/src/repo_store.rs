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

    fn row_to_repo(row: &rusqlite::Row) -> rusqlite::Result<Repository> {
        Ok(Repository {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            canonical_path: row.get(3)?,
            remote_url: row.get(4)?,
            current_branch: row.get(5)?,
            head_commit: row.get(6)?,
            dirty_state: row.get(7)?,
            first_indexed_at: row.get(8)?,
            last_indexed_at: row.get(9)?,
            capability_index_status: row.get(10)?,
            last_capability_indexed_at: row.get(11)?,
            last_capability_error: row.get(12)?,
        })
    }

    const SELECT_COLS: &'static str =
        "id, name, path, canonical_path, remote_url, current_branch, head_commit, \
         dirty_state, first_indexed_at, last_indexed_at, capability_index_status, \
         last_capability_indexed_at, last_capability_error";

    pub fn insert(&self, repo: &Repository) -> Result<()> {
        self.db.conn().execute(
            "INSERT OR REPLACE INTO repositories (id, name, path, canonical_path, remote_url, current_branch, head_commit, dirty_state, first_indexed_at, last_indexed_at, capability_index_status, last_capability_indexed_at, last_capability_error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                repo.id,
                repo.name,
                repo.path,
                repo.canonical_path,
                repo.remote_url,
                repo.current_branch,
                repo.head_commit,
                repo.dirty_state,
                repo.first_indexed_at,
                repo.last_indexed_at,
                repo.capability_index_status,
                repo.last_capability_indexed_at,
                repo.last_capability_error,
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Repository>> {
        let sql = format!(
            "SELECT {} FROM repositories WHERE id = ?1",
            Self::SELECT_COLS
        );
        let mut stmt = self.db.conn().prepare(&sql)?;

        let mut rows = stmt.query_map(params![id], Self::row_to_repo)?;

        match rows.next() {
            Some(Ok(repo)) => Ok(Some(repo)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn get_by_path(&self, path: &str) -> Result<Option<Repository>> {
        let sql = format!(
            "SELECT {} FROM repositories WHERE path = ?1",
            Self::SELECT_COLS
        );
        let mut stmt = self.db.conn().prepare(&sql)?;

        let mut rows = stmt.query_map(params![path], Self::row_to_repo)?;

        match rows.next() {
            Some(Ok(repo)) => Ok(Some(repo)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Look up a repository by its canonical (symlink-resolved) path.
    pub fn get_by_canonical_path(&self, canonical_path: &str) -> Result<Option<Repository>> {
        let sql = format!(
            "SELECT {} FROM repositories WHERE canonical_path = ?1",
            Self::SELECT_COLS
        );
        let mut stmt = self.db.conn().prepare(&sql)?;

        let mut rows = stmt.query_map(params![canonical_path], Self::row_to_repo)?;

        match rows.next() {
            Some(Ok(repo)) => Ok(Some(repo)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Upsert a repository by its canonical path.
    ///
    /// If a repo with the same canonical_path already exists, updates its metadata
    /// and returns the existing repo. Otherwise inserts a new record.
    pub fn upsert_by_path(&self, repo: &Repository) -> Result<Repository> {
        // Look up by canonical path first
        if let Some(mut existing) = self.get_by_canonical_path(&repo.canonical_path)? {
            // Existing repo found — update metadata, preserve original ID and first_indexed_at
            existing.name = repo.name.clone();
            existing.path = repo.path.clone();
            existing.remote_url = repo.remote_url.clone();
            existing.current_branch = repo.current_branch.clone();
            existing.head_commit = repo.head_commit.clone();
            existing.dirty_state = repo.dirty_state.clone();
            existing.last_indexed_at = repo.last_indexed_at.clone();
            self.insert(&existing)?;
            Ok(existing)
        } else {
            // New repo — insert with new UUID if id is empty
            let mut new_repo = repo.clone();
            if new_repo.id.is_empty() {
                new_repo.id = uuid::Uuid::new_v4().to_string();
            }
            new_repo.first_indexed_at = chrono::Utc::now().to_rfc3339();
            new_repo.last_indexed_at = new_repo.first_indexed_at.clone();
            self.insert(&new_repo)?;
            Ok(new_repo)
        }
    }

    pub fn list_all(&self) -> Result<Vec<Repository>> {
        let sql = format!(
            "SELECT {} FROM repositories ORDER BY name",
            Self::SELECT_COLS
        );
        let mut stmt = self.db.conn().prepare(&sql)?;

        let rows = stmt.query_map([], Self::row_to_repo)?;

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

    /// Delete a repository and all associated resources and doctor reports
    /// in a single transaction. Migration history is preserved (target marked deleted).
    pub fn delete_cascade(&self, id: &str) -> Result<()> {
        let conn = self.db.conn();
        conn.execute_batch("BEGIN")?;

        if let Err(e) = conn.execute(
            "DELETE FROM capability_resources WHERE repo_id = ?1",
            params![id],
        ) {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e);
        }
        if let Err(e) = conn.execute(
            "DELETE FROM doctor_reports WHERE repo_id = ?1",
            params![id],
        ) {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e);
        }
        if let Err(e) = conn.execute(
            "DELETE FROM repositories WHERE id = ?1",
            params![id],
        ) {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e);
        }
        conn.execute_batch("COMMIT")?;
        Ok(())
    }

    /// Update the capability index status for a repository.
    ///
    /// Sets `capability_index_status`, optionally records an error message,
    /// and updates `last_capability_indexed_at` when status is `fresh`.
    pub fn update_index_status(&self, id: &str, status: &str, error: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.conn().execute(
            "UPDATE repositories SET capability_index_status = ?1, last_capability_error = ?2, \
             last_capability_indexed_at = CASE WHEN ?3 = 'fresh' THEN ?4 ELSE last_capability_indexed_at END \
             WHERE id = ?5",
            params![status, error, status, now, id],
        )?;
        Ok(())
    }
}
