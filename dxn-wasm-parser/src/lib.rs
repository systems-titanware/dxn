use dxn_shared::wasm_memory::*;
use serde::{Deserialize, Serialize};
use pulldown_cmark;

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

/// WASM function that parses markdown to HTML
/// Accepts markdown text as JSON string (can be plain string or {"markdown": "..."})
#[no_mangle]
pub extern "C" fn parse_markdown(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Read JSON string from WASM memory
        let json_str = match read_json_from_memory(json_ptr, json_len) {
            Ok(s) => s,
            Err(e) => {
                return write_error(&e);
            }
        };
        
        // Parse markdown text from JSON input
        // Supports multiple formats:
        // 1. JSON array: ["markdown text"] (from API controller)
        // 2. JSON object: {"markdown": "text"}
        // 3. Plain string: "text"
        let markdown_text = match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(v) => {
                // Check if it's an array (from API: params are converted to array)
                if let Some(arr) = v.as_array() {
                    if let Some(first) = arr.first() {
                        // If first element is a string, use it
                        if let Some(s) = first.as_str() {
                            s.to_string()
                        } else if let Some(obj) = first.as_object() {
                            // If first element is an object, check for "markdown" field
                            obj.get("markdown")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string()
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    }
                } else if let Some(md) = v.get("markdown") {
                    // Object with "markdown" field
                    md.as_str().unwrap_or("").to_string()
                } else if v.is_string() {
                    // Plain string
                    v.as_str().unwrap_or("").to_string()
                } else {
                    "".to_string()
                }
            },
            Err(_) => {
                // Try as plain string (remove quotes if present)
                json_str.trim_matches('"').to_string()
            }
        };
        
        if markdown_text.is_empty() {
            return write_error("No markdown content provided");
        }
        
        // Parse markdown to HTML using pulldown_cmark
        let parser = pulldown_cmark::Parser::new(&markdown_text);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        
        // Return HTML result
        let result_json = serde_json::json!({
            "html": html_output,
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