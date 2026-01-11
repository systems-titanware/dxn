use super::*;
use rusqlite::Connection;

#[test]
fn test_remove_last_char() {
    assert_eq!(remove_last_char("hello"), "hell");
    assert_eq!(remove_last_char("test"), "tes");
    assert_eq!(remove_last_char("a"), "");
    assert_eq!(remove_last_char(""), "");
}

#[test]
fn test_get_object_from_path() {
    assert_eq!(get_object_from_path("/api/data/profile/123"), "profile");
    assert_eq!(get_object_from_path("/api/data/user/456"), "user");
    assert_eq!(get_object_from_path("/api/data/test"), "test");
}

#[test]
fn test_row_to_json_value() {
    // Create an in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    
    // Create a test table
    conn.execute(
        "CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            name TEXT,
            age INTEGER,
            active INTEGER,
            score REAL
        )",
        [],
    ).unwrap();

    // Insert test data
    conn.execute(
        "INSERT INTO test_table (name, age, active, score) VALUES (?, ?, ?, ?)",
        rusqlite::params!["John Doe", 30, 1, 95.5],
    ).unwrap();

    // Query and get the row
    let mut stmt = conn.prepare("SELECT id, name, age, active, score FROM test_table").unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();

    // Convert row to JSON value
    let json_value = row_to_json_value(row).unwrap();

    // Verify the JSON structure
    assert!(json_value.is_object());
    let obj = json_value.as_object().unwrap();
    
    assert_eq!(obj.get("id").unwrap().as_i64().unwrap(), 1);
    assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "John Doe");
    assert_eq!(obj.get("age").unwrap().as_i64().unwrap(), 30);
    assert_eq!(obj.get("active").unwrap().as_i64().unwrap(), 1);
    assert_eq!(obj.get("score").unwrap().as_f64().unwrap(), 95.5);
}

#[test]
fn test_row_to_json_value_with_null() {
    // Create an in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    
    // Create a test table with nullable field
    conn.execute(
        "CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            name TEXT,
            optional_field TEXT
        )",
        [],
    ).unwrap();

    // Insert test data with NULL
    conn.execute(
        "INSERT INTO test_table (name, optional_field) VALUES (?, ?)",
        rusqlite::params!["Test", None::<String>],
    ).unwrap();

    // Query and get the row
    let mut stmt = conn.prepare("SELECT id, name, optional_field FROM test_table").unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();

    // Convert row to JSON value
    let json_value = row_to_json_value(row).unwrap();

    // Verify NULL is handled correctly
    let obj = json_value.as_object().unwrap();
    assert_eq!(obj.get("optional_field").unwrap(), &serde_json::Value::Null);
}

#[test]
fn test_row_to_json_value_with_blob() {
    // Create an in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    
    // Create a test table with BLOB field
    conn.execute(
        "CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            data BLOB
        )",
        [],
    ).unwrap();

    // Insert test data with BLOB
    let blob_data = b"binary data";
    conn.execute(
        "INSERT INTO test_table (data) VALUES (?)",
        rusqlite::params![blob_data],
    ).unwrap();

    // Query and get the row
    let mut stmt = conn.prepare("SELECT id, data FROM test_table").unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();

    // Convert row to JSON value
    let json_value = row_to_json_value(row).unwrap();

    // Verify BLOB is converted to string
    let obj = json_value.as_object().unwrap();
    let data_str = obj.get("data").unwrap().as_str().unwrap();
    assert_eq!(data_str, "binary data");
}

#[test]
fn test_row_to_json_value_empty_table() {
    // Create an in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    
    // Create a test table
    conn.execute(
        "CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            name TEXT
        )",
        [],
    ).unwrap();

    // Query empty table
    let mut stmt = conn.prepare("SELECT id, name FROM test_table").unwrap();
    let mut rows = stmt.query([]).unwrap();
    
    // Should return no rows
    assert!(rows.next().unwrap().is_none());
}

