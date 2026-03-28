//! Tests for File Directory Repository

use super::*;
use uuid::Uuid;

fn unique_name(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::now_v7())
}

/// Ensures test-created directory rows are deleted even if a test panics.
struct CleanupGuard {
    names: Vec<String>,
}

impl CleanupGuard {
    fn new() -> Self {
        Self { names: Vec::new() }
    }

    fn track(&mut self, name: String) -> String {
        self.names.push(name.clone());
        name
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        for name in &self.names {
            let _ = delete_directory(name);
        }
    }
}

#[test]
fn test_init_files_table() {
    let result = init_files_table();
    assert!(result.is_ok(), "Should create file_directories table");
    
    // Calling again should not fail (idempotent)
    let result2 = init_files_table();
    assert!(result2.is_ok(), "Should be idempotent");
}

#[test]
fn test_insert_and_get_directory() {
    init_files_table().unwrap();
    let mut cleanup = CleanupGuard::new();
    
    let name = cleanup.track(unique_name("test-dir"));
    let directory = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/test/path".to_string(),
        icon: Some("📁".to_string()),
        source: None,
        config: None,
    };
    
    let result = insert_runtime_directory(&directory);
    assert!(result.is_ok(), "Should insert directory");
    
    let retrieved = get_directory_by_name(&name);
    assert!(retrieved.is_ok(), "Should retrieve directory");
    
    let dir = retrieved.unwrap();
    assert_eq!(dir.name, name);
    assert_eq!(dir.provider, "local");
    assert_eq!(dir.path, "/test/path");
    assert_eq!(dir.source, Some("runtime".to_string()));
}

#[test]
fn test_directory_exists() {
    init_files_table().unwrap();
    let mut cleanup = CleanupGuard::new();
    
    let name = cleanup.track(unique_name("exists-test"));
    
    // Should not exist initially
    assert!(!directory_exists(&name).unwrap());
    
    // Create directory
    let directory = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/exists/test".to_string(),
        icon: None,
        source: None,
        config: None,
    };
    insert_runtime_directory(&directory).unwrap();
    
    // Should exist now
    assert!(directory_exists(&name).unwrap());
}

#[test]
fn test_update_directory() {
    init_files_table().unwrap();
    let mut cleanup = CleanupGuard::new();
    
    let name = cleanup.track(unique_name("update-test"));
    let directory = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/original/path".to_string(),
        icon: Some("📁".to_string()),
        source: None,
        config: None,
    };
    
    insert_runtime_directory(&directory).unwrap();
    
    // Update the directory
    let updated = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/updated/path".to_string(),
        icon: Some("📂".to_string()),
        source: None,
        config: None,
    };
    
    let result = update_directory(&name, &updated);
    assert!(result.is_ok(), "Should update directory");
    
    let retrieved = get_directory_by_name(&name).unwrap();
    assert_eq!(retrieved.path, "/updated/path");
    assert_eq!(retrieved.icon, Some("📂".to_string()));
}

#[test]
fn test_delete_directory() {
    init_files_table().unwrap();
    
    let name = unique_name("delete-test");
    let directory = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/delete/test".to_string(),
        icon: None,
        source: None,
        config: None,
    };
    
    insert_runtime_directory(&directory).unwrap();
    assert!(directory_exists(&name).unwrap());
    
    let result = delete_directory(&name);
    assert!(result.is_ok(), "Should delete directory");
    
    assert!(!directory_exists(&name).unwrap());
}

#[test]
fn test_get_all_directories() {
    init_files_table().unwrap();
    let mut cleanup = CleanupGuard::new();
    
    let name1 = cleanup.track(unique_name("all-test-1"));
    let name2 = cleanup.track(unique_name("all-test-2"));
    
    let dir1 = SystemFileDirectory {
        name: name1.clone(),
        provider: "local".to_string(),
        path: "/all/test/1".to_string(),
        icon: None,
        source: None,
        config: None,
    };
    
    let dir2 = SystemFileDirectory {
        name: name2.clone(),
        provider: "local".to_string(),
        path: "/all/test/2".to_string(),
        icon: None,
        source: None,
        config: None,
    };
    
    insert_runtime_directory(&dir1).unwrap();
    insert_runtime_directory(&dir2).unwrap();
    
    let all = get_all_directories().unwrap();
    
    // Should contain at least our two directories
    let names: Vec<_> = all.iter().map(|d| d.name.clone()).collect();
    assert!(names.contains(&name1));
    assert!(names.contains(&name2));
}

#[test]
fn test_upsert_directory_config_source() {
    init_files_table().unwrap();
    let mut cleanup = CleanupGuard::new();
    
    let name = cleanup.track(unique_name("upsert-config"));
    let directory = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/upsert/config".to_string(),
        icon: Some("🔧".to_string()),
        source: None,
        config: None,
    };
    
    // Upsert as config source
    let result = upsert_directory(&directory, "config");
    assert!(result.is_ok(), "Should upsert directory");
    
    let retrieved = get_directory_by_name(&name).unwrap();
    assert_eq!(retrieved.source, Some("config".to_string()));
    
    // Upsert again with updated path
    let updated = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/upsert/config/updated".to_string(),
        icon: Some("⚙️".to_string()),
        source: None,
        config: None,
    };
    
    upsert_directory(&updated, "config").unwrap();
    
    let retrieved2 = get_directory_by_name(&name).unwrap();
    assert_eq!(retrieved2.path, "/upsert/config/updated");
    assert_eq!(retrieved2.icon, Some("⚙️".to_string()));
}

#[test]
fn test_count_directories() {
    init_files_table().unwrap();
    let mut cleanup = CleanupGuard::new();
    
    let initial_count = count_directories().unwrap();
    
    let name = cleanup.track(unique_name("count-test"));
    let directory = SystemFileDirectory {
        name: name.clone(),
        provider: "local".to_string(),
        path: "/count/test".to_string(),
        icon: None,
        source: None,
        config: None,
    };
    
    insert_runtime_directory(&directory).unwrap();
    
    let new_count = count_directories().unwrap();
    // Count must increase by at least 1 (may be more if other tests run in parallel on shared system.db)
    assert!(
        new_count >= initial_count + 1,
        "count should increase after insert: got {} (was {})",
        new_count,
        initial_count
    );
    assert!(get_directory_by_name(&name).is_ok(), "inserted directory should be retrievable");
}
