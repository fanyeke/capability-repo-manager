# Contract: Operation Events

**Purpose**: Define the operation event storage and query interfaces.

## DB Table: operation_events

```sql
CREATE TABLE IF NOT EXISTS operation_events (
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
CREATE INDEX IF NOT EXISTS idx_op_events_type ON operation_events(operation_type);
```

## EventStore Methods

### insert_event(event) → Result
Insert a completed operation event. Called at the end of each user operation.

**Input**: OperationEvent struct with all fields populated.

### list_events(limit, offset, type_filter?) → Vec<OperationEvent>
List operation events in reverse chronological order. Used by the Activity History page.

**Input**: limit (default 50), offset (default 0), optional type filter.
**Output**: List of OperationEvent records.

### get_by_operation(operation_id) → Vec<OperationEvent>
Get all events sharing the same operation_id. Used for trace debugging.

**Input**: operation_id UUID string.
**Output**: All events with that operation_id.

## Tauri Commands

### list_operation_events

```typescript
Input:  { limit?: number; offset?: number; operation_type?: string }
Output: OperationEvent[]
```

Returns operation events for the Activity History page.
