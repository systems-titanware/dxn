# Function Implementation Examples

This document provides examples of how to implement and use different function types in DXN.

## Structure

- **`dxn-shared`**: Shared types crate (no dependencies on dxn-core)
- **`dxn-wasm-wallet`**: WASM function example
- **`dxn-native-wallet`**: Native function example  
- **`dxn-remote-ai`**: Remote function server example
- **`dxn-files/scripts/`**: Script function examples (JavaScript/TypeScript)

## Building Examples

### WASM Functions

```bash
cd dxn-wasm-wallet
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM file will be at:
`target/wasm32-unknown-unknown/release/dxn_wasm_wallet.wasm`

### Native Functions

```bash
cd dxn-native-wallet
cargo build --release
```

The compiled library will be at:
- Linux: `target/release/libdxn_native_wallet.so`
- macOS: `target/release/libdxn_native_wallet.dylib`
- Windows: `target/release/dxn_native_wallet.dll`

### Remote Function Server

```bash
cd dxn-remote-ai
cargo build --release
cargo run --release
```

The server will start on `http://127.0.0.1:8081`

### Script Functions

Scripts are stored as files in `dxn-files/scripts/`. No separate compilation needed.

**Note**: To use script functions, build `dxn-core` with the `script-support` feature:

```bash
cd dxn-core
cargo build --features script-support
```

This enables JavaScript/TypeScript script execution using QuickJS (JavaScript runtime) and SWC (TypeScript transpiler).

## Configuration Examples

### WASM Function

```json
{
    "name": "wallet_get_balance_wasm",
    "functionType": "wasm",
    "version": 1,
    "path": "../dxn-wasm-wallet/target/wasm32-unknown-unknown/release/dxn_wasm_wallet.wasm",
    "functionName": "get_balance",
    "parameters": ["String"],
    "return": "String"
}
```

### Native Function

```json
{
    "name": "wallet_get_balance_native",
    "functionType": "native",
    "version": 1,
    "libraryPath": "../dxn-native-wallet/target/release/libdxn_native_wallet.so",
    "symbolName": "get_balance",
    "parameters": ["String"],
    "return": "String"
}
```

### Remote Function

```json
{
    "name": "ai_generate_text",
    "functionType": "remote",
    "version": 1,
    "serviceName": "my_ai",
    "endpoint": "/api/functions/generate_text",
    "parameters": ["String"],
    "return": "String"
}
```

**Note**: The `serviceName` must match a service in `serviceMesh.localServices` or `serviceMesh.publicServices`.

### Script Function (TypeScript)

```json
{
    "name": "wallet_transform_address",
    "functionType": "script",
    "version": 1,
    "scriptPath": "../dxn-files/scripts/wallet_transform.ts",
    "scriptLanguage": "typescript",
    "functionName": "transformAddress",
    "parameters": ["String"],
    "return": "String"
}
```

**Note**: Scripts can be written in JavaScript (`.js`) or TypeScript (`.ts`). TypeScript is automatically transpiled to JavaScript at runtime.

## Usage

All functions are called the same way regardless of type:

```rust
use dxn_core::functions::call_function;

let params = vec![serde_json::json!("0x1234567890abcdef")];
let result = call_function("wallet_get_balance_wasm", &params).await?;
```

## Function Type Comparison

| Type | Shareable | Performance | Customization | Use Case |
|------|-----------|------------|---------------|----------|
| WASM | ✅ Yes | ⭐⭐⭐ Good | ⭐⭐ Limited | Cross-platform, shareable functions |
| Native | ❌ No | ⭐⭐⭐⭐⭐ Excellent | ⭐⭐⭐⭐⭐ Full | Server-specific, performance-critical |
| Remote | ✅ Yes | ⭐⭐ Network overhead | ⭐⭐⭐ Good | Distributed, specialized resources |
| Script | ✅ Yes | ⭐⭐ Interpreter overhead | ⭐⭐⭐ Good | Quick iteration, simple logic |

## Notes

- **WASM functions** cannot directly import from `dxn-core` (circular dependency)
- **Native functions** can import from `dxn-core` for full server access
- **Remote functions** are standalone servers that can be written in any language
- **Script functions** are interpreted at runtime (JavaScript) or transpiled then executed (TypeScript), no separate compilation step needed
- **Script support** requires building with `--features script-support` and uses QuickJS (JavaScript runtime) + SWC (TypeScript transpiler)

