// wasm_memory.rs - Simple memory helper for WASM functions
// Shared across all DXN WASM function crates
// Compatible with dxn-core executor which:
// - Writes input JSON to memory at offset 1024
// - Expects function to return (ptr: i32, len: i32) packed as i64
// - Reads result from the returned pointer

use serde::{Deserialize, Serialize};
use core::ptr;

// Output buffer offset - where we write results in WASM memory
// We use offset 2048 to avoid conflicts with input (at 1024)
const OUTPUT_BASE: usize = 2048;
const OUTPUT_BUFFER_SIZE: usize = 16384;

// Track where to write next output (simple allocator)
static mut OUTPUT_OFFSET: usize = 0;

/// Read JSON string from WASM linear memory
/// 
/// The executor writes input JSON to WASM memory at the given pointer.
/// This function reads directly from WASM linear memory at that location.
/// 
/// # Safety
/// This function is unsafe because it performs raw pointer operations.
/// The caller must ensure ptr and len are valid.
pub unsafe fn read_json_from_memory(ptr: i32, len: i32) -> Result<String, String> {
    if len < 0 {
        return Err(format!("Invalid length: {}", len));
    }
    
    let len_usize = len as usize;
    
    // Validate length is reasonable (prevent buffer overflow)
    if len_usize > 1024 * 1024 {  // 1MB max
        return Err(format!("Length too large: {}", len));
    }
    
    // Read from WASM linear memory at the given pointer
    // In WASM, memory starts at address 0, so we can use the pointer directly
    let mut bytes = vec![0u8; len_usize];
    
    // Copy bytes from WASM memory to our buffer
    // ptr::copy_nonoverlapping is safe for non-overlapping regions
    ptr::copy_nonoverlapping(
        ptr as *const u8,
        bytes.as_mut_ptr(),
        len_usize
    );
    
    String::from_utf8(bytes)
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// Write result to WASM linear memory and return pointer and length packed as i64
/// 
/// Returns i64 where lower 32 bits = ptr, upper 32 bits = len
/// 
/// # Safety
/// This function is unsafe because it performs raw pointer operations.
pub unsafe fn write_result_to_memory(result: &str) -> i64 {
    let result_bytes = result.as_bytes();
    let len = result_bytes.len();
    
    // Check if we have space
    if OUTPUT_OFFSET + len > OUTPUT_BUFFER_SIZE {
        // Reset if buffer is full (simple circular buffer)
        OUTPUT_OFFSET = 0;
    }
    
    // Calculate write pointer in WASM memory
    let write_ptr = OUTPUT_BASE + OUTPUT_OFFSET;
    
    // Ensure we don't overflow
    let available_space = OUTPUT_BUFFER_SIZE - OUTPUT_OFFSET;
    let write_len = len.min(available_space);
    
    // Write to WASM linear memory
    // ptr::copy_nonoverlapping is safe for non-overlapping regions
    ptr::copy_nonoverlapping(
        result_bytes.as_ptr(),
        write_ptr as *mut u8,
        write_len
    );
    
    // Update offset for next write
    OUTPUT_OFFSET = (OUTPUT_OFFSET + write_len + 1).min(OUTPUT_BUFFER_SIZE - 100);
    
    // Pack (ptr, len) into i64: lower 32 bits = ptr, upper 32 bits = len
    (write_ptr as i64) | ((write_len as i64) << 32)
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

