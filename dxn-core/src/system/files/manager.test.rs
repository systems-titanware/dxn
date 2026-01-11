use super::*;
use std::fs;
use std::path::Path;

// Helper function to create a temporary test directory
fn setup_test_env() -> io::Result<()> {
    let test_dir = Path::new("../dxn_public/_files/test");
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;
    Ok(())
}

// Helper function to cleanup test directory
fn cleanup_test_env() -> io::Result<()> {
    let test_dir = Path::new("../dxn_public/_files/test");
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
    }
    Ok(())
}

#[test]
fn test_read_file() {
    setup_test_env().unwrap();
    
    // Create a test file
    let test_path = "test/read_test.txt";
    let test_content = "Hello, World!";
    add_file_content(test_content, test_path).unwrap();
    
    // Read the file
    let content = read_file(test_path).unwrap();
    assert_eq!(content, test_content);
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_read_file_not_found() {
    let result = read_file("test/nonexistent.txt");
    assert!(result.is_err());
}

#[test]
fn test_add_file_content() {
    setup_test_env().unwrap();
    
    let test_path = "test/write_test.txt";
    let test_content = "Test content for writing";
    
    // Write content
    add_file_content(test_content, test_path).unwrap();
    
    // Verify content was written
    let content = read_file(test_path).unwrap();
    assert_eq!(content, test_content);
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_add_file_content_overwrite() {
    setup_test_env().unwrap();
    
    let test_path = "test/overwrite_test.txt";
    let initial_content = "Initial content";
    let new_content = "New content";
    
    // Write initial content
    add_file_content(initial_content, test_path).unwrap();
    
    // Overwrite with new content
    add_file_content(new_content, test_path).unwrap();
    
    // Verify new content
    let content = read_file(test_path).unwrap();
    assert_eq!(content, new_content);
    assert_ne!(content, initial_content);
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_add_content_append() {
    setup_test_env().unwrap();
    
    let test_path = "test/append_test.txt";
    let initial_content = "Initial";
    let appended_content = "Appended";
    
    // Create file with initial content
    add_file_content(initial_content, test_path).unwrap();
    
    // Append content
    add_content(appended_content, test_path).unwrap();
    
    // Verify both contents are present
    let content = read_file(test_path).unwrap();
    assert_eq!(content, format!("{}{}", initial_content, appended_content));
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_add_content_creates_file() {
    setup_test_env().unwrap();
    
    let test_path = "test/create_append_test.txt";
    let content = "New file content";
    
    // Append to non-existent file (should create it)
    add_content(content, test_path).unwrap();
    
    // Verify file was created with content
    let file_content = read_file(test_path).unwrap();
    assert_eq!(file_content, content);
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_add_file() {
    setup_test_env().unwrap();
    
    let test_path = "test/touch_test.txt";
    
    // Create empty file
    add_file(test_path).unwrap();
    
    // Verify file exists
    let full_path = get_full_path(test_path);
    assert!(full_path.exists());
    
    // Verify file is empty
    let content = read_file(test_path).unwrap();
    assert_eq!(content, "");
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_add_file_existing() {
    setup_test_env().unwrap();
    
    let test_path = "test/touch_existing_test.txt";
    let initial_content = "Some content";
    
    // Create file with content
    add_file_content(initial_content, test_path).unwrap();
    
    // Touch existing file (should not error)
    add_file(test_path).unwrap();
    
    // Verify file still exists (content may be cleared or preserved)
    let full_path = get_full_path(test_path);
    assert!(full_path.exists());
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_add_dir() {
    setup_test_env().unwrap();
    
    let test_path = "test/new_directory";
    
    // Create directory
    add_dir(test_path).unwrap();
    
    // Verify directory exists
    let full_path = get_full_path(test_path);
    assert!(full_path.exists());
    assert!(full_path.is_dir());
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_add_dir_nested() {
    setup_test_env().unwrap();
    
    let test_path = "test/nested/deep/directory";
    
    // Create nested directory
    add_dir(test_path).unwrap();
    
    // Verify nested directory exists
    let full_path = get_full_path(test_path);
    assert!(full_path.exists());
    assert!(full_path.is_dir());
    
    cleanup_test_env().unwrap();
}

#[test]
fn test_get_full_path() {
    let path = "test/file.txt";
    let full_path = get_full_path(path);
    
    assert!(full_path.to_string_lossy().contains(ROOT_FILE_PATH));
    assert!(full_path.to_string_lossy().contains("test/file.txt"));
}

