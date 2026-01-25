// dxn-wasm-template
// 
// This is a template example for creating DXN WASM functions.
// Copy this crate and modify it to create your own WASM function crate.
//
// Build command:
//   cargo build --target wasm32-unknown-unknown --release
//
// The resulting .wasm file should be placed in your config.json

use dxn_shared::wasm_memory::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// STEP 1: Define your structs
// ============================================================================
// Define structs for input and output data.
// These structs are internal to your crate - no external dependencies needed!

#[derive(Deserialize, Serialize, Debug)]
struct MyInput {
    name: String,
    value: u64,
    enabled: bool,
}

#[derive(Serialize, Debug)]
struct MyOutput {
    result: String,
    computed_value: u64,
    status: String,
}

// ============================================================================
// STEP 2: Write your WASM functions
// ============================================================================
// All WASM functions follow this pattern:
// - Accept: (json_ptr: i32, json_len: i32) - pointer to JSON input in WASM memory
// - Return: i64 - packed (ptr, len) pointing to JSON result in WASM memory
//
// The executor automatically:
// - Writes input JSON to memory at offset 1024
// - Calls your function with (1024, len)
// - Reads result from the returned pointer

/// Example function that processes input and returns a result
/// 
/// This function:
/// 1. Reads JSON from WASM memory
/// 2. Deserializes into your struct
/// 3. Processes the data
/// 4. Serializes result and writes to memory
/// 5. Returns (ptr, len) packed as i64
#[no_mangle]
pub extern "C" fn process_data(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Step 1: Deserialize JSON into your struct
        let input: MyInput = match deserialize_json(json_ptr, json_len) {
            Ok(data) => data,
            Err(e) => {
                // Return error if deserialization fails
                return write_error(&format!("Failed to parse input: {}", e));
            }
        };
        
        // Step 2: Process the data (your business logic here)
        let computed_value = input.value * 2;
        let result = format!("Processed: {}", input.name);
        let status = if input.enabled { "active" } else { "inactive" }.to_string();
        
        // Step 3: Create output struct
        let output = MyOutput {
            result,
            computed_value,
            status,
        };
        
        // Step 4: Serialize and write result to memory
        serialize_and_write(&output)
    }
}

/// Example function that accepts primitives (as JSON array)
/// 
/// Even for simple parameters, use JSON format for consistency.
/// The executor passes params as a JSON array: [value1, value2, ...]
#[no_mangle]
pub extern "C" fn add_numbers(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Read JSON string
        let json_str = match read_json_from_memory(json_ptr, json_len) {
            Ok(s) => s,
            Err(e) => {
                return write_error(&e);
            }
        };
        
        // Parse as JSON array
        let numbers: Vec<u64> = match serde_json::from_str(&json_str) {
            Ok(n) => n,
            Err(e) => {
                return write_error(&format!("Failed to parse numbers: {}", e));
            }
        };
        
        if numbers.len() < 2 {
            return write_error("Expected at least 2 numbers");
        }
        
        // Process
        let sum: u64 = numbers.iter().sum();
        
        // Return result
        let result = serde_json::json!({
            "sum": sum,
            "count": numbers.len()
        });
        
        serialize_and_write(&result)
    }
}

/// Example function with no parameters
#[no_mangle]
pub extern "C" fn get_status() -> i64 {
    // Note: SystemTime is not available in WASM, use a placeholder or host function
    let status = serde_json::json!({
        "status": "ok",
        "version": "1.0.0",
        "message": "Function is operational"
    });
    
    unsafe {
        serialize_and_write(&status)
    }
}

// ============================================================================
// STEP 3: Export your functions
// ============================================================================
// Functions marked with #[no_mangle] and pub extern "C" are automatically
// exported and can be called by the executor.
//
// Make sure the function name in your config.json matches the function name here!

// ============================================================================
// USAGE IN CONFIG.JSON
// ============================================================================
// Add to your config.json:
//
// {
//   "name": "process_data",
//   "functionType": "wasm",
//   "version": 1,
//   "path": "./dxn-wasm-template/target/wasm32-unknown-unknown/release/dxn_wasm_template.wasm",
//   "functionName": "process_data",
//   "parameters": ["MyInput"]
// }
//
// Then call from your server:
//   let params = vec![json!({
//       "name": "test",
//       "value": 100,
//       "enabled": true
//   })];
//   manager::call_function("process_data", &params).await

