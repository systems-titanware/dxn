//! Event Store Repository
//! 
//! This module provides event sourcing capabilities for the DXN system.
//! All state changes are recorded as immutable events, enabling:
//! - Full audit trail of all changes
//! - Time-travel debugging
//! - Event replay and state reconstruction
//! - Analytics and reporting on changes

use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use crate::data::models::{Event, EventType, EventQueryParams};

/// Database name for the event store
const EVENT_STORE_DB: &str = "system";
/// Table name for events
const EVENTS_TABLE: &str = "events";

// ============================================================================
// TABLE INITIALIZATION
// ============================================================================

/// Creates the events table if it doesn't exist.
/// 
/// The events table stores all domain events with:
/// - Unique event ID
/// - Aggregate ID (the entity being modified)
/// - Schema name (entity type)
/// - Event type (created, updated, deleted, custom)
/// - Payload (the event data as JSON)
/// - Previous state (optional snapshot before change)
/// - Version (for optimistic concurrency)
/// - User ID (who triggered the event)
/// - Timestamp
pub fn init_events_table() -> Result<()> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    let sql = r#"
        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY NOT NULL,
            aggregate_id TEXT NOT NULL,
            schema_name TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            previous_state TEXT,
            version INTEGER NOT NULL,
            user_id TEXT,
            timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
            
            UNIQUE(aggregate_id, version)
        );
        
        CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_id);
        CREATE INDEX IF NOT EXISTS idx_events_schema ON events(schema_name);
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
        CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
    "#;
    
    conn.execute_batch(sql)?;
    Ok(())
}

// ============================================================================
// APPEND OPERATIONS
// ============================================================================

/// Appends a new event to the event store.
/// 
/// # Arguments
/// * `event` - The event to append
/// 
/// # Returns
/// The ID of the appended event
/// 
/// # Errors
/// Returns error if version conflict occurs (optimistic concurrency)
pub fn append_event(event: &Event) -> Result<String> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    let payload_json = serde_json::to_string(&event.payload)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let previous_state_json = event.previous_state.as_ref()
        .map(|s| serde_json::to_string(s))
        .transpose()
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let sql = r#"
        INSERT INTO events (id, aggregate_id, schema_name, event_type, payload, previous_state, version, user_id, timestamp)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
    "#;
    
    conn.execute(sql, params![
        event.id,
        event.aggregate_id,
        event.schema_name,
        event.event_type.as_str(),
        payload_json,
        previous_state_json,
        event.version,
        event.user_id,
        event.timestamp,
    ])?;
    
    Ok(event.id.clone())
}

/// Creates and appends a new event with auto-generated ID and timestamp.
/// 
/// # Arguments
/// * `aggregate_id` - The entity ID
/// * `schema_name` - The schema/entity type
/// * `event_type` - Type of event
/// * `payload` - Event data
/// * `previous_state` - Optional state before change
/// * `user_id` - Optional user who triggered the event
/// 
/// # Returns
/// The created Event
pub fn create_and_append_event(
    aggregate_id: &str,
    schema_name: &str,
    event_type: EventType,
    payload: serde_json::Value,
    previous_state: Option<serde_json::Value>,
    user_id: Option<String>,
) -> Result<Event> {
    // Get the next version for this aggregate
    let version = get_next_version(aggregate_id)?;
    
    let event = Event {
        id: Uuid::now_v7().to_string(),
        aggregate_id: aggregate_id.to_string(),
        schema_name: schema_name.to_string(),
        event_type,
        payload,
        previous_state,
        version,
        user_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    append_event(&event)?;
    Ok(event)
}

// ============================================================================
// QUERY OPERATIONS
// ============================================================================

/// Gets the next version number for an aggregate.
/// 
/// # Arguments
/// * `aggregate_id` - The entity ID
/// 
/// # Returns
/// The next version number (1 if no events exist)
pub fn get_next_version(aggregate_id: &str) -> Result<u32> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    let result: Option<u32> = conn.query_row(
        "SELECT MAX(version) FROM events WHERE aggregate_id = ?1",
        params![aggregate_id],
        |row| row.get(0),
    ).ok();
    
    Ok(result.unwrap_or(0) + 1)
}

/// Gets all events for a specific aggregate (entity).
/// 
/// # Arguments
/// * `aggregate_id` - The entity ID
/// 
/// # Returns
/// Vector of events ordered by version
pub fn get_events_by_aggregate(aggregate_id: &str) -> Result<Vec<Event>> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, aggregate_id, schema_name, event_type, payload, previous_state, version, user_id, timestamp 
         FROM events 
         WHERE aggregate_id = ?1 
         ORDER BY version ASC"
    )?;
    
    let rows = stmt.query_map(params![aggregate_id], row_to_event)?;
    rows.collect()
}

/// Gets all events for a specific schema type.
/// 
/// # Arguments
/// * `schema_name` - The schema/entity type
/// * `params` - Optional query parameters for filtering
/// 
/// # Returns
/// Vector of events matching the criteria
pub fn get_events_by_schema(schema_name: &str, query_params: Option<&EventQueryParams>) -> Result<Vec<Event>> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    let mut sql = String::from(
        "SELECT id, aggregate_id, schema_name, event_type, payload, previous_state, version, user_id, timestamp 
         FROM events 
         WHERE schema_name = ?1"
    );
    
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(schema_name.to_string())];
    
    if let Some(params) = query_params {
        if let Some(since) = &params.since {
            sql.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(since.clone()));
        }
        if let Some(until) = &params.until {
            sql.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(until.clone()));
        }
        if let Some(event_type) = &params.event_type {
            sql.push_str(" AND event_type = ?");
            params_vec.push(Box::new(event_type.clone()));
        }
    }
    
    sql.push_str(" ORDER BY timestamp DESC");
    
    if let Some(params) = query_params {
        if let Some(limit) = params.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = params.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
    }
    
    let mut stmt = conn.prepare(&sql)?;
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), row_to_event)?;
    rows.collect()
}

/// Gets recent events across all schemas.
/// 
/// # Arguments
/// * `limit` - Maximum number of events to return
/// 
/// # Returns
/// Vector of recent events
pub fn get_recent_events(limit: u32) -> Result<Vec<Event>> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, aggregate_id, schema_name, event_type, payload, previous_state, version, user_id, timestamp 
         FROM events 
         ORDER BY timestamp DESC 
         LIMIT ?1"
    )?;
    
    let rows = stmt.query_map(params![limit], row_to_event)?;
    rows.collect()
}

/// Gets a single event by ID.
/// 
/// # Arguments
/// * `event_id` - The event ID
/// 
/// # Returns
/// The event if found
pub fn get_event_by_id(event_id: &str) -> Result<Event> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    conn.query_row(
        "SELECT id, aggregate_id, schema_name, event_type, payload, previous_state, version, user_id, timestamp 
         FROM events 
         WHERE id = ?1",
        params![event_id],
        row_to_event,
    )
}

/// Counts total events for a schema.
/// 
/// # Arguments
/// * `schema_name` - The schema/entity type
/// 
/// # Returns
/// Total event count
pub fn count_events_by_schema(schema_name: &str) -> Result<u32> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    conn.query_row(
        "SELECT COUNT(*) FROM events WHERE schema_name = ?1",
        params![schema_name],
        |row| row.get(0),
    )
}

/// Counts total events for an aggregate.
/// 
/// # Arguments
/// * `aggregate_id` - The entity ID
/// 
/// # Returns
/// Total event count
pub fn count_events_by_aggregate(aggregate_id: &str) -> Result<u32> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    conn.query_row(
        "SELECT COUNT(*) FROM events WHERE aggregate_id = ?1",
        params![aggregate_id],
        |row| row.get(0),
    )
}

// ============================================================================
// REPLAY OPERATIONS
// ============================================================================

/// Replays all events for an aggregate to reconstruct its current state.
/// 
/// This is useful for:
/// - Debugging state issues
/// - Verifying data integrity
/// - Building read models
/// 
/// # Arguments
/// * `aggregate_id` - The entity ID
/// 
/// # Returns
/// The reconstructed state as JSON, or None if no events exist
pub fn replay_aggregate(aggregate_id: &str) -> Result<Option<serde_json::Value>> {
    let events = get_events_by_aggregate(aggregate_id)?;
    
    if events.is_empty() {
        return Ok(None);
    }
    
    // Save the last version before consuming events in the loop
    let last_version = events.last().map(|e| e.version).unwrap_or(0);
    
    let mut state = serde_json::Map::new();
    
    for event in events {
        match event.event_type {
            EventType::Created | EventType::Updated => {
                // Merge payload into state
                if let serde_json::Value::Object(payload_map) = event.payload {
                    for (key, value) in payload_map {
                        state.insert(key, value);
                    }
                }
            }
            EventType::Deleted => {
                // Mark as deleted but keep the state for reference
                state.insert("_deleted".to_string(), serde_json::Value::Bool(true));
                state.insert("_deleted_at".to_string(), serde_json::Value::String(event.timestamp));
            }
            EventType::Custom(_) => {
                // Custom events may modify state in domain-specific ways
                // For now, just merge the payload
                if let serde_json::Value::Object(payload_map) = event.payload {
                    for (key, value) in payload_map {
                        state.insert(key, value);
                    }
                }
            }
        }
    }
    
    // Add metadata
    state.insert("_aggregate_id".to_string(), serde_json::Value::String(aggregate_id.to_string()));
    state.insert("_version".to_string(), serde_json::Value::Number(last_version.into()));
    
    Ok(Some(serde_json::Value::Object(state)))
}

// ============================================================================
// REBUILD OPERATIONS
// ============================================================================

/// Result of a rebuild operation
#[derive(Debug)]
pub struct RebuildResult {
    /// Number of records rebuilt
    pub records_rebuilt: u32,
    /// Number of events processed
    pub events_processed: u32,
    /// IDs of records that were deleted (soft-deleted via events)
    pub deleted_records: Vec<String>,
}

/// Gets all unique aggregate IDs for a schema.
/// 
/// # Arguments
/// * `schema_name` - The schema/entity type
/// 
/// # Returns
/// Vector of unique aggregate IDs
pub fn get_unique_aggregates_for_schema(schema_name: &str) -> Result<Vec<String>> {
    let conn = Connection::open(format!("{}.db", EVENT_STORE_DB))?;
    
    let mut stmt = conn.prepare(
        "SELECT DISTINCT aggregate_id FROM events WHERE schema_name = ?1"
    )?;
    
    let rows = stmt.query_map(params![schema_name], |row| row.get(0))?;
    rows.collect()
}

/// Rebuilds a schema's data table from events.
/// 
/// This function:
/// 1. Gets all unique aggregates for the schema
/// 2. Clears/truncates the existing data table
/// 3. Replays events for each aggregate to reconstruct records
/// 4. Inserts reconstructed records into the data table
/// 
/// **WARNING**: This is a destructive operation that will replace all existing
/// data in the table with state derived from events.
/// 
/// # Arguments
/// * `schema_name` - The schema/entity type to rebuild
/// * `db_name` - The database name (e.g., "public")
/// 
/// # Returns
/// `RebuildResult` with statistics about the operation
pub fn rebuild_schema_from_events(schema_name: &str, db_name: &str) -> Result<RebuildResult> {
    use crate::data::db::sqlite::repository;
    
    // Step 1: Get all unique aggregates for this schema
    let aggregate_ids = get_unique_aggregates_for_schema(schema_name)?;
    
    if aggregate_ids.is_empty() {
        return Ok(RebuildResult {
            records_rebuilt: 0,
            events_processed: 0,
            deleted_records: vec![],
        });
    }
    
    // Step 2: Clear the existing table
    let data_conn = Connection::open(format!("{}.db", db_name))?;
    data_conn.execute(&format!("DELETE FROM {}", schema_name), [])?;
    
    // Step 3: Replay each aggregate and insert into table
    let mut records_rebuilt = 0u32;
    let mut events_processed = 0u32;
    let mut deleted_records = Vec::new();
    
    for aggregate_id in aggregate_ids {
        // Get events for this aggregate
        let events = get_events_by_aggregate(&aggregate_id)?;
        events_processed += events.len() as u32;
        
        // Replay to get final state
        if let Some(state) = replay_aggregate(&aggregate_id)? {
            // Check if the record was deleted
            if state.get("_deleted").and_then(|v| v.as_bool()).unwrap_or(false) {
                deleted_records.push(aggregate_id.clone());
                continue; // Don't insert deleted records
            }
            
            // Extract fields for insertion (excluding metadata fields)
            if let serde_json::Value::Object(map) = state {
                let mut keys: Vec<String> = Vec::new();
                let mut values: Vec<serde_json::Value> = Vec::new();
                
                for (key, value) in map {
                    // Skip internal metadata fields
                    if key.starts_with('_') {
                        continue;
                    }
                    keys.push(key);
                    values.push(value);
                }
                
                // Ensure we have the aggregate_id as 'id'
                if !keys.iter().any(|k| k == "id") {
                    keys.insert(0, "id".to_string());
                    values.insert(0, serde_json::Value::String(aggregate_id.clone()));
                }
                
                // Insert the reconstructed record
                match repository::insert(
                    db_name.to_string(),
                    schema_name.to_string(),
                    keys,
                    values,
                ) {
                    Ok(_) => records_rebuilt += 1,
                    Err(e) => {
                        eprintln!("Warning: Failed to rebuild record {}: {}", aggregate_id, e);
                    }
                }
            }
        }
    }
    
    Ok(RebuildResult {
        records_rebuilt,
        events_processed,
        deleted_records,
    })
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Maps a database row to an Event struct.
fn row_to_event(row: &rusqlite::Row) -> Result<Event> {
    let payload_str: String = row.get(4)?;
    let payload: serde_json::Value = serde_json::from_str(&payload_str)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let previous_state_str: Option<String> = row.get(5)?;
    let previous_state: Option<serde_json::Value> = previous_state_str
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let event_type_str: String = row.get(3)?;
    
    Ok(Event {
        id: row.get(0)?,
        aggregate_id: row.get(1)?,
        schema_name: row.get(2)?,
        event_type: EventType::from_str(&event_type_str),
        payload,
        previous_state,
        version: row.get(6)?,
        user_id: row.get(7)?,
        timestamp: row.get(8)?,
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
#[path = "repository_events.test.rs"]
mod tests;
