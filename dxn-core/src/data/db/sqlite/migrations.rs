use rusqlite::{Connection, Result, params};
use rusqlite::ffi;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use chrono::Utc;

/// Migration system for SQLite databases
/// 
/// This module provides a safe, approval-based migration system that:
/// - Stores UP and DOWN migration scripts
/// - Creates database backups before applying migrations
/// - Requires explicit approval for destructive operations
/// - Tracks migration history in a migrations table
/// - Stores migration files in the file server

// ============================================================================
// TYPES
// ============================================================================

/// Represents a database migration with UP and DOWN scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Unique migration identifier (e.g., "001_add_email_column")
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// SQL script to apply the migration
    pub up: String,
    /// SQL script to rollback the migration
    pub down: String,
    /// Timestamp when migration was created
    pub created_at: String,
}

/// Migration status in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// Migration ID
    pub id: String,
    /// Whether migration has been applied
    pub applied: bool,
    /// Timestamp when applied
    pub applied_at: Option<String>,
}

/// Result of a migration operation
#[derive(Debug)]
pub enum MigrationResult {
    /// Migration applied successfully
    Applied,
    /// Migration requires approval (destructive operation detected)
    RequiresApproval { reason: String },
    /// Migration failed
    Failed { error: String },
}

// ============================================================================
// CONSTANTS
// ============================================================================

/// Path to store migration files (relative to dxn-files)
const MIGRATIONS_DIR: &str = "_files/migrations";

/// Path to store database backup files (relative to dxn-files)
const DB_BACKUP_DIR: &str = "db-backup";

/// Name of the migrations tracking table
const MIGRATIONS_TABLE: &str = "__dxn_migrations";

// ============================================================================
// MIGRATION FILE MANAGEMENT
// ============================================================================

/// Gets the full path to the migrations directory
fn get_migrations_path() -> PathBuf {
    PathBuf::from("../dxn-files").join(MIGRATIONS_DIR)
}

/// Gets the full path to the database backup directory
fn get_db_backup_path() -> PathBuf {
    PathBuf::from("../dxn-files").join(DB_BACKUP_DIR)
}

/// Saves a migration to a file in the migrations directory
/// 
/// # Arguments
/// * `migration` - Migration to save
/// 
/// # Returns
/// `Result<PathBuf>` with the path to the saved migration file
pub fn save_migration(migration: &Migration) -> io::Result<PathBuf> {
    let migrations_dir = get_migrations_path();
    
    // Create migrations directory if it doesn't exist
    fs::create_dir_all(&migrations_dir)?;

    // Create migration file path: migrations/001_add_email_column.json
    let file_path = migrations_dir.join(format!("{}.json", migration.id));

    // Serialize migration to JSON
    let json = serde_json::to_string_pretty(migration)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Write to file
    fs::write(&file_path, json)?;

    Ok(file_path)
}

/// Loads all migration files from the migrations directory
/// 
/// # Returns
/// Vector of migrations sorted by ID
pub fn load_migrations() -> io::Result<Vec<Migration>> {
    let migrations_dir = get_migrations_path();

    if !migrations_dir.exists() {
        return Ok(Vec::new());
    }

    let mut migrations = Vec::new();

    // Read all .json files in migrations directory
    for entry in fs::read_dir(&migrations_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let migration: Migration = serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            migrations.push(migration);
        }
    }

    // Sort by ID (assumes IDs are sortable, e.g., "001", "002")
    migrations.sort_by_key(|m| m.id.clone());

    Ok(migrations)
}

// ============================================================================
// DATABASE BACKUP
// ============================================================================

/// Creates a backup of the database before migration
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// 
/// # Returns
/// Path to the backup file
pub fn backup_database(db_name: &str) -> io::Result<PathBuf> {
    let source = format!("{}.db", db_name);
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}.db.backup", db_name, timestamp);
    
    // Get backup directory and ensure it exists
    let backup_dir = get_db_backup_path();
    fs::create_dir_all(&backup_dir)?;
    
    // Create full backup path
    let backup_path = backup_dir.join(&backup_name);

    // Copy database file
    fs::copy(&source, &backup_path)?;

    println!("[MIGRATION] Database backed up to: {}", backup_path.display());
    Ok(backup_path)
}

// ============================================================================
// MIGRATION TRACKING
// ============================================================================

/// Initializes the migrations tracking table
/// 
/// # Arguments
/// * `conn` - Database connection
pub fn init_migrations_table(conn: &Connection) -> Result<()> {
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id TEXT PRIMARY KEY NOT NULL,
            description TEXT NOT NULL,
            applied BOOLEAN NOT NULL DEFAULT 0,
            applied_at TEXT,
            created_at TEXT NOT NULL
        )",
        MIGRATIONS_TABLE
    );

    conn.execute(&sql, [])?;
    Ok(())
}

/// Gets the list of applied migrations from the database
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// 
/// # Returns
/// Vector of migration IDs that have been applied
pub fn get_applied_migrations(db_name: &str) -> Result<Vec<String>> {
    let conn = Connection::open(format!("{}.db", db_name))?;
    
    // Ensure migrations table exists
    init_migrations_table(&conn)?;

    let mut stmt = conn.prepare(&format!(
        "SELECT id FROM {} WHERE applied = 1 ORDER BY id",
        MIGRATIONS_TABLE
    ))?;

    let ids: Result<Vec<String>> = stmt
        .query_map([], |row| row.get(0))?
        .collect();

    ids
}

/// Records a migration as applied in the database
/// 
/// # Arguments
/// * `conn` - Database connection
/// * `migration` - Migration that was applied
fn record_migration_applied(conn: &Connection, migration: &Migration) -> Result<()> {
    let sql = format!(
        "INSERT OR REPLACE INTO {} (id, description, applied, applied_at, created_at)
         VALUES (?1, ?2, 1, ?3, ?4)",
        MIGRATIONS_TABLE
    );

    let applied_at = Utc::now().to_rfc3339();
    conn.execute(
        &sql,
        params![migration.id, migration.description, applied_at, migration.created_at],
    )?;

    Ok(())
}

/// Records a migration as rolled back in the database
/// 
/// # Arguments
/// * `conn` - Database connection
/// * `migration_id` - ID of migration to rollback
fn record_migration_rolled_back(conn: &Connection, migration_id: &str) -> Result<()> {
    let sql = format!(
        "UPDATE {} SET applied = 0, applied_at = NULL WHERE id = ?1",
        MIGRATIONS_TABLE
    );

    conn.execute(&sql, params![migration_id])?;
    Ok(())
}

// ============================================================================
// SCHEMA DIFF DETECTION
// ============================================================================

/// Detects if a migration contains potentially destructive operations
/// 
/// # Arguments
/// * `sql` - SQL script to analyze
/// 
/// # Returns
/// `true` if destructive operations are detected
fn is_destructive(sql: &str) -> bool {
    let sql_upper = sql.to_uppercase();
    
    // Keywords that indicate destructive operations
    let destructive_keywords = [
        "DROP TABLE",
        "DROP COLUMN",
        "DELETE FROM",
        "TRUNCATE",
        "ALTER TABLE DROP",
    ];

    destructive_keywords.iter().any(|keyword| sql_upper.contains(keyword))
}

/// Analyzes a migration and determines if approval is required
/// 
/// # Arguments
/// * `migration` - Migration to analyze
/// 
/// # Returns
/// `Option<String>` with reason if approval is required
pub fn requires_approval(migration: &Migration) -> Option<String> {
    if is_destructive(&migration.up) {
        return Some(format!(
            "Migration '{}' contains destructive operations (DROP, DELETE, TRUNCATE)",
            migration.id
        ));
    }

    None
}

// ============================================================================
// MIGRATION EXECUTION
// ============================================================================

/// Applies a single migration to the database
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `migration` - Migration to apply
/// * `force` - If true, skip approval check (use with caution!)
/// 
/// # Returns
/// `MigrationResult` indicating success or requiring approval
pub fn apply_migration(
    db_name: &str,
    migration: &Migration,
    force: bool,
) -> Result<MigrationResult> {
    // Check if migration requires approval
    if !force {
        if let Some(reason) = requires_approval(migration) {
            return Ok(MigrationResult::RequiresApproval { reason });
        }
    }

    // Create backup before applying
    backup_database(db_name)
        .map_err(|e| rusqlite::Error::SqliteFailure(
            ffi::Error::new(ffi::SQLITE_IOERR),
            Some(format!("Failed to backup database: {}", e))
        ))?;

    let conn = Connection::open(format!("{}.db", db_name))?;
    
    // Initialize migrations table if needed
    init_migrations_table(&conn)?;

    // Check if already applied
    let applied = get_applied_migrations(db_name)?;
    if applied.contains(&migration.id) {
        println!("[MIGRATION] Migration '{}' already applied, skipping", migration.id);
        return Ok(MigrationResult::Applied);
    }

    // Execute migration in a transaction
    let tx = conn.unchecked_transaction()?;
    
    match tx.execute_batch(&migration.up) {
        Ok(_) => {
            // Record migration as applied
            record_migration_applied(&tx, migration)?;
            tx.commit()?;
            
            println!("[MIGRATION] Successfully applied migration: {}", migration.id);
            Ok(MigrationResult::Applied)
        }
        Err(e) => {
            tx.rollback()?;
            Ok(MigrationResult::Failed {
                error: format!("Failed to apply migration '{}': {}", migration.id, e),
            })
        }
    }
}

/// Rolls back a single migration
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `migration` - Migration to rollback
/// * `force` - If true, skip approval check
/// 
/// # Returns
/// `MigrationResult` indicating success or requiring approval
pub fn rollback_migration(
    db_name: &str,
    migration: &Migration,
    force: bool,
) -> Result<MigrationResult> {
    // Always require approval for rollbacks (they're inherently destructive)
    if !force {
        return Ok(MigrationResult::RequiresApproval {
            reason: format!(
                "Rollback of migration '{}' requires explicit approval",
                migration.id
            ),
        });
    }

    // Create backup before rollback
    backup_database(db_name)
        .map_err(|e| rusqlite::Error::SqliteFailure(
            ffi::Error::new(ffi::SQLITE_IOERR),
            Some(format!("Failed to backup database: {}", e))
        ))?;

    let conn = Connection::open(format!("{}.db", db_name))?;
    let tx = conn.unchecked_transaction()?;

    match tx.execute_batch(&migration.down) {
        Ok(_) => {
            record_migration_rolled_back(&tx, &migration.id)?;
            tx.commit()?;
            
            println!("[MIGRATION] Successfully rolled back migration: {}", migration.id);
            Ok(MigrationResult::Applied)
        }
        Err(e) => {
            tx.rollback()?;
            Ok(MigrationResult::Failed {
                error: format!("Failed to rollback migration '{}': {}", migration.id, e),
            })
        }
    }
}

/// Applies all pending migrations
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `force` - If true, skip approval checks
/// 
/// # Returns
/// Vector of results for each migration
pub fn apply_all_pending(db_name: &str, force: bool) -> Result<Vec<(String, MigrationResult)>> {
    let migrations = load_migrations()
        .map_err(|e| rusqlite::Error::SqliteFailure(
            ffi::Error::new(ffi::SQLITE_IOERR),
            Some(format!("Failed to load migrations: {}", e))
        ))?;
    let applied = get_applied_migrations(db_name)?;

    let pending: Vec<&Migration> = migrations
        .iter()
        .filter(|m| !applied.contains(&m.id))
        .collect();

    let mut results = Vec::new();

    for migration in pending {
        let result = apply_migration(db_name, migration, force)?;
        results.push((migration.id.clone(), result));
    }

    Ok(results)
}

// ============================================================================
// MIGRATION CREATION HELPERS
// ============================================================================

/// Creates a new migration with UP and DOWN scripts
/// 
/// # Arguments
/// * `id` - Unique identifier (e.g., "001_add_email_column")
/// * `description` - Human-readable description
/// * `up` - SQL to apply the migration
/// * `down` - SQL to rollback the migration
/// 
/// # Returns
/// `Migration` struct ready to be saved
pub fn create_migration(id: String, description: String, up: String, down: String) -> Migration {
    Migration {
        id,
        description,
        up,
        down,
        created_at: Utc::now().to_rfc3339(),
    }
}
