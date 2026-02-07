//! Schema Repository
//! 
//! This module manages the `schema` table which stores data schema definitions.
//! It supports the hybrid approach where:
//! - Config-defined schemas are synced on every startup (source = "config")
//! - Runtime-created schemas persist across restarts (source = "runtime")
//! 
//! Soft delete support:
//! - Schemas can be soft-deleted (status = "deleted") preserving data
//! - Hard delete (cascade = true) removes schema and all associated data

use rusqlite::{params, Connection, Result, Row};
use crate::data::models::{SystemDataModel, SystemDataModelField, SchemaStatus};

/// Database name where the schema table lives
const REGISTRY_DB: &str = "system";
/// Table name for storing schema definitions
const SCHEMA_TABLE: &str = "schema";

// ============================================================================
// TABLE INITIALIZATION
// ============================================================================

/// Creates the `schema` table if it doesn't exist.
/// 
/// Schema:
/// - id: Primary key (auto-increment)
/// - name: Unique schema name
/// - version: Schema version for migrations
/// - db: Target database (e.g., "public", "private")
/// - public: Whether the schema is publicly accessible (default: false)
/// - icon: Icon for the schema (emoji or identifier, e.g., "📦", "mdi-account")
/// - fields: JSON array of field definitions
/// - source: Origin of the schema ("config" or "runtime")
/// - status: Schema status ("active" or "deleted")
/// - deleted_at: Timestamp when schema was soft-deleted
/// - created_at: Timestamp when the schema was created
/// - updated_at: Timestamp when the schema was last updated
pub fn init_schema_table() -> Result<()> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let sql = r#"
        CREATE TABLE IF NOT EXISTS schema (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT UNIQUE NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            db TEXT NOT NULL DEFAULT 'public',
            public INTEGER NOT NULL DEFAULT 0,
            icon TEXT,
            fields TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'runtime',
            status TEXT NOT NULL DEFAULT 'active',
            deleted_at TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
    "#;
    
    conn.execute(sql, [])?;
    
    // Migrations for existing databases
    let _ = conn.execute("ALTER TABLE schema ADD COLUMN icon TEXT", []);
    let _ = conn.execute("ALTER TABLE schema ADD COLUMN status TEXT NOT NULL DEFAULT 'active'", []);
    let _ = conn.execute("ALTER TABLE schema ADD COLUMN deleted_at TEXT", []);
    
    Ok(())
}

// ============================================================================
// CRUD OPERATIONS
// ============================================================================

/// Inserts or updates a schema in the repository.
/// 
/// For config-sourced schemas, this will update existing entries.
/// For runtime schemas, it will fail if a schema with the same name exists.
/// 
/// # Arguments
/// * `schema` - The schema definition to upsert
/// * `source` - The source of the schema ("config" or "runtime")
/// 
/// # Returns
/// The ID of the inserted/updated schema
pub fn upsert_schema(schema: &SystemDataModel, source: &str) -> Result<u64> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let fields_json = serde_json::to_string(&schema.fields)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let sql = r#"
        INSERT INTO schema (name, version, db, public, icon, fields, source, status, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', CURRENT_TIMESTAMP)
        ON CONFLICT(name) DO UPDATE SET
            version = excluded.version,
            db = excluded.db,
            public = excluded.public,
            icon = excluded.icon,
            fields = excluded.fields,
            source = CASE 
                WHEN schema.source = 'config' THEN excluded.source 
                ELSE schema.source 
            END,
            status = 'active',
            deleted_at = NULL,
            updated_at = CURRENT_TIMESTAMP
        WHERE schema.source = 'config' OR excluded.source = 'runtime'
    "#;
    
    conn.execute(sql, params![
        schema.name,
        schema.version,
        schema.db,
        schema.public as i32,
        schema.icon,
        fields_json,
        source
    ])?;
    
    Ok(conn.last_insert_rowid() as u64)
}

/// Inserts a new runtime schema. Fails if schema already exists.
/// 
/// # Arguments
/// * `schema` - The schema definition to insert
/// 
/// # Returns
/// The ID of the inserted schema, or error if name already exists
pub fn insert_runtime_schema(schema: &SystemDataModel) -> Result<u64> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let fields_json = serde_json::to_string(&schema.fields)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let sql = r#"
        INSERT INTO schema (name, version, db, public, icon, fields, source, status)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'runtime', 'active')
    "#;
    
    conn.execute(sql, params![
        schema.name,
        schema.version,
        schema.db,
        schema.public as i32,
        schema.icon,
        fields_json
    ])?;
    
    Ok(conn.last_insert_rowid() as u64)
}

/// Updates an existing schema by name.
/// 
/// # Arguments
/// * `name` - The name of the schema to update
/// * `schema` - The new schema definition
/// 
/// # Returns
/// Number of rows affected (1 on success, 0 if not found)
pub fn update_schema(name: &str, schema: &SystemDataModel) -> Result<usize> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let fields_json = serde_json::to_string(&schema.fields)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let sql = r#"
        UPDATE schema 
        SET version = ?1, db = ?2, public = ?3, icon = ?4, fields = ?5, updated_at = CURRENT_TIMESTAMP
        WHERE name = ?6 AND status = 'active'
    "#;
    
    conn.execute(sql, params![
        schema.version,
        schema.db,
        schema.public as i32,
        schema.icon,
        fields_json,
        name
    ])
}

/// Soft deletes a schema by name.
/// Sets status to 'deleted' and records deletion timestamp.
/// Data remains intact and can be restored.
/// 
/// # Arguments
/// * `name` - The name of the schema to soft delete
/// 
/// # Returns
/// Number of rows affected (1 on success, 0 if not found)
pub fn soft_delete_schema(name: &str) -> Result<usize> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    conn.execute(
        "UPDATE schema SET status = 'deleted', deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE name = ?1 AND status = 'active'",
        params![name]
    )
}

/// Hard deletes a schema by name.
/// Permanently removes the schema from the repository.
/// Use with cascade to also drop the data table.
/// 
/// # Arguments
/// * `name` - The name of the schema to permanently delete
/// 
/// # Returns
/// Number of rows affected (1 on success, 0 if not found)
pub fn hard_delete_schema(name: &str) -> Result<usize> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    conn.execute("DELETE FROM schema WHERE name = ?1", params![name])
}

/// Restores a soft-deleted schema.
/// Sets status back to 'active' and clears deletion timestamp.
/// 
/// # Arguments
/// * `name` - The name of the schema to restore
/// 
/// # Returns
/// Number of rows affected (1 on success, 0 if not found or not deleted)
pub fn restore_schema(name: &str) -> Result<usize> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    conn.execute(
        "UPDATE schema SET status = 'active', deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE name = ?1 AND status = 'deleted'",
        params![name]
    )
}

/// Legacy delete function - now performs soft delete by default.
/// Use hard_delete_schema() for permanent deletion.
/// 
/// # Arguments
/// * `name` - The name of the schema to delete
/// 
/// # Returns
/// Number of rows affected (1 on success, 0 if not found)
pub fn delete_schema(name: &str) -> Result<usize> {
    soft_delete_schema(name)
}

// ============================================================================
// QUERY OPERATIONS
// ============================================================================

/// Retrieves an active schema by name.
/// Does not return soft-deleted schemas.
/// 
/// # Arguments
/// * `name` - The schema name to look up
/// 
/// # Returns
/// The schema if found and active, or QueryReturnedNoRows error if not
pub fn get_schema_by_name(name: &str) -> Result<SystemDataModel> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    conn.query_row(
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE name = ?1 AND status = 'active'",
        params![name],
        row_to_schema
    )
}

/// Retrieves a schema by name regardless of status (including deleted).
/// 
/// # Arguments
/// * `name` - The schema name to look up
/// 
/// # Returns
/// The schema if found (active or deleted), or QueryReturnedNoRows error if not
pub fn get_schema_by_name_include_deleted(name: &str) -> Result<SystemDataModel> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    conn.query_row(
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE name = ?1",
        params![name],
        row_to_schema
    )
}

/// Retrieves all active schemas from the repository.
/// Does not include soft-deleted schemas.
/// 
/// # Returns
/// Vector of all active schemas
pub fn get_all_schemas() -> Result<Vec<SystemDataModel>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE status = 'active' ORDER BY name"
    )?;
    
    let rows = stmt.query_map([], row_to_schema)?;
    rows.collect()
}

/// Retrieves all schemas including soft-deleted ones.
/// 
/// # Arguments
/// * `include_deleted` - If true, includes soft-deleted schemas
/// 
/// # Returns
/// Vector of schemas (optionally including deleted)
pub fn get_all_schemas_with_deleted(include_deleted: bool) -> Result<Vec<SystemDataModel>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let sql = if include_deleted {
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema ORDER BY name"
    } else {
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE status = 'active' ORDER BY name"
    };
    
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], row_to_schema)?;
    rows.collect()
}

/// Retrieves all soft-deleted schemas.
/// 
/// # Returns
/// Vector of deleted schemas
pub fn get_deleted_schemas() -> Result<Vec<SystemDataModel>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE status = 'deleted' ORDER BY deleted_at DESC"
    )?;
    
    let rows = stmt.query_map([], row_to_schema)?;
    rows.collect()
}

/// Retrieves schemas by source (e.g., "config" or "runtime").
/// Only returns active schemas.
/// 
/// # Arguments
/// * `source` - The source filter
/// 
/// # Returns
/// Vector of active schemas matching the source
pub fn get_schemas_by_source(source: &str) -> Result<Vec<SystemDataModel>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE source = ?1 AND status = 'active' ORDER BY name"
    )?;
    
    let rows = stmt.query_map(params![source], row_to_schema)?;
    rows.collect()
}

/// Retrieves all public active schemas.
/// 
/// # Returns
/// Vector of active schemas where public = true
pub fn get_public_schemas() -> Result<Vec<SystemDataModel>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE public = 1 AND status = 'active' ORDER BY name"
    )?;
    
    let rows = stmt.query_map([], row_to_schema)?;
    rows.collect()
}

/// Retrieves active schemas by target database.
/// 
/// # Arguments
/// * `db` - The database name filter (e.g., "public", "private")
/// 
/// # Returns
/// Vector of active schemas targeting the specified database
pub fn get_schemas_by_db(db: &str) -> Result<Vec<SystemDataModel>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, version, db, public, icon, fields, source, status, deleted_at FROM schema WHERE db = ?1 AND status = 'active' ORDER BY name"
    )?;
    
    let rows = stmt.query_map(params![db], row_to_schema)?;
    rows.collect()
}

/// Counts active schemas in the repository.
/// 
/// # Returns
/// Number of active schemas
pub fn count_schemas() -> Result<u32> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    conn.query_row("SELECT COUNT(*) FROM schema WHERE status = 'active'", [], |row| row.get(0))
}

/// Counts deleted schemas in the repository.
/// 
/// # Returns
/// Number of deleted schemas
pub fn count_deleted_schemas() -> Result<u32> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    conn.query_row("SELECT COUNT(*) FROM schema WHERE status = 'deleted'", [], |row| row.get(0))
}

/// Checks if an active schema exists by name.
/// 
/// # Arguments
/// * `name` - The schema name to check
/// 
/// # Returns
/// true if an active schema exists, false otherwise
pub fn schema_exists(name: &str) -> Result<bool> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM schema WHERE name = ?1 AND status = 'active'",
        params![name],
        |row| row.get(0)
    )?;
    Ok(count > 0)
}

/// Checks if schema is soft-deleted.
/// 
/// # Arguments
/// * `name` - The schema name to check
/// 
/// # Returns
/// true if schema exists and is deleted, false otherwise
pub fn is_schema_deleted(name: &str) -> Result<bool> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM schema WHERE name = ?1 AND status = 'deleted'",
        params![name],
        |row| row.get(0)
    )?;
    Ok(count > 0)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Maps a database row to a SystemDataModel.
/// Column order: name, version, db, public, icon, fields, source, status, deleted_at
fn row_to_schema(row: &Row) -> Result<SystemDataModel> {
    let fields_json: String = row.get(5)?;
    let fields: Vec<SystemDataModelField> = serde_json::from_str(&fields_json)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    let public_int: i32 = row.get(3)?;
    let icon: Option<String> = row.get(4)?;
    let source: String = row.get(6)?;
    let status_str: String = row.get(7)?;
    let deleted_at: Option<String> = row.get(8)?;
    
    Ok(SystemDataModel {
        name: row.get(0)?,
        version: row.get(1)?,
        db: row.get(2)?,
        public: public_int != 0,
        source: Some(source),
        icon,
        status: SchemaStatus::from_str(&status_str),
        deleted_at,
        fields,
    })
}

#[cfg(test)]
#[path = "repository_schema.test.rs"]
mod tests;
