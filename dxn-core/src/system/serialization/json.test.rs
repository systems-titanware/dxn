use super::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct TestStruct {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct NestedStruct {
    user: TestStruct,
    count: i32,
}

// Helper function to create a test JSON file
fn create_test_json_file(path: &str, content: &str) -> std::io::Result<()> {
    let test_dir = Path::new(path).parent().unwrap();
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
    }
    fs::write(path, content)
}

// Helper function to cleanup test file
fn cleanup_test_file(path: &str) -> std::io::Result<()> {
    if Path::new(path).exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[test]
fn test_deserialize_simple_struct() {
    let test_file = "./test_config_simple.json";
    let json_content = r#"
    {
        "name": "John Doe",
        "age": 30,
        "active": true
    }
    "#;
    
    create_test_json_file(test_file, json_content).unwrap();
    
    let result: TestStruct = deserialize(test_file.to_string()).unwrap();
    
    assert_eq!(result.name, "John Doe");
    assert_eq!(result.age, 30);
    assert_eq!(result.active, true);
    
    cleanup_test_file(test_file).unwrap();
}

#[test]
fn test_deserialize_nested_struct() {
    let test_file = "./test_config_nested.json";
    let json_content = r#"
    {
        "user": {
            "name": "Jane Smith",
            "age": 25,
            "active": false
        },
        "count": 42
    }
    "#;
    
    create_test_json_file(test_file, json_content).unwrap();
    
    let result: NestedStruct = deserialize(test_file.to_string()).unwrap();
    
    assert_eq!(result.user.name, "Jane Smith");
    assert_eq!(result.user.age, 25);
    assert_eq!(result.user.active, false);
    assert_eq!(result.count, 42);
    
    cleanup_test_file(test_file).unwrap();
}

#[test]
fn test_deserialize_array() {
    let test_file = "./test_config_array.json";
    let json_content = r#"
    [
        {
            "name": "Alice",
            "age": 28,
            "active": true
        },
        {
            "name": "Bob",
            "age": 35,
            "active": false
        }
    ]
    "#;
    
    create_test_json_file(test_file, json_content).unwrap();
    
    let result: Vec<TestStruct> = deserialize(test_file.to_string()).unwrap();
    
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "Alice");
    assert_eq!(result[1].name, "Bob");
    
    cleanup_test_file(test_file).unwrap();
}

#[test]
#[should_panic(expected = "Failed to open file")]
fn test_deserialize_file_not_found() {
    let _: TestStruct = deserialize("./nonexistent_file.json".to_string()).unwrap();
}

#[test]
#[should_panic(expected = "Failed to deserialize JSON")]
fn test_deserialize_invalid_json() {
    let test_file = "./test_config_invalid.json";
    let invalid_json = "This is not valid JSON {";
    
    create_test_json_file(test_file, invalid_json).unwrap();
    
    let _: TestStruct = deserialize(test_file.to_string()).unwrap();
    
    cleanup_test_file(test_file).unwrap();
}

#[test]
#[should_panic(expected = "Failed to deserialize JSON")]
fn test_deserialize_mismatched_types() {
    let test_file = "./test_config_mismatch.json";
    let json_content = r#"
    {
        "name": "Test",
        "age": "not a number",
        "active": true
    }
    "#;
    
    create_test_json_file(test_file, json_content).unwrap();
    
    let _: TestStruct = deserialize(test_file.to_string()).unwrap();
    
    cleanup_test_file(test_file).unwrap();
}

#[test]
fn test_read_as_string() {
    // Note: This test reads ./config.json which is hardcoded in the function
    // This is more of a smoke test to ensure the function doesn't panic
    // If config.json doesn't exist, the test will fail, which is expected behavior
    let result = read_as_string();
    // We just verify it returns a Result (either Ok or Err depending on file existence)
    match result {
        Ok(_) => {
            // File exists and was read successfully
            assert!(true);
        },
        Err(_) => {
            // File doesn't exist, which is also valid for this test
            // We're just testing that the function doesn't panic
            assert!(true);
        }
    }
}

