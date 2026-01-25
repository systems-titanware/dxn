use dxn_shared::wasm_memory::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    multiplier: u32,
    offset: u32,
}

/// WASM function with no parameters
#[no_mangle]
pub extern "C" fn no_params() -> i64 {
    let result = (10 * 1) + 5;
    let result_json = serde_json::json!({
        "result": result
    });
    unsafe {
        serialize_and_write(&result_json)
    }
}

/// WASM function with no parameters but returns a result
#[no_mangle]
pub extern "C" fn no_params_with_result() -> i64 {
    let result = (10 * 1) + 5;
    let result_json = serde_json::json!({
        "result": result
    });
    unsafe {
        serialize_and_write(&result_json)
    }
}

/// WASM function that accepts JSON config
#[no_mangle]
pub extern "C" fn serialized_params(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Deserialize the input JSON string
        let config: Config = match deserialize_json(json_ptr, json_len) {
            Ok(cfg) => cfg,
            Err(e) => {
                // Return error as JSON
                let error_json = serde_json::json!({
                    "error": format!("Failed to parse JSON: {}", e)
                });
                return serialize_and_write(&error_json);
            }
        };

        // Perform some logic (replace with your actual library logic)
        let result = (10 * config.multiplier) + config.offset;

        // Return result as JSON
        let result_json = serde_json::json!({
            "result": result
        });
        serialize_and_write(&result_json)
    }
}

/// WASM function that accepts two i32 parameters
/// Note: For primitives, we still use JSON format for consistency
#[no_mangle]
pub extern "C" fn typed_params(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Read JSON which should be an array [left, right]
        let json_str = match read_json_from_memory(json_ptr, json_len) {
            Ok(s) => s,
            Err(e) => {
                return write_error(&e);
            }
        };
        
        // Parse as JSON array
        let params: Vec<i32> = match serde_json::from_str(&json_str) {
            Ok(p) => p,
            Err(e) => {
                return write_error(&format!("Failed to parse params: {}", e));
            }
        };
        
        if params.len() < 2 {
            return write_error("Expected 2 parameters");
        }
        
        let left = params[0];
        let right = params[1];
        let result = left + right;
        
        let result_json = serde_json::json!({
            "result": result
        });
        serialize_and_write(&result_json)
    }
}

/// WASM function that parses markdown (accepts path as JSON string)
#[no_mangle]
pub extern "C" fn parse_markdown(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Read JSON string
        let json_str = match read_json_from_memory(json_ptr, json_len) {
            Ok(s) => s,
            Err(e) => {
                return write_error(&e);
            }
        };
        
        // Parse path from JSON (could be just a string or {"path": "..."})
        let path = match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(v) => {
                if let Some(p) = v.get("path") {
                    p.as_str().unwrap_or("").to_string()
                } else if v.is_string() {
                    v.as_str().unwrap_or("").to_string()
                } else {
                    "".to_string()
                }
            },
            Err(_) => {
                // Try as plain string
                json_str.trim_matches('"').to_string()
            }
        };
        
        // Placeholder implementation
        // In a real implementation, you would read the file and parse markdown
        let result_json = serde_json::json!({
            "path": path,
            "content": "Parsed markdown content would go here",
            "status": "success"
        });
        
        serialize_and_write(&result_json)
    }
}

/*
 UPDATE ABOVE FUNCTION TO THE BELOW

#[plugin_fn]
pub fn process_data(json_data: String) -> FnResult<String> {
    // Deserialize the input JSON string
    let config: Config = serde_json::from_str(&json_data)?;

    // Perform some logic (replace with your actual library logic)
    let result = (10 * config.multiplier) + config.offset;

    // Return a result
    Ok(format!("Computed result: {}", result))
}


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
 */