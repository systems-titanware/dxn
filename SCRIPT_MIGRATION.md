# Script Function Migration: Lua → JavaScript/TypeScript

## Summary

DXN's script function support has been migrated from Lua to JavaScript/TypeScript using:
- **QuickJS** (`rquickjs`) - Lightweight JavaScript runtime
- **SWC** - TypeScript transpiler (transpiles TS to JS at runtime)

## Changes Made

### 1. Dependencies Updated

**Removed:**
- `mlua` (Lua bindings)

**Added:**
- `rquickjs` - QuickJS JavaScript runtime
- `swc-common`, `swc-ecma-parser`, `swc-ecma-transforms`, `swc-ecma-codegen` - TypeScript transpilation

### 2. Code Changes

- **`dxn-core/src/functions/executors.rs`**: Replaced Lua executor with JavaScript/TypeScript executor
- **`dxn-core/config.json`**: Updated script examples to use TypeScript
- **`dxn-files/scripts/`**: Converted Lua scripts to TypeScript

### 3. Documentation Updated

- **`function_architecture.md`**: Updated all Lua references to JavaScript/TypeScript
- **`FUNCTION_EXAMPLES.md`**: Updated examples and build instructions

## Building

### Without Script Support (Default)
```bash
cd dxn-core
cargo build
```

### With Script Support
```bash
cd dxn-core
cargo build --features script-support
```

**Note**: No external dependencies needed! QuickJS is embedded, and SWC is a Rust crate.

## Writing Script Functions

### TypeScript Example
```typescript
// transform.ts
export function transformAddress(address: string): string {
    return address.toLowerCase().startsWith("0x") 
        ? address.toLowerCase() 
        : "0x" + address.toLowerCase();
}
```

### JavaScript Example
```javascript
// transform.js
export function transformAddress(address) {
    return address.toLowerCase().startsWith("0x") 
        ? address.toLowerCase() 
        : "0x" + address.toLowerCase();
}
```

## Configuration

```json
{
    "name": "wallet_transform",
    "functionType": "script",
    "scriptPath": "../dxn-files/scripts/wallet_transform.ts",
    "scriptLanguage": "typescript",
    "functionName": "transformAddress",
    "parameters": ["String"],
    "return": "String"
}
```

## Benefits

1. **More Popular**: JavaScript/TypeScript is more widely known than Lua
2. **Type Safety**: TypeScript provides compile-time type checking
3. **Better Tooling**: Rich ecosystem of editors, linters, and formatters
4. **No External Dependencies**: QuickJS is embedded, no system Lua installation needed
5. **Modern Features**: ES2020 support via QuickJS

## Migration Notes

- Old Lua scripts need to be converted to JavaScript/TypeScript
- Function names should use camelCase (JavaScript convention) instead of snake_case (Lua convention)
- TypeScript files (`.ts`) are automatically transpiled to JavaScript before execution

