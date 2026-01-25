use rusqlite::{params, Connection, Result, Row};
use crate::data::db::models::DbColumn;

/// SQLite repository for database operations
/// 
/// This module provides CRUD operations for SQLite databases with proper
/// error handling, parameterized queries, and connection management.

// ============================================================================
// COLUMN HELPERS
// ============================================================================

/// Creates a primary key column definition
/// 
/// # Arguments
/// * `name` - Column name
/// * `col_type` - SQLite column type (e.g., "INTEGER", "TEXT", "BLOB")
/// 
/// # Returns
/// A `DbColumn` configured as a primary key
pub fn create_col_primary(name: String, col_type: String) -> DbColumn {
    DbColumn {
        name,
        col_type,
        value: None,
        primary_key: Some(true),
        secondary_key: None,
        nullable: false,
        unique: None,
        default: None,
        autoincrement: None,
        check: None,
    }
}

/// Creates a standard column definition
/// 
/// # Arguments
/// * `name` - Column name
/// * `col_type` - SQLite column type
/// * `nullable` - Whether the column allows NULL values
/// 
/// # Returns
/// A `DbColumn` with the specified configuration
pub fn create_col(name: String, col_type: String, nullable: bool) -> DbColumn {
    DbColumn {
        name,
        col_type,
        value: None,
        primary_key: None,
        secondary_key: None,
        nullable,
        unique: None,
        default: None,
        autoincrement: None,
        check: None,
    }
}

// ============================================================================
// TABLE OPERATIONS
// ============================================================================

/// Creates a table dynamically based on column definitions
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `table_name` - Name of the table to create
/// * `fields` - Vector of column definitions
/// 
/// # Returns
/// `Result<()>` indicating success or failure
/// 
/// # Errors
/// Returns `rusqlite::Error` if table creation fails
/// 
/// # Note
/// Uses `CREATE TABLE IF NOT EXISTS` to avoid errors on re-runs
pub fn create_dynamic_table(
    db_name: String,
    table_name: String,
    fields: Vec<DbColumn>,
) -> Result<()> {
    let conn = Connection::open(format!("{}.db", db_name))?;

    let mut columns = String::new();
    for field in fields.iter() {
        let mut column_def = format!("{} {}", field.name, field.col_type);
        
        // Handle PRIMARY KEY
        if let Some(true) = field.primary_key {
            column_def.push_str(" PRIMARY KEY");
            
            // AUTOINCREMENT for INTEGER PRIMARY KEY
            if let Some(true) = field.autoincrement {
                if field.col_type.to_uppercase().contains("INTEGER") {
                    column_def.push_str(" AUTOINCREMENT");
                }
            }
            
            // Primary keys are NOT NULL by default in SQLite
            column_def.push_str(" NOT NULL");
        } else {
            // Handle UNIQUE constraint (only if not primary key)
            if let Some(true) = field.unique {
                column_def.push_str(" UNIQUE");
            }
            
            // Handle NOT NULL constraint
            if !field.nullable {
                column_def.push_str(" NOT NULL");
            }
        }
        
        // Handle DEFAULT value
        if let Some(ref default_val) = field.default {
            // Don't quote if it's a SQL function or expression
            if default_val.starts_with("CURRENT_") || 
               default_val.parse::<i64>().is_ok() || 
               default_val.parse::<f64>().is_ok() {
                column_def.push_str(&format!(" DEFAULT {}", default_val));
            } else {
                // Quote string defaults
                column_def.push_str(&format!(" DEFAULT '{}'", default_val));
            }
        }
        
        // Handle CHECK constraint
        if let Some(ref check_expr) = field.check {
            column_def.push_str(&format!(" CHECK ({})", check_expr));
        }
        
        columns.push_str(&format!("{},\n", column_def));
    }

    // Remove trailing comma and newline
    if columns.ends_with(",\n") {
        columns.pop(); // Remove \n
        columns.pop(); // Remove ,
    }

    let sql = format!("CREATE TABLE IF NOT EXISTS {} (\n{})", table_name, columns);
    conn.execute(&sql, [])?;
    Ok(())
}

// ============================================================================
// CRUD OPERATIONS
// ============================================================================

/// Inserts a new record into the specified table
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `table_name` - Table name
/// * `keys` - Vector of column names
/// * `values` - Vector of values to insert (as `serde_json::Value`)
/// 
/// # Returns
/// Number of rows affected (should be 1 on success)
/// 
/// # Errors
/// Returns `rusqlite::Error` if insertion fails
/// 
/// # Security
/// Uses parameterized queries to prevent SQL injection
pub fn insert(
    db_name: String,
    table_name: String,
    keys: Vec<String>,
    values: Vec<serde_json::Value>,
) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open(format!("{}.db", db_name))?;

    // Convert serde_json::Value to rusqlite parameters
    let params_iter = serde_rusqlite::to_params(&values)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    // Generate placeholders: ?1, ?2, ?3, ...
    let placeholders: Vec<String> = (1..=values.len())
        .map(|i| format!("?{}", i))
        .collect();

    // Build parameterized query
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        keys.join(","),
        placeholders.join(",")
    );

    conn.execute(&query, params_iter)
}

/// Updates an existing record in the specified table
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `table_name` - Table name
/// * `id` - Record ID to update (as string for flexibility)
/// * `keys` - Vector of column names to update
/// * `values` - Vector of new values
/// 
/// # Returns
/// Number of rows affected
/// 
/// # Errors
/// Returns `rusqlite::Error` if update fails
/// 
/// # Security
/// Uses parameterized queries to prevent SQL injection
pub fn update(
    db_name: String,
    table_name: String,
    id: String,
    keys: Vec<String>,
    values: Vec<serde_json::Value>,
) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open(format!("{}.db", db_name))?;

    // Build SET clause: key1 = ?1, key2 = ?2, ...
    let set_clauses: Vec<String> = keys
        .iter()
        .enumerate()
        .map(|(i, key)| format!("{} = ?{}", key, i + 1))
        .collect();

    // Combine values with ID parameter
    let mut all_params: Vec<serde_json::Value> = values;
    all_params.push(serde_json::Value::String(id));

    // Build parameterized query with ID as parameter
    let query = format!(
        "UPDATE {} SET {} WHERE id = ?{}",
        table_name,
        set_clauses.join(", "),
        all_params.len()
    );

    // Convert serde_json::Value to rusqlite parameters
    let params_iter = serde_rusqlite::to_params(&all_params)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    conn.execute(&query, params_iter)
}

/// Deletes a record from the specified table
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `table_name` - Table name
/// * `id` - Record ID to delete
/// 
/// # Returns
/// Number of rows affected (should be 1 on success)
/// 
/// # Errors
/// Returns `rusqlite::Error` if deletion fails
/// 
/// # Security
/// Uses parameterized queries to prevent SQL injection
pub fn delete(db_name: String, table_name: String, id: u32) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open(format!("{}.db", db_name))?;
    let query = format!("DELETE FROM {} WHERE id = ?1", table_name);

    conn.execute(&query, params![id])
}

// ============================================================================
// QUERY OPERATIONS
// ============================================================================

/// Retrieves a single record by ID
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `table_name` - Table name
/// * `id` - Record ID
/// * `mapper` - Closure to map database row to target type
/// 
/// # Returns
/// Mapped result of type `T`
/// 
/// # Errors
/// Returns `rusqlite::Error` if query fails or record not found
pub fn get<F, T>(db_name: String, table_name: String, id: u32, mapper: F) -> Result<T>
where
    F: FnMut(&Row) -> Result<T>,
    T: Sized,
{
    let conn = Connection::open(format!("{}.db", db_name))?;
    let query = format!("SELECT * FROM {} WHERE id = ?1", table_name);

    conn.query_row(&query, [id], mapper)
}

/// Lists all records from the specified table
/// 
/// # Arguments
/// * `db_name` - Database name (without .db extension)
/// * `table_name` - Table name
/// * `page_size` - Number of records per page (currently unused, reserved for pagination)
/// * `page` - Page number (currently unused, reserved for pagination)
/// * `query` - Additional query string (currently unused, reserved for filtering)
/// * `mapper` - Closure to map database rows to target type
/// 
/// # Returns
/// Vector of mapped results
/// 
/// # Errors
/// Returns `rusqlite::Error` if query fails
/// 
/// # Note
/// Pagination and filtering parameters are reserved for future implementation
pub fn list<F, T>(
    db_name: String,
    table_name: String,
    _page_size: u32,
    _page: u32,
    _query: String,
    mapper: F,
) -> Result<Vec<T>>
where
    F: FnMut(&Row) -> Result<T>,
    T: Sized,
{
    let conn = Connection::open(format!("{}.db", db_name))?;
    let query = format!("SELECT * FROM {}", table_name);
    let mut stmt = conn.prepare(&query)?;

    let rows = stmt.query_map([], mapper)?;
    rows.collect()
}

#[cfg(test)]
#[path = "repository.test.rs"]
mod tests;
