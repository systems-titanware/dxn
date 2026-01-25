# DXN WASM Function Template

This is a template example for creating DXN WASM functions.

## Quick Start

1. **Copy this crate** to create your own WASM function crate:
   ```bash
   cp -r dxn-wasm-template dxn-wasm-yourname
   cd dxn-wasm-yourname
   ```

2. **Update Cargo.toml**:
   - Change `name` to `dxn-wasm-yourname`
   - Update description

3. **Define your structs** in `src/lib.rs`:
   ```rust
   #[derive(Deserialize, Serialize)]
   struct YourInput {
       field1: String,
       field2: u64,
   }
   ```

4. **Write your functions**:
   ```rust
   #[no_mangle]
   pub extern "C" fn your_function(json_ptr: i32, json_len: i32) -> i64 {
       unsafe {
           let input: YourInput = deserialize_json(json_ptr, json_len)?;
           // Your logic here
           serialize_and_write(&result)
       }
   }
   ```

5. **Build**:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

6. **Add to config.json**:
   ```json
   {
     "name": "your_function",
     "functionType": "wasm",
     "path": "./dxn-wasm-yourname/target/wasm32-unknown-unknown/release/dxn_wasm_yourname.wasm",
     "functionName": "your_function"
   }
   ```

## Function Pattern

All WASM functions follow this pattern:

- **Input**: `(json_ptr: i32, json_len: i32)` - pointer to JSON in WASM memory
- **Output**: `i64` - packed `(ptr, len)` pointing to result JSON
- **Memory**: Executor writes input to offset 1024, functions write output to offset 2048+

## Memory Helpers

Use the shared memory helpers from `dxn_shared::wasm_memory`:

- `deserialize_json<T>(ptr, len)` - Read and deserialize JSON into your struct
- `serialize_and_write<T>(value)` - Serialize struct and write to memory
- `write_error(msg)` - Write error message as JSON
- `read_json_from_memory(ptr, len)` - Read JSON string from memory

## Examples

See `src/lib.rs` for complete examples:
- `process_data` - Accepts a struct, processes it, returns result
- `add_numbers` - Accepts JSON array of numbers
- `get_status` - No parameters, returns status

## Calling from Server

```rust
use crate::functions::manager;
use serde_json::json;

// Call with struct data
let params = vec![json!({
    "name": "test",
    "value": 100,
    "enabled": true
})];

let result = manager::call_function("process_data", &params).await?;
```

## Notes

- Functions are self-contained - define your own structs
- No external dependencies needed (except `serde` and `dxn_shared`)
- Memory helpers handle serialization/deserialization
- Error handling: Return JSON with `{"error": "message"}` on failure

