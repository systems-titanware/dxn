use super::*;

/// Cleanup function - not used by default to avoid race conditions in parallel tests.
/// Tests now use idempotent init + delete of specific test data instead.
#[allow(dead_code)]
fn cleanup_test_db() {
    let _ = std::fs::remove_file(format!("{}.db", REGISTRY_DB));
}

#[test]
fn test_init_and_insert_schema() {
    // Initialize table (idempotent - uses CREATE TABLE IF NOT EXISTS)
    init_schema_table().expect("Failed to init schema table");
    
    // Clean up any existing test data from previous runs (hard delete for tests)
    let _ = hard_delete_schema("test_schema");
    
    // Create a test schema
    let schema = SystemDataModel {
        name: "test_schema".to_string(),
        version: 1,
        db: "public".to_string(),
        public: false,
        source: None,
        icon: Some("📦".to_string()),
        status: SchemaStatus::Active,
        deleted_at: None,
        fields: vec![
            SystemDataModelField {
                name: "id".to_string(),
                datatype: "INTEGER".to_string(),
                value: String::new(),
                primary: Some(true),
                secondary: None,
            },
            SystemDataModelField {
                name: "name".to_string(),
                datatype: "TEXT".to_string(),
                value: String::new(),
                primary: None,
                secondary: None,
            },
        ],
    };
    
    // Insert schema
    let id = insert_runtime_schema(&schema).expect("Failed to insert schema");
    assert!(id > 0);
    
    // Verify it exists
    assert!(schema_exists("test_schema").unwrap());
    
    // Retrieve and verify
    let retrieved = get_schema_by_name("test_schema").expect("Failed to get schema");
    assert_eq!(retrieved.name, "test_schema");
    assert_eq!(retrieved.version, 1);
    assert_eq!(retrieved.fields.len(), 2);
    assert_eq!(retrieved.source, Some("runtime".to_string()));
    assert_eq!(retrieved.status, SchemaStatus::Active);
    
    // Note: cleanup at end removed to avoid race conditions with parallel tests
}

#[test]
fn test_upsert_config_schema() {
    // Initialize table (idempotent - uses CREATE TABLE IF NOT EXISTS)
    init_schema_table().expect("Failed to init schema table");
    
    // Clean up any existing test data from previous runs (hard delete for tests)
    let _ = hard_delete_schema("config_schema");
    
    let schema_v1 = SystemDataModel {
        name: "config_schema".to_string(),
        version: 1,
        db: "public".to_string(),
        public: false,
        source: None,
        icon: None,
        status: SchemaStatus::Active,
        deleted_at: None,
        fields: vec![],
    };
    
    // First upsert
    upsert_schema(&schema_v1, "config").expect("Failed to upsert v1");
    
    let schema_v2 = SystemDataModel {
        name: "config_schema".to_string(),
        version: 2,
        db: "public".to_string(),
        public: true,
        source: None,
        icon: Some("⚙️".to_string()),
        status: SchemaStatus::Active,
        deleted_at: None,
        fields: vec![],
    };
    
    // Second upsert should update
    upsert_schema(&schema_v2, "config").expect("Failed to upsert v2");
    
    // Verify version updated
    let retrieved = get_schema_by_name("config_schema").unwrap();
    assert_eq!(retrieved.version, 2);
    assert!(retrieved.public);
    
    // Note: cleanup at end removed to avoid race conditions with parallel tests
}

#[test]
fn test_soft_delete_and_restore() {
    // Initialize table
    init_schema_table().expect("Failed to init schema table");
    
    // Clean up from previous runs
    let _ = hard_delete_schema("soft_delete_test");
    
    // Create a schema
    let schema = SystemDataModel {
        name: "soft_delete_test".to_string(),
        version: 1,
        db: "public".to_string(),
        public: false,
        source: None,
        icon: None,
        status: SchemaStatus::Active,
        deleted_at: None,
        fields: vec![],
    };
    
    insert_runtime_schema(&schema).expect("Failed to insert schema");
    
    // Verify it exists
    assert!(schema_exists("soft_delete_test").unwrap());
    
    // Soft delete
    let deleted = soft_delete_schema("soft_delete_test").expect("Failed to soft delete");
    assert_eq!(deleted, 1);
    
    // Should no longer appear in active queries
    assert!(!schema_exists("soft_delete_test").unwrap());
    
    // But should be marked as deleted
    assert!(is_schema_deleted("soft_delete_test").unwrap());
    
    // Should still be retrievable with include_deleted
    let deleted_schema = get_schema_by_name_include_deleted("soft_delete_test").unwrap();
    assert_eq!(deleted_schema.status, SchemaStatus::Deleted);
    assert!(deleted_schema.deleted_at.is_some());
    
    // Restore
    let restored = restore_schema("soft_delete_test").expect("Failed to restore");
    assert_eq!(restored, 1);
    
    // Should be active again
    assert!(schema_exists("soft_delete_test").unwrap());
    assert!(!is_schema_deleted("soft_delete_test").unwrap());
    
    // Verify status
    let active_schema = get_schema_by_name("soft_delete_test").unwrap();
    assert_eq!(active_schema.status, SchemaStatus::Active);
    assert!(active_schema.deleted_at.is_none());
}
