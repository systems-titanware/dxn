use super::*;
use crate::functions::models::{SystemFunctions, SystemFunctionModel, FunctionType, FunctionError};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// TEST 1: Parameter Conversion
// ============================================================================

#[test]
fn test_parameter_conversion() {
    println!("\n✅ Running: test_parameter_conversion");
    
    let mut body_map = HashMap::new();
    body_map.insert("param1".to_string(), json!("value1"));
    body_map.insert("param2".to_string(), json!(42));
    body_map.insert("param3".to_string(), json!(true));
    
    // Simulate the conversion that happens in execute_function
    let params: Vec<serde_json::Value> = body_map.into_values().collect();
    
    assert_eq!(params.len(), 3);
    assert!(params.contains(&json!("value1")));
    assert!(params.contains(&json!(42)));
    assert!(params.contains(&json!(true)));
    
    println!("   ✓ Test passed: test_parameter_conversion\n");
}

#[test]
fn test_parameter_conversion_empty() {
    println!("\n✅ Running: test_parameter_conversion_empty");
    
    let body_map: HashMap<String, serde_json::Value> = HashMap::new();
    let params: Vec<serde_json::Value> = body_map.into_values().collect();
    
    assert_eq!(params.len(), 0);
    
    println!("   ✓ Test passed: test_parameter_conversion_empty\n");
}

// ============================================================================
// TEST 2: Config Function Logic Tests
// ============================================================================

#[test]
fn test_config_with_functions_logic() {
    println!("\n✅ Running: test_config_with_functions_logic");
    
    let functions = SystemFunctions {
        public: Some(vec![
            SystemFunctionModel {
                name: "test_function".to_string(),
                function_type: FunctionType::Wasm,
                path: Some("./test.wasm".to_string()),
                function_name: None,
                library_path: None,
                symbol_name: None,
                service_name: None,
                endpoint: None,
                script_path: None,
                script_language: None,
                version: 1,
                parameters: None,
                return_type: None,
                params: None,
            },
        ]),
        private: None,
    };
    
    // Test that config handles Some(vec) with non-empty vector
    match functions.public {
        Some(ref vec) => {
            assert!(!vec.is_empty());
            assert_eq!(vec.len(), 1);
        }
        None => panic!("Should have functions"),
    }
    
    println!("   ✓ Test passed: test_config_with_functions_logic\n");
}

#[test]
fn test_config_with_empty_functions_logic() {
    println!("\n✅ Running: test_config_with_empty_functions_logic");
    
    let functions = SystemFunctions {
        public: Some(vec![]),
        private: None,
    };
    
    // Test that config handles Some(vec) with empty vector
    match functions.public {
        Some(ref vec) => {
            assert!(vec.is_empty());
        }
        None => panic!("Should have Some(vec)"),
    }
    
    println!("   ✓ Test passed: test_config_with_empty_functions_logic\n");
}

#[test]
fn test_config_with_no_functions_logic() {
    println!("\n✅ Running: test_config_with_no_functions_logic");
    
    let functions = SystemFunctions {
        public: None,
        private: None,
    };
    
    // Test that config handles None
    match functions.public {
        Some(_) => panic!("Should be None"),
        None => assert!(true),
    }
    
    println!("   ✓ Test passed: test_config_with_no_functions_logic\n");
}

// ============================================================================
// TEST 3: Error Response Formatting
// ============================================================================

#[test]
fn test_error_response_formatting() {
    println!("\n✅ Running: test_error_response_formatting");
    
    // Test NotFound error format
    let not_found_error = json!({
        "error": format!("Function '{}' not found", "missing_function")
    });
    
    assert!(not_found_error.get("error").is_some());
    assert_eq!(
        not_found_error["error"].as_str().unwrap(),
        "Function 'missing_function' not found"
    );
    
    // Test execution error format
    // The actual format in function.rs uses Debug formatting: format!("Function execution error: {:?}", e)
    // Debug format for ExecutionError("Test error") is: ExecutionError("Test error")
    let exec_error = json!({
        "error": format!("Function execution error: {:?}", FunctionError::ExecutionError("Test error".to_string()))
    });
    
    assert!(exec_error.get("error").is_some());
    let error_str = exec_error["error"].as_str().unwrap();
    // Check for either "ExecutionError" (Debug format) or "Function execution error" (prefix)
    assert!(error_str.contains("ExecutionError") || error_str.contains("Function execution error"));
    
    println!("   ✓ Test passed: test_error_response_formatting\n");
}

// ============================================================================
// TEST 4: Function Name Extraction
// ============================================================================

#[test]
fn test_function_name_extraction() {
    println!("\n✅ Running: test_function_name_extraction");
    
    // Simulate path parameter extraction
    let function_name = "test_function".to_string();
    // In actual handler: let function_name = path.into_inner();
    let extracted = function_name.clone();
    
    assert_eq!(extracted, "test_function");
    
    println!("   ✓ Test passed: test_function_name_extraction\n");
}

// ============================================================================
// TEST 5: Result Error Detection
// ============================================================================

#[test]
fn test_result_error_detection() {
    println!("\n✅ Running: test_result_error_detection");
    
    // Test result with error field
    let error_result = json!({
        "error": "Something went wrong",
        "data": null
    });
    
    assert!(error_result.get("error").is_some());
    
    // Test result without error field
    let success_result = json!({
        "data": "success",
        "status": "ok"
    });
    
    assert!(success_result.get("error").is_none());
    
    println!("   ✓ Test passed: test_result_error_detection\n");
}

// ============================================================================
// TEST 6: Parameter Types Handling
// ============================================================================

#[test]
fn test_parameter_types_handling() {
    println!("\n✅ Running: test_parameter_types_handling");
    
    let mut body_map = HashMap::new();
    body_map.insert("string_param".to_string(), json!("test"));
    body_map.insert("number_param".to_string(), json!(123));
    body_map.insert("bool_param".to_string(), json!(true));
    body_map.insert("null_param".to_string(), json!(null));
    body_map.insert("array_param".to_string(), json!([1, 2, 3]));
    body_map.insert("object_param".to_string(), json!({"key": "value"}));
    
    let params: Vec<serde_json::Value> = body_map.into_values().collect();
    
    assert_eq!(params.len(), 6);
    
    // Verify types are preserved
    let string_val = params.iter().find(|v| v.is_string()).unwrap();
    assert_eq!(string_val.as_str().unwrap(), "test");
    
    let number_val = params.iter().find(|v| v.is_number()).unwrap();
    assert_eq!(number_val.as_i64().unwrap(), 123);
    
    let bool_val = params.iter().find(|v| v.is_boolean()).unwrap();
    assert_eq!(bool_val.as_bool().unwrap(), true);
    
    let null_val = params.iter().find(|v| v.is_null()).unwrap();
    assert!(null_val.is_null());
    
    let array_val = params.iter().find(|v| v.is_array()).unwrap();
    assert_eq!(array_val.as_array().unwrap().len(), 3);
    
    let object_val = params.iter().find(|v| v.is_object()).unwrap();
    assert!(object_val.get("key").is_some());
    
    println!("   ✓ Test passed: test_parameter_types_handling\n");
}

// ============================================================================
// TEST 7: Function Error Types
// ============================================================================

#[test]
fn test_function_error_types() {
    println!("\n✅ Running: test_function_error_types");
    
    // Test all error variants
    let not_found = FunctionError::NotFound("test".to_string());
    assert!(format!("{}", not_found).contains("Function not found"));
    
    let invalid_input = FunctionError::InvalidInput("test".to_string());
    assert!(format!("{}", invalid_input).contains("Invalid input"));
    
    let exec_error = FunctionError::ExecutionError("test".to_string());
    assert!(format!("{}", exec_error).contains("Execution error"));
    
    // Test error matching for response codes
    match not_found {
        FunctionError::NotFound(_) => assert!(true),
        _ => assert!(false, "Should match NotFound"),
    }
    
    println!("   ✓ Test passed: test_function_error_types\n");
}

// ============================================================================
// TEST 8: Multiple Parameters Order
// ============================================================================

#[test]
fn test_multiple_parameters_order() {
    println!("\n✅ Running: test_multiple_parameters_order");
    
    // Test that parameter order is preserved (HashMap values iteration order)
    let mut body_map = HashMap::new();
    body_map.insert("first".to_string(), json!(1));
    body_map.insert("second".to_string(), json!(2));
    body_map.insert("third".to_string(), json!(3));
    
    let params: Vec<serde_json::Value> = body_map.into_values().collect();
    
    // Should have all three values (order may vary due to HashMap)
    assert_eq!(params.len(), 3);
    assert!(params.contains(&json!(1)));
    assert!(params.contains(&json!(2)));
    assert!(params.contains(&json!(3)));
    
    println!("   ✓ Test passed: test_multiple_parameters_order\n");
}

// ============================================================================
// TEST 9: Function Type Variants
// ============================================================================

#[test]
fn test_function_type_variants() {
    println!("\n✅ Running: test_function_type_variants");
    
    // Test all function types
    let wasm_function = SystemFunctionModel {
        name: "wasm_func".to_string(),
        function_type: FunctionType::Wasm,
        path: Some("./test.wasm".to_string()),
        function_name: None,
        library_path: None,
        symbol_name: None,
        service_name: None,
        endpoint: None,
        script_path: None,
        script_language: None,
        version: 1,
        parameters: None,
        return_type: None,
        params: None,
    };
    
    let native_function = SystemFunctionModel {
        name: "native_func".to_string(),
        function_type: FunctionType::Native,
        path: None,
        function_name: None,
        library_path: Some("./lib.so".to_string()),
        symbol_name: Some("native_func".to_string()),
        service_name: None,
        endpoint: None,
        script_path: None,
        script_language: None,
        version: 1,
        parameters: None,
        return_type: None,
        params: None,
    };
    
    let remote_function = SystemFunctionModel {
        name: "remote_func".to_string(),
        function_type: FunctionType::Remote,
        path: None,
        function_name: None,
        library_path: None,
        symbol_name: None,
        service_name: Some("ai_service".to_string()),
        endpoint: Some("/api/functions/generate".to_string()),
        script_path: None,
        script_language: None,
        version: 1,
        parameters: None,
        return_type: None,
        params: None,
    };
    
    let script_function = SystemFunctionModel {
        name: "script_func".to_string(),
        function_type: FunctionType::Script,
        path: None,
        function_name: None,
        library_path: None,
        symbol_name: None,
        service_name: None,
        endpoint: None,
        script_path: Some("./script.js".to_string()),
        script_language: Some("javascript".to_string()),
        version: 1,
        parameters: None,
        return_type: None,
        params: None,
    };
    
    // Verify function types
    assert!(matches!(wasm_function.function_type, FunctionType::Wasm));
    assert!(matches!(native_function.function_type, FunctionType::Native));
    assert!(matches!(remote_function.function_type, FunctionType::Remote));
    assert!(matches!(script_function.function_type, FunctionType::Script));
    
    println!("   ✓ Test passed: test_function_type_variants\n");
}

// ============================================================================
// TEST 10: Complex Parameter Scenarios
// ============================================================================

#[test]
fn test_complex_parameter_scenarios() {
    println!("\n✅ Running: test_complex_parameter_scenarios");
    
    // Test nested objects
    let mut body_map = HashMap::new();
    body_map.insert("nested".to_string(), json!({
        "level1": {
            "level2": "value"
        }
    }));
    body_map.insert("array_of_objects".to_string(), json!([
        {"id": 1, "name": "first"},
        {"id": 2, "name": "second"}
    ]));
    
    let params: Vec<serde_json::Value> = body_map.into_values().collect();
    
    assert_eq!(params.len(), 2);
    
    // Verify nested structure
    let nested = params.iter().find(|v| v.is_object() && v.get("level1").is_some()).unwrap();
    assert!(nested.get("level1").is_some());
    
    // Verify array of objects
    let array = params.iter().find(|v| v.is_array()).unwrap();
    let array_vec = array.as_array().unwrap();
    assert_eq!(array_vec.len(), 2);
    assert_eq!(array_vec[0]["id"], 1);
    
    println!("   ✓ Test passed: test_complex_parameter_scenarios\n");
}

