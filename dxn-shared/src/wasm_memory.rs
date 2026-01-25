// wasm_memory.rs - Simple memory helper for WASM functions
// Shared across all DXN WASM function crates
// Compatible with dxn-core executor which:
// - Writes input JSON to memory at offset 1024
// - Expects function to return (ptr: i32, len: i32) packed as i64
// - Reads result from the returned pointer

use serde::{Deserialize, Serialize};

// Fixed buffer for reading input (executor writes to offset 1024)
static mut INPUT_BUFFER: [u8; 8192] = [0; 8192];

// Fixed buffer for writing output (functions write to offset 2048)
static mut OUTPUT_BUFFER: [u8; 16384] = [0; 16384];
static mut OUTPUT_OFFSET: usize = 0;

/// Read JSON string from WASM memory
/// 
/// The executor writes input JSON to memory at the given pointer (offset 1024).
/// This function reads from that location.
/// 
/// # Note
/// This implementation uses a static buffer as a placeholder.
/// In production, you should read directly from WASM linear memory at the given pointer.
/// To do this, you'll need to import memory and use unsafe pointer access, or use
/// host-provided memory read functions.
/// 
/// # Safety
/// This function is unsafe because it uses static mutable buffers.
pub unsafe fn read_json_from_memory(ptr: i32, len: i32) -> Result<String, String> {
    // TODO: In production, read directly from WASM linear memory at ptr
    // For now, using static buffer as placeholder
    // The executor writes to WASM memory at offset 1024, but this static buffer
    // won't actually see that data. Update this to read from actual WASM memory.
    
    if len < 0 || len as usize > INPUT_BUFFER.len() {
        return Err(format!("Invalid length: {}", len));
    }
    
    // Placeholder: Read from static buffer
    // In production: Read from WASM linear memory at ptr
    let bytes = &INPUT_BUFFER[0..len as usize];
    
    String::from_utf8(bytes.to_vec())
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// Write result to WASM memory and return pointer and length packed as i64
/// 
/// Returns i64 where lower 32 bits = ptr, upper 32 bits = len
/// 
/// # Safety
/// This function is unsafe because it uses static mutable buffers.
pub unsafe fn write_result_to_memory(result: &str) -> i64 {
    let result_bytes = result.as_bytes();
    let len = result_bytes.len();
    
    // Check if we have space
    if OUTPUT_OFFSET + len > OUTPUT_BUFFER.len() {
        // Reset offset if buffer is full (simple circular buffer)
        OUTPUT_OFFSET = 0;
    }
    
    // Write to buffer
    let available_space = OUTPUT_BUFFER.len() - OUTPUT_OFFSET;
    let write_len = len.min(available_space);
    
    OUTPUT_BUFFER[OUTPUT_OFFSET..OUTPUT_OFFSET + write_len]
        .copy_from_slice(&result_bytes[..write_len]);
    
    // Return pointer (base offset 2048) and length packed as i64
    let ptr = 2048 + OUTPUT_OFFSET;
    OUTPUT_OFFSET = (OUTPUT_OFFSET + write_len + 1).min(OUTPUT_BUFFER.len() - 100);
    
    // Pack (ptr, len) into i64: lower 32 bits = ptr, upper 32 bits = len
    (ptr as i64) | ((write_len as i64) << 32)
}

/// Helper to deserialize JSON into a struct
/// 
/// Example:
/// ```rust
/// let config: MyConfig = deserialize_json(ptr, len)?;
/// ```
pub unsafe fn deserialize_json<T>(ptr: i32, len: i32) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let json_str = read_json_from_memory(ptr, len)?;
    serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to deserialize JSON: {}", e))
}

/// Helper to serialize a struct to JSON and write to memory
/// 
/// Returns i64 with (ptr, len) packed
pub unsafe fn serialize_and_write<T>(value: &T) -> i64
where
    T: Serialize,
{
    let json = serde_json::to_string(value)
        .unwrap_or_else(|_| r#"{"error":"Serialization failed"}"#.to_string());
    write_result_to_memory(&json)
}

/// Helper to create an error response
pub unsafe fn write_error(error_msg: &str) -> i64 {
    let error_json = format!(r#"{{"error":"{}"}}"#, error_msg);
    write_result_to_memory(&error_json)
}

/// Pack (ptr, len) into i64
/// Lower 32 bits = ptr, Upper 32 bits = len
pub fn pack_ptr_len(ptr: i32, len: i32) -> i64 {
    (ptr as i64) | ((len as i64) << 32)
}

