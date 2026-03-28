use super::*;
use std::fs;
use std::io;
use tempfile::TempDir;

/// Creates a temporary test directory that auto-cleans on drop
fn create_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Helper to create a test file path within a temp directory
fn test_file_path(temp_dir: &TempDir, name: &str) -> PathBuf {
    temp_dir.path().join(name)
}

#[test]
fn test_read_file_not_found() {
    let result = read_file("nonexistent_file_that_does_not_exist.txt");
    assert!(result.is_err());
}

#[test]
fn test_add_file_content() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "write_test.txt");
    let test_content = "Test content for writing";
    
    // Write content directly using fs (testing sync guarantee)
    let mut file = File::create(&test_path).unwrap();
    file.write_all(test_content.as_bytes()).unwrap();
    file.sync_all().unwrap();
    
    // Verify file exists immediately after sync
    assert!(test_path.exists(), "File should exist after write with sync_all");
    
    // Verify content
    let content = fs::read_to_string(&test_path).unwrap();
    assert_eq!(content, test_content);
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_add_file_content_overwrite() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "overwrite_test.txt");
    let initial_content = "Initial content";
    let new_content = "New content";
    
    // Write initial content
    {
        let mut file = File::create(&test_path).unwrap();
        file.write_all(initial_content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
    
    // Verify file exists and initial content was written
    assert!(test_path.exists(), "File should exist after first write");
    let initial_read = fs::read_to_string(&test_path).unwrap();
    assert_eq!(initial_read, initial_content);
    
    // Overwrite with new content
    {
        let mut file = File::create(&test_path).unwrap();
        file.write_all(new_content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
    
    // Verify file still exists and has new content
    assert!(test_path.exists(), "File should exist after second write");
    let content = fs::read_to_string(&test_path).unwrap();
    assert_eq!(content, new_content);
    assert_ne!(content, initial_content);
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_add_content_append() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "append_test.txt");
    let initial_content = "Initial";
    let appended_content = "Appended";
    
    // Create file with initial content
    {
        let mut file = File::create(&test_path).unwrap();
        file.write_all(initial_content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
    
    // Append content
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&test_path)
            .unwrap();
        file.write_all(appended_content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
    
    // Verify both contents are present
    let content = fs::read_to_string(&test_path).unwrap();
    assert_eq!(content, format!("{}{}", initial_content, appended_content));
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_add_content_creates_file() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "create_append_test.txt");
    let content = "New file content";
    
    // Create file using OpenOptions (simulating add_content behavior)
    {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&test_path)
            .unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
    
    // Verify file was created with content
    let file_content = fs::read_to_string(&test_path).unwrap();
    assert_eq!(file_content, content);
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_add_file() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "touch_test.txt");
    
    // Create empty file (like touch)
    {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&test_path)
            .unwrap();
        file.sync_all().unwrap();
    }
    
    // Verify file exists
    assert!(test_path.exists());
    
    // Verify file is empty
    let content = fs::read_to_string(&test_path).unwrap();
    assert_eq!(content, "");
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_add_file_existing() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "touch_existing_test.txt");
    let initial_content = "Some content";
    
    // Create file with content
    {
        let mut file = File::create(&test_path).unwrap();
        file.write_all(initial_content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
    
    // Touch existing file (open with create, should not error)
    {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&test_path)
            .unwrap();
        file.sync_all().unwrap();
    }
    
    // Verify file still exists
    assert!(test_path.exists());
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_add_dir() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "new_directory");
    
    // Create directory
    fs::create_dir_all(&test_path).unwrap();
    
    // Verify directory exists
    assert!(test_path.exists());
    assert!(test_path.is_dir());
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_add_dir_nested() {
    let temp_dir = create_test_dir();
    let test_path = test_file_path(&temp_dir, "nested/deep/directory");
    
    // Create nested directory
    fs::create_dir_all(&test_path).unwrap();
    
    // Verify nested directory exists
    assert!(test_path.exists());
    assert!(test_path.is_dir());
    
    // temp_dir auto-cleans on drop
}

#[test]
fn test_get_full_path() {
    let path = "test/file.txt";
    let full_path = get_full_path(path);
    
    assert!(full_path.to_string_lossy().contains("dxn-files"));
    assert!(full_path.to_string_lossy().contains("_files"));
    assert!(full_path.to_string_lossy().contains("test/file.txt"));
}

// ============================================================================
// Integration tests using the actual file manager functions
// ============================================================================

#[test]
fn test_file_manager_add_file_content_integration() {
    // This test uses the actual file manager with ROOT_FILE_PATH
    // It tests that the sync guarantee works end-to-end
    
    let test_path = "test_integration/sync_test.txt";
    let test_content = "Integration test content";
    
    // Write using the file manager
    add_file_content(test_content, test_path).unwrap();
    
    // Immediately verify (sync_all guarantees this will work)
    let full_path = get_full_path(test_path);
    assert!(full_path.exists(), "File should exist immediately after add_file_content");
    
    let content = read_file(test_path).unwrap();
    assert_eq!(content, test_content);
    
    // Cleanup
    let _ = fs::remove_file(full_path);
    let _ = fs::remove_dir(get_full_path("test_integration"));
}

#[test]
fn test_file_manager_add_content_integration() {
    // Test append functionality with actual file manager
    
    let test_path = "test_integration/append_test.txt";
    let initial = "Hello";
    let appended = " World";
    
    // Write initial content
    add_file_content(initial, test_path).unwrap();
    
    // Append more content
    add_content(appended, test_path).unwrap();
    
    // Verify combined content
    let content = read_file(test_path).unwrap();
    assert_eq!(content, "Hello World");
    
    // Cleanup
    let _ = fs::remove_file(get_full_path(test_path));
    let _ = fs::remove_dir(get_full_path("test_integration"));
}
