use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// A completed operation event stored in the database.
///
/// Used by the Activity History page for querying completed operations,
/// and by the Debug Bundle export for including recent operation summaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationEvent {
    pub id: String,
    pub operation_id: String,
    pub operation_type: String,
    pub status: String,
    pub repo_id: Option<String>,
    pub pack_id: Option<String>,
    pub migration_run_id: Option<String>,
    pub summary: Option<String>,
    pub detail_json: Option<String>,
    pub created_at: String,
}

/// Parameters for inserting a new operation event.
#[derive(Debug)]
pub struct NewOperationEvent {
    pub operation_id: String,
    pub operation_type: String,
    pub status: String,
    pub repo_id: Option<String>,
    pub pack_id: Option<String>,
    pub migration_run_id: Option<String>,
    pub summary: Option<String>,
    pub detail_json: Option<String>,
}

/// Store for reading and writing `operation_events` table records.
pub struct EventStore<'a> {
    conn: &'a Connection,
}

impl<'a> EventStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Insert a new operation event.
    pub fn insert_event(&self, event: NewOperationEvent) -> Result<OperationEvent> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO operation_events (id, operation_id, operation_type, status, repo_id, pack_id, migration_run_id, summary, detail_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                event.operation_id,
                event.operation_type,
                event.status,
                event.repo_id,
                event.pack_id,
                event.migration_run_id,
                event.summary,
                event.detail_json,
                created_at,
            ],
        )?;

        Ok(OperationEvent {
            id,
            operation_id: event.operation_id,
            operation_type: event.operation_type,
            status: event.status,
            repo_id: event.repo_id,
            pack_id: event.pack_id,
            migration_run_id: event.migration_run_id,
            summary: event.summary,
            detail_json: event.detail_json,
            created_at,
        })
    }

    /// List operation events in reverse chronological order.
    pub fn list_events(&self, limit: i64, offset: i64, type_filter: Option<&str>) -> Result<Vec<OperationEvent>> {
        let (sql, param_values): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(t) = type_filter {
            (
                "SELECT id, operation_id, operation_type, status, repo_id, pack_id, migration_run_id, summary, detail_json, created_at
                 FROM operation_events
                 WHERE operation_type = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2 OFFSET ?3".to_string(),
                vec![
                    Box::new(t.to_string()),
                    Box::new(limit),
                    Box::new(offset),
                ],
            )
        } else {
            (
                "SELECT id, operation_id, operation_type, status, repo_id, pack_id, migration_run_id, summary, detail_json, created_at
                 FROM operation_events
                 ORDER BY created_at DESC
                 LIMIT ?1 OFFSET ?2".to_string(),
                vec![
                    Box::new(limit),
                    Box::new(offset),
                ],
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(OperationEvent {
                id: row.get(0)?,
                operation_id: row.get(1)?,
                operation_type: row.get(2)?,
                status: row.get(3)?,
                repo_id: row.get(4)?,
                pack_id: row.get(5)?,
                migration_run_id: row.get(6)?,
                summary: row.get(7)?,
                detail_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    /// Get all events sharing the same operation_id.
    pub fn get_by_operation(&self, operation_id: &str) -> Result<Vec<OperationEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, operation_id, operation_type, status, repo_id, pack_id, migration_run_id, summary, detail_json, created_at
             FROM operation_events
             WHERE operation_id = ?1
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![operation_id], |row| {
            Ok(OperationEvent {
                id: row.get(0)?,
                operation_id: row.get(1)?,
                operation_type: row.get(2)?,
                status: row.get(3)?,
                repo_id: row.get(4)?,
                pack_id: row.get(5)?,
                migration_run_id: row.get(6)?,
                summary: row.get(7)?,
                detail_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS operation_events (
                id TEXT PRIMARY KEY NOT NULL,
                operation_id TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                status TEXT NOT NULL,
                repo_id TEXT,
                pack_id TEXT,
                migration_run_id TEXT,
                summary TEXT,
                detail_json TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_op_events_operation_id ON operation_events(operation_id);
            CREATE INDEX IF NOT EXISTS idx_op_events_created_at ON operation_events(created_at);
            CREATE INDEX IF NOT EXISTS idx_op_events_type ON operation_events(operation_type);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_insert_and_list_event() {
        let conn = setup_db();
        let store = EventStore::new(&conn);

        let event = store
            .insert_event(NewOperationEvent {
                operation_id: "op-123".to_string(),
                operation_type: "scan_repositories".to_string(),
                status: "success".to_string(),
                repo_id: None,
                pack_id: None,
                migration_run_id: None,
                summary: Some("Scanned 3 directories".to_string()),
                detail_json: None,
            })
            .unwrap();

        assert_eq!(event.operation_id, "op-123");
        assert_eq!(event.operation_type, "scan_repositories");
        assert_eq!(event.status, "success");
        assert_eq!(event.summary, Some("Scanned 3 directories".to_string()));

        let events = store.list_events(10, 0, None).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].operation_id, "op-123");
    }

    #[test]
    fn test_list_events_with_type_filter() {
        let conn = setup_db();
        let store = EventStore::new(&conn);

        store
            .insert_event(NewOperationEvent {
                operation_id: "op-1".to_string(),
                operation_type: "scan_repositories".to_string(),
                status: "success".to_string(),
                repo_id: None,
                pack_id: None,
                migration_run_id: None,
                summary: None,
                detail_json: None,
            })
            .unwrap();

        store
            .insert_event(NewOperationEvent {
                operation_id: "op-2".to_string(),
                operation_type: "export_pack".to_string(),
                status: "failure".to_string(),
                repo_id: None,
                pack_id: Some("pack-1".to_string()),
                migration_run_id: None,
                summary: None,
                detail_json: None,
            })
            .unwrap();

        let scan_events = store.list_events(10, 0, Some("scan_repositories")).unwrap();
        assert_eq!(scan_events.len(), 1);
        assert_eq!(scan_events[0].operation_type, "scan_repositories");

        let export_events = store.list_events(10, 0, Some("export_pack")).unwrap();
        assert_eq!(export_events.len(), 1);
        assert_eq!(export_events[0].operation_type, "export_pack");

        let all_events = store.list_events(10, 0, None).unwrap();
        assert_eq!(all_events.len(), 2);
    }

    #[test]
    fn test_get_by_operation() {
        let conn = setup_db();
        let store = EventStore::new(&conn);

        store
            .insert_event(NewOperationEvent {
                operation_id: "op-scan".to_string(),
                operation_type: "scan_repositories".to_string(),
                status: "success".to_string(),
                repo_id: None,
                pack_id: None,
                migration_run_id: None,
                summary: None,
                detail_json: None,
            })
            .unwrap();

        store
            .insert_event(NewOperationEvent {
                operation_id: "op-scan".to_string(),
                operation_type: "scan_repositories".to_string(),
                status: "failure".to_string(),
                repo_id: Some("repo-1".to_string()),
                pack_id: None,
                migration_run_id: None,
                summary: None,
                detail_json: None,
            })
            .unwrap();

        let events = store.get_by_operation("op-scan").unwrap();
        assert_eq!(events.len(), 2);

        let empty = store.get_by_operation("non-existent").unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_list_events_pagination() {
        let conn = setup_db();
        let store = EventStore::new(&conn);

        for i in 0..5 {
            store
                .insert_event(NewOperationEvent {
                    operation_id: format!("op-{}", i),
                    operation_type: "scan_repositories".to_string(),
                    status: "success".to_string(),
                    repo_id: None,
                    pack_id: None,
                    migration_run_id: None,
                    summary: None,
                    detail_json: None,
                })
                .unwrap();
        }

        let first_page = store.list_events(2, 0, None).unwrap();
        assert_eq!(first_page.len(), 2);

        let second_page = store.list_events(2, 2, None).unwrap();
        assert_eq!(second_page.len(), 2);
    }
}
