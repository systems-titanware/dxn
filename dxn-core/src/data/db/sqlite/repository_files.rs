//! File Directory Repository
//! 
//! This module manages the `file_directories` table which stores file directory configurations.
//! It supports the hybrid approach where:
//! - Config-defined directories are synced on every startup (source = "config")
//! - Runtime-created directories persist across restarts (source = "runtime")

use rusqlite::{params, Connection, Result, Row};
use crate::data::models::SystemFileDirectory;

/// Database name where the file directories table lives
const REGISTRY_DB: &str = "system";
/// Table name for storing file directory definitions
const FILES_TABLE: &str = "file_directories";

// ============================================================================
// TABLE INITIALIZATION
// ============================================================================

/// Creates the `file_directories` table if it doesn't exist.
/// 
/// Schema:
/// - id: Primary key (auto-increment)
/// - name: Unique directory name
/// - provider: Provider type (local, sftp, s3, etc.)
/// - path: Base path for the directory
/// - icon: Optional icon for UI display
/// - config: JSON configuration for provider-specific settings
/// - source: Origin of the directory ("config" or "runtime")
/// - created_at: Timestamp when the directory was created
/// - updated_at: Timestamp when the directory was last updated
pub fn init_files_table() -> Result<()> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let sql = r#"
        CREATE TABLE IF NOT EXISTS file_directories (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT UNIQUE NOT NULL,
            provider TEXT NOT NULL DEFAULT 'local',
            path TEXT NOT NULL,
            icon TEXT,
            config TEXT,
            source TEXT NOT NULL DEFAULT 'runtime',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
    "#;
    
    conn.execute(sql, [])?;
    Ok(())
}

// ============================================================================
// CRUD OPERATIONS
// ============================================================================

/// Inserts or updates a directory in the repository.
/// 
/// For config-sourced directories, this will update existing entries.
/// For runtime directories, it will fail if a directory with the same name exists.
/// 
/// # Arguments
/// * `directory` - The directory definition to upsert
/// * `source` - The source of the directory ("config" or "runtime")
/// 
/// # Returns
/// The ID of the inserted/updated directory
pub fn upsert_directory(directory: &SystemFileDirectory, source: &str) -> Result<u64> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let config_json = directory.config.as_ref()
        .map(|c| serde_json::to_string(c).ok())
        .flatten();
    
    let sql = r#"
        INSERT INTO file_directories (name, provider, path, icon, config, source, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
        ON CONFLICT(name) DO UPDATE SET
            provider = excluded.provider,
            path = excluded.path,
            icon = excluded.icon,
            config = excluded.config,
            source = CASE 
                WHEN file_directories.source = 'config' THEN excluded.source 
                ELSE file_directories.source 
            END,
            updated_at = CURRENT_TIMESTAMP
        WHERE file_directories.source = 'config' OR excluded.source = 'runtime'
    "#;
    
    conn.execute(sql, params![
        directory.name,
        directory.provider,
        directory.path,
        directory.icon,
        config_json,
        source
    ])?;
    
    Ok(conn.last_insert_rowid() as u64)
}

/// Inserts a new runtime directory. Fails if directory already exists.
/// 
/// # Arguments
/// * `directory` - The directory definition to insert
/// 
/// # Returns
/// The ID of the inserted directory, or error if name already exists
pub fn insert_runtime_directory(directory: &SystemFileDirectory) -> Result<u64> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let config_json = directory.config.as_ref()
        .map(|c| serde_json::to_string(c).ok())
        .flatten();
    
    let sql = r#"
        INSERT INTO file_directories (name, provider, path, icon, config, source)
        VALUES (?1, ?2, ?3, ?4, ?5, 'runtime')
    "#;
    
    conn.execute(sql, params![
        directory.name,
        directory.provider,
        directory.path,
        directory.icon,
        config_json
    ])?;
    
    Ok(conn.last_insert_rowid() as u64)
}

/// Updates an existing directory by name.
/// 
/// # Arguments
/// * `name` - The name of the directory to update
/// * `directory` - The new directory definition
/// 
/// # Returns
/// Number of rows affected (1 on success, 0 if not found)
pub fn update_directory(name: &str, directory: &SystemFileDirectory) -> Result<usize> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    let config_json = directory.config.as_ref()
        .map(|c| serde_json::to_string(c).ok())
        .flatten();
    
    let sql = r#"
        UPDATE file_directories 
        SET provider = ?1, path = ?2, icon = ?3, config = ?4, updated_at = CURRENT_TIMESTAMP
        WHERE name = ?5
    "#;
    
    conn.execute(sql, params![
        directory.provider,
        directory.path,
        directory.icon,
        config_json,
        name
    ])
}

/// Deletes a directory by name.
/// 
/// # Arguments
/// * `name` - The name of the directory to delete
/// 
/// # Returns
/// Number of rows affected (1 on success, 0 if not found)
pub fn delete_directory(name: &str) -> Result<usize> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    conn.execute("DELETE FROM file_directories WHERE name = ?1", params![name])
}

// ============================================================================
// QUERY OPERATIONS
// ============================================================================

/// Retrieves a directory by name.
/// 
/// # Arguments
/// * `name` - The directory name to look up
/// 
/// # Returns
/// The directory if found, or QueryReturnedNoRows error if not
pub fn get_directory_by_name(name: &str) -> Result<SystemFileDirectory> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    
    conn.query_row(
        "SELECT name, provider, path, icon, config, source FROM file_directories WHERE name = ?1",
        params![name],
        row_to_directory
    )
}

/// Retrieves all directories from the repository.
/// 
/// # Returns
/// Vector of all registered directories
pub fn get_all_directories() -> Result<Vec<SystemFileDirectory>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, provider, path, icon, config, source FROM file_directories ORDER BY name"
    )?;
    
    let rows = stmt.query_map([], row_to_directory)?;
    rows.collect()
}

/// Retrieves directories by source (e.g., "config" or "runtime").
/// 
/// # Arguments
/// * `source` - The source filter
/// 
/// # Returns
/// Vector of directories matching the source
pub fn get_directories_by_source(source: &str) -> Result<Vec<SystemFileDirectory>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, provider, path, icon, config, source FROM file_directories WHERE source = ?1 ORDER BY name"
    )?;
    
    let rows = stmt.query_map(params![source], row_to_directory)?;
    rows.collect()
}

/// Retrieves directories by provider type.
/// 
/// # Arguments
/// * `provider` - The provider type filter (e.g., "local", "sftp")
/// 
/// # Returns
/// Vector of directories using the specified provider
pub fn get_directories_by_provider(provider: &str) -> Result<Vec<SystemFileDirectory>> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let mut stmt = conn.prepare(
        "SELECT name, provider, path, icon, config, source FROM file_directories WHERE provider = ?1 ORDER BY name"
    )?;
    
    let rows = stmt.query_map(params![provider], row_to_directory)?;
    rows.collect()
}

/// Counts total directories in the repository.
/// 
/// # Returns
/// Number of registered directories
pub fn count_directories() -> Result<u32> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    conn.query_row("SELECT COUNT(*) FROM file_directories", [], |row| row.get(0))
}

/// Checks if a directory exists by name.
/// 
/// # Arguments
/// * `name` - The directory name to check
/// 
/// # Returns
/// true if the directory exists, false otherwise
pub fn directory_exists(name: &str) -> Result<bool> {
    let conn = Connection::open(format!("{}.db", REGISTRY_DB))?;
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM file_directories WHERE name = ?1",
        params![name],
        |row| row.get(0)
    )?;
    Ok(count > 0)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Maps a database row to a SystemFileDirectory.
/// Column order: name, provider, path, icon, config, source
fn row_to_directory(row: &Row) -> Result<SystemFileDirectory> {
    let config_json: Option<String> = row.get(4)?;
    let config: Option<serde_json::Value> = config_json
        .and_then(|s| serde_json::from_str(&s).ok());
    
    let source: String = row.get(5)?;
    
    Ok(SystemFileDirectory {
        name: row.get(0)?,
        provider: row.get(1)?,
        path: row.get(2)?,
        icon: row.get(3)?,
        config,
        source: Some(source),
    })
}

#[cfg(test)]
#[path = "repository_files.test.rs"]
mod tests;
