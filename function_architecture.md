# Function Architecture - Implementation Options Analysis

## Recommended Implementation: Hybrid Approach with WASM as Default

### Why Hybrid?

**1. WASM as Default for Shareability**
- Cross-platform compatibility
- Secure isolation
- Easy to share and distribute
- Good for most general-purpose functions

**2. Native for Customization**
- Full server access when needed
- Better performance for critical paths
- For server-specific logic
- When you need deep integration with server internals

**3. Remote for Distribution**
- Works seamlessly with service mesh architecture
- Specialized function servers
- Scalable and independent
- Language-agnostic

**4. Scripts as Optional**
- Quick iteration and prototyping
- Simple functions that don't need compilation
- Good for frequently-modified logic

### Implementation Strategy

**Phase 1: Keep WASM, Add Native**
- WASM remains default and primary
- Add native Rust library support
- Maintain full backward compatibility
- Users can opt into native when needed

**Phase 2: Add Remote**
- Integrate with service mesh
- Remote function calls via HTTP
- OAuth support for secure access
- Function discovery from registry

**Phase 3: Optional Scripts**
- Add scripting support (if needed)
- Lua or JavaScript runtime
- For quick iteration and simple logic
- Optional feature, not required

### Benefits Summary

- **Versatile**: Right tool for each function
- **Extensible**: Easy to add new function types
- **Shareable**: WASM for sharing, native for customization
- **Backward Compatible**: Existing WASM functions work unchanged
- **Performance**: Native when you need it
- **Distributed**: Remote functions via service mesh

---

## Data Models for Hybrid Approach

### Enhanced Function Model

```rust
// In functions/models.rs

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FunctionType {
    Wasm,      // Current: WASM modules (default, shareable)
    Native,    // New: Native Rust dynamic libraries
    Remote,    // New: Remote function servers
    Script,    // Optional: Scripting languages (Lua/JS)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemFunctionModel {
    pub(crate) name: String,
    #[serde(default = "default_function_type")]
    pub(crate) function_type: FunctionType,  // Defaults to Wasm
    
    // For WASM functions (current)
    #[serde(default)]
    pub(crate) path: Option<String>,  // Path to .wasm file
    #[serde(default)]
    pub(crate) function_name: Option<String>,  // Exported function name
    
    // For Native functions
    #[serde(default)]
    pub(crate) library_path: Option<String>,  // Path to .so/.dylib
    #[serde(default)]
    pub(crate) symbol_name: Option<String>,  // Function symbol name
    
    // For Remote functions
    #[serde(default)]
    pub(crate) service_name: Option<String>,  // Service in mesh
    #[serde(default)]
    pub(crate) endpoint: Option<String>,     // Function endpoint URL
    
    // For Script functions
    #[serde(default)]
    pub(crate) script_path: Option<String>,  // Path to script file
    #[serde(default)]
    pub(crate) script_language: Option<String>, // "lua", "javascript"
    
    pub(crate) version: u32,
    #[serde(default)]
    pub(crate) parameters: Option<Vec<String>>,  // Parameter types
    #[serde(default)]
    pub(crate) return_type: Option<String>,     // Return type
}

fn default_function_type() -> FunctionType {
    FunctionType::Wasm
}
```

### Enhanced Function Manager

```rust
// In functions/manager.rs

pub enum FunctionExecutor {
    Wasm(WasmExecutor),
    Native(NativeExecutor),
    Remote(RemoteExecutor),
    Script(ScriptExecutor),
}

/// Unified function call API
pub fn call_function(
    name: &str,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    let function = get_function(name)?;
    
    match function.function_type {
        FunctionType::Wasm => {
            // Current WASM execution
            execute_wasm(function, params)
        },
        FunctionType::Native => {
            // Native Rust library execution
            execute_native(function, params)
        },
        FunctionType::Remote => {
            // Remote HTTP call
            execute_remote(function, params)
        },
        FunctionType::Script => {
            // Script execution
            execute_script(function, params)
        }
    }
}

// WASM executor (current implementation)
fn execute_wasm(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    // Current WASM execution logic
    // Convert params to WASM types
    // Call via Wasmtime
    // Return result
}

// Native executor (new)
fn execute_native(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    // Load dynamic library
    // Get function symbol
    // Call native function
    // Return result
}

// Remote executor (new)
fn execute_remote(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    // Resolve service from mesh
    // Get OAuth token
    // Make HTTP request
    // Return result
}

// Script executor (optional)
fn execute_script(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    // Load script file
    // Execute in interpreter
    // Return result
}
```

---

## Configuration Examples

### WASM Function (Default, Shareable)

```json
{
    "name": "parse_markdown",
    "functionType": "wasm",
    "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
    "functionName": "parse_markdown",
    "version": 1,
    "parameters": ["String"],
    "return": "String"
}
```

**Or simplified (defaults to WASM):**
```json
{
    "name": "parse_markdown",
    "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
    "functionName": "parse_markdown",
    "version": 1
}
```

### Native Function (Performance/Customization)

```json
{
    "name": "custom_processing",
    "functionType": "native",
    "libraryPath": "../dxn_public/functions/target/release/libcustom_functions.so",
    "symbolName": "custom_processing",
    "version": 1,
    "parameters": ["i32", "i32"],
    "return": "i32"
}
```

**Native Function Implementation:**
```rust
// In custom_functions library
use dxn_core::system::files::manager;
use dxn_core::data::db::sqlite;

#[no_mangle]
pub extern "C" fn custom_processing(a: i32, b: i32) -> i32 {
    // Direct access to server internals
    let file = manager::read_file("config.txt").unwrap();
    // Can use any Rust crate
    // Can access database directly
    a + b + file.len() as i32
}
```

### Remote Function (Distributed)

```json
{
    "name": "ai_generate",
    "functionType": "remote",
    "serviceName": "my_ai_server",
    "endpoint": "/api/functions/generate",
    "version": 1,
    "parameters": ["String"],
    "return": "String"
}
```

**Remote Function Server:**
```rust
// On remote AI server
#[actix_web::post("/api/functions/generate")]
async fn generate_function(req: web::Json<FunctionRequest>) -> impl Responder {
    let prompt = &req.params["prompt"];
    let result = ai_model.generate(prompt).await;
    json!({
        "success": true,
        "result": result
    })
}
```

### Script Function (Quick Iteration)

```json
{
    "name": "simple_transform",
    "functionType": "script",
    "scriptPath": "../dxn_public/scripts/transform.lua",
    "scriptLanguage": "lua",
    "version": 1,
    "parameters": ["String"],
    "return": "String"
}
```

**Lua Script:**
```lua
-- transform.lua
function simple_transform(input)
    -- Simple transformation logic
    return string.upper(input) .. " processed"
end
```

---

## Implementation Phases

### Phase 1: Foundation (Keep WASM, Add Native)
**Goal:** Add native support while maintaining WASM as default

**Tasks:**
1. Add `FunctionType` enum to models
2. Update `SystemFunctionModel` with type-specific fields
3. Implement native library loader (`libloading`)
4. Create native function executor
5. Update function manager to route by type
6. Maintain backward compatibility (default to WASM)

**Deliverables:**
- WASM functions continue to work
- Native functions can be loaded and executed
- Unified API for both types

### Phase 2: Remote Functions
**Goal:** Add remote function execution

**Tasks:**
1. Integrate with service mesh client
2. Implement remote function executor
3. OAuth token management for remote calls
4. HTTP client for function calls
5. Error handling and retries

**Deliverables:**
- Can call functions on remote servers
- OAuth authentication working
- Works with service mesh architecture

### Phase 3: Script Support (Optional)
**Goal:** Add scripting language support

**Tasks:**
1. Choose scripting language (Lua recommended)
2. Embed interpreter (mlua for Lua)
3. Implement script executor
4. Sandboxing for security
5. API bindings for server functions

**Deliverables:**
- Can execute script-based functions
- Secure sandboxed execution
- Access to server APIs from scripts

---

## Native Function Implementation Details

### Dynamic Library Loading

```rust
// In functions/native_executor.rs

use libloading::{Library, Symbol};

pub struct NativeExecutor {
    library: Library,
}

impl NativeExecutor {
    pub fn load(library_path: &str) -> Result<Self, FunctionError> {
        unsafe {
            let library = Library::new(library_path)?;
            Ok(NativeExecutor { library })
        }
    }
    
    pub fn call_function<T, R>(
        &self,
        symbol_name: &str,
        params: T
    ) -> Result<R, FunctionError> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(T) -> R> = 
                self.library.get(symbol_name.as_bytes())?;
            Ok(func(params))
        }
    }
}
```

### Native Function Template

```rust
// Template for native functions
use dxn_core::system::files::manager;
use dxn_core::data::db::sqlite;

#[no_mangle]
pub extern "C" fn my_custom_function(input: String) -> String {
    // Full access to server APIs
    let file = manager::read_file(&input).unwrap();
    
    // Can use any Rust crate
    // Can access database
    // Can call integrations
    
    format!("Processed: {}", file)
}
```

---

## Remote Function Implementation Details

### Remote Function Executor

```rust
// In functions/remote_executor.rs

use crate::integrations::service_mesh;

pub struct RemoteExecutor;

impl RemoteExecutor {
    pub async fn call_function(
        function: &SystemFunctionModel,
        params: &[serde_json::Value]
    ) -> Result<serde_json::Value, FunctionError> {
        // Resolve service from mesh
        let service = service_mesh::get_service(&function.service_name?)?;
        
        // Get OAuth token
        let token = get_oauth_token(&service).await?;
        
        // Make HTTP request
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}{}", service.url, function.endpoint?))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "params": params
            }))
            .send()
            .await?;
        
        let result: serde_json::Value = response.json().await?;
        Ok(result["result"].clone())
    }
}
```

---

## Script Function Implementation Details

### Lua Executor Example

```rust
// In functions/script_executor.rs

use mlua::{Lua, Result as LuaResult};

pub struct ScriptExecutor {
    lua: Lua,
}

impl ScriptExecutor {
    pub fn new() -> Self {
        ScriptExecutor {
            lua: Lua::new()
        }
    }
    
    pub fn load_script(&mut self, script_path: &str) -> LuaResult<()> {
        let script = std::fs::read_to_string(script_path)?;
        self.lua.load(&script).exec()?;
        Ok(())
    }
    
    pub fn call_function(
        &self,
        function_name: &str,
        params: &[serde_json::Value]
    ) -> Result<serde_json::Value, FunctionError> {
        let globals = self.lua.globals();
        let func: mlua::Function = globals.get(function_name)?;
        
        // Convert params to Lua values
        let lua_params: Vec<mlua::Value> = params.iter()
            .map(|v| json_to_lua_value(&self.lua, v))
            .collect();
        
        let result = func.call(lua_params)?;
        Ok(lua_value_to_json(result))
    }
}
```

---

## Migration Path

### Backward Compatibility

**Existing Config (WASM):**
```json
{
    "name": "typed_params",
    "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
    "functionName": "typed_params",
    "version": 1
}
```

**Still Works:** If `functionType` is not specified, defaults to `Wasm`

**New Config (Explicit):**
```json
{
    "name": "typed_params",
    "functionType": "wasm",
    "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
    "functionName": "typed_params",
    "version": 1
}
```

### Gradual Adoption

1. **Phase 1:** All existing functions continue as WASM
2. **Phase 2:** Users can add native functions for customization
3. **Phase 3:** Users can add remote functions for distribution
4. **Phase 4:** Users can add scripts for quick iteration

---

## Use Case Scenarios

### Scenario 1: Shareable Utility Function
**Need:** Function to parse markdown that can be shared across servers

**Solution:** WASM
```json
{
    "name": "parse_markdown",
    "functionType": "wasm",
    "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
    "functionName": "parse_markdown"
}
```

**Why:** Cross-platform, shareable, secure

### Scenario 2: Server-Specific Custom Logic
**Need:** Function that needs direct database access and server internals

**Solution:** Native
```json
{
    "name": "custom_data_processing",
    "functionType": "native",
    "libraryPath": "../dxn_public/functions/target/release/libcustom.so",
    "symbolName": "custom_data_processing"
}
```

**Why:** Full access, better performance, server-specific

### Scenario 3: AI Function on Dedicated Server
**Need:** AI generation function hosted on specialized GPU server

**Solution:** Remote
```json
{
    "name": "ai_generate",
    "functionType": "remote",
    "serviceName": "my_ai_server",
    "endpoint": "/api/functions/generate"
}
```

**Why:** Distributed, specialized resources, scalable

### Scenario 4: Quick Prototype Function
**Need:** Simple transformation function that changes frequently

**Solution:** Script
```json
{
    "name": "quick_transform",
    "functionType": "script",
    "scriptPath": "../dxn_public/scripts/transform.lua",
    "scriptLanguage": "lua"
}
```

**Why:** No compilation, quick iteration, easy to modify

---

## Security Considerations

### WASM Functions
- ✅ Sandboxed execution
- ✅ No direct system access
- ✅ Memory isolation
- ✅ Safe by default

### Native Functions
- ⚠️ Full system access
- ⚠️ Can crash server
- ⚠️ No sandboxing
- ✅ Trust required (user's own code)

### Remote Functions
- ✅ Isolated on remote server
- ✅ OAuth authentication
- ✅ Network isolation
- ⚠️ Trust remote server

### Script Functions
- ⚠️ Requires sandboxing
- ⚠️ Interpreter security
- ✅ Can limit API access
- ⚠️ Runtime errors possible

### Recommendations
- **WASM**: Default for untrusted/shared code
- **Native**: Only for trusted, server-specific code
- **Remote**: Use OAuth and verify service identity
- **Scripts**: Implement sandboxing and API limits

---

## Performance Considerations

### Execution Speed
1. **Native**: Fastest (no overhead)
2. **WASM**: Fast (minimal overhead)
3. **Scripts**: Slower (interpreter overhead)
4. **Remote**: Slowest (network latency)

### Resource Usage
1. **Native**: Low (direct execution)
2. **WASM**: Low (efficient runtime)
3. **Scripts**: Medium (interpreter)
4. **Remote**: Low on primary server (offloaded)

### Recommendations
- **Performance-critical**: Use Native
- **General purpose**: Use WASM
- **Simple logic**: Use Scripts
- **Resource-intensive**: Use Remote

---

## Integration with Service Mesh

### Remote Functions as Services

Remote functions can be discovered via service mesh:

```json
{
    "serviceMesh": {
        "publicServices": [
            {
                "name": "public_ai_functions",
                "discoverFrom": "registry",
                "filter": {
                    "serviceType": "function_server",
                    "capabilities": ["ai", "llm"]
                }
            }
        ]
    },
    "functions": {
        "public": [
            {
                "name": "ai_generate",
                "functionType": "remote",
                "serviceName": "public_ai_functions",
                "endpoint": "/api/functions/generate"
            }
        ]
    }
}
```

### Function Servers

Dedicated servers can host functions:

```
┌─────────────────────────┐
│  Function Server        │
│  - AI Functions         │
│  - Math Functions       │
│  - Image Processing     │
└──────────┬──────────────┘
           │
           │ HTTP/RPC
           │
┌──────────▼──────────────┐
│  Primary Server         │
│  - Calls remote funcs   │
└─────────────────────────┘
```

---

## Developer Experience

### Writing WASM Functions
```rust
// Standard Rust, compile to WASM
#[no_mangle]
pub extern "C" fn my_function(input: String) -> String {
    // Logic here
    format!("Processed: {}", input)
}
```

### Writing Native Functions
```rust
// Standard Rust, compile as dynamic library
use dxn_core::system::files::manager;

#[no_mangle]
pub extern "C" fn my_function(input: String) -> String {
    // Full access to server
    let file = manager::read_file(&input).unwrap();
    format!("Processed: {}", file)
}
```

### Writing Remote Functions
```rust
// On remote server - any language
#[actix_web::post("/api/functions/my_function")]
async fn my_function(req: web::Json<FunctionRequest>) -> impl Responder {
    let result = process(req.params);
    json!({"result": result})
}
```

### Writing Script Functions
```lua
-- Simple Lua script
function my_function(input)
    return "Processed: " .. input
end
```

---

## Recommended Approach Summary

### Hybrid with WASM Default

**Default to WASM for:**
- Shareable functions
- Cross-platform compatibility
- Security isolation
- General-purpose logic

**Use Native for:**
- Performance-critical code
- Server-specific customization
- Deep server integration
- When you need full Rust ecosystem

**Use Remote for:**
- Distributed execution
- Specialized resources
- Scalability
- Integration with service mesh

**Use Scripts for:**
- Quick prototyping
- Simple transformations
- Frequently-modified logic
- Non-critical functions

### Benefits

1. **Versatility**: Right tool for each function
2. **Shareability**: WASM for distribution
3. **Customization**: Native for server-specific code
4. **Distribution**: Remote for specialized servers
5. **Flexibility**: Scripts for quick iteration
6. **Backward Compatible**: Existing code works unchanged

---

## Implementation Priority

**Phase 1 (MVP):**
- Keep WASM as default
- Add native function support
- Maintain backward compatibility

**Phase 2:**
- Add remote function support
- Integrate with service mesh
- OAuth for remote calls

**Phase 3 (Optional):**
- Add scripting support
- Lua or JavaScript runtime
- Sandboxing and security

This hybrid approach provides maximum flexibility while maintaining the benefits of WASM for shareability and security.

---

# Appendix: Implementation Options Analysis

## Current State

### Current Implementation
- Functions are WASM-only
- Compiled to `wasm32-unknown-unknown`
- Executed via Wasmtime runtime
- Stored as `.wasm` files
- Isolated execution environment

### Strengths
- ✅ Cross-platform compatibility
- ✅ Security isolation
- ✅ Shareable binary format
- ✅ Language-agnostic (can compile from Rust, C, etc.)

### Limitations
- ❌ Requires WASM compilation step
- ❌ Limited access to host system
- ❌ Performance overhead vs native
- ❌ Complex parameter passing
- ❌ No direct access to server internals

---

## Option 1: WASM Only (Current)

### How It Works
- All functions must be compiled to WASM
- Single execution model
- All functions share the same runtime characteristics

### Pros
- ✅ Simple architecture - one execution model
- ✅ Consistent execution environment
- ✅ Security isolation - sandboxed execution
- ✅ Cross-platform - same binary works everywhere
- ✅ Shareable - easy to distribute `.wasm` files
- ✅ Language-agnostic - can compile from multiple languages

### Cons
- ❌ Requires compilation for all functions
- ❌ Performance overhead compared to native
- ❌ Limited customization options
- ❌ Can't access server internals directly
- ❌ Harder to debug WASM code
- ❌ Complex parameter passing (type conversion needed)

### Use Case
**Best for:** Shareable, isolated functions that need to work across platforms

**Example:**
```json
{
    "name": "parse_markdown",
    "functionType": "wasm",
    "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
    "functionName": "parse_markdown"
}
```

---

## Option 2: Native Rust Functions (Dynamic Libraries)

### How It Works
- Functions compiled as Rust dynamic libraries (`.so`/`.dylib`/`.dll`)
- Loaded at runtime via `dlopen`/`libloading`
- Direct access to server APIs and internals
- Full Rust ecosystem available

### Pros
- ✅ **Best Performance** - Native speed, no WASM overhead
- ✅ **Full Access** - Can use entire Rust ecosystem
- ✅ **Server Integration** - Direct access to server internals
- ✅ **Easier Debugging** - Standard Rust debugging tools
- ✅ **No WASM Compilation** - Standard Rust build process
- ✅ **Type Safety** - Full Rust type system

### Cons
- ❌ **Platform-Specific** - Different binaries per platform
- ❌ **Security Concerns** - Full system access, no sandboxing
- ❌ **Less Shareable** - Need platform-specific builds
- ❌ **Recompilation Required** - Per platform/architecture
- ❌ **Potential Crashes** - Can crash entire server
- ❌ **Dependency Management** - Must match server's dependencies

### Use Case
**Best for:** Performance-critical, server-specific functions that need deep integration

**Example:**
```json
{
    "name": "custom_processing",
    "functionType": "native",
    "libraryPath": "../dxn_public/functions/target/release/libcustom_functions.so",
    "symbolName": "custom_processing"
}
```

**Implementation:**
```rust
// In native function library
use dxn_core::system::files::manager;

#[no_mangle]
pub extern "C" fn custom_processing(input: String) -> String {
    // Direct access to server APIs
    let file = manager::read_file(&input).unwrap();
    // Custom processing with full Rust ecosystem
    process_custom_logic(file)
}
```

---

## Option 3: Remote Functions (HTTP/RPC)

### How It Works
- Functions hosted on remote servers
- Called via HTTP/RPC
- Similar to service mesh pattern for integrations
- Can be any language/technology

### Pros
- ✅ **Distributed Execution** - Functions on separate servers
- ✅ **Specialized Servers** - Dedicated function servers
- ✅ **Independent Scaling** - Scale functions separately
- ✅ **Language-Agnostic** - Any server can host functions
- ✅ **Works with Service Mesh** - Integrates with integration architecture
- ✅ **Resource Isolation** - Functions don't consume primary server resources

### Cons
- ❌ **Network Latency** - HTTP overhead
- ❌ **Network Dependency** - Requires connectivity
- ❌ **Complex Error Handling** - Network failures, timeouts
- ❌ **OAuth/Security Needed** - Authentication required
- ❌ **Potential Failures** - Network issues affect availability

### Use Case
**Best for:** Distributed, specialized function servers, or functions that need specific resources

**Example:**
```json
{
    "name": "ai_generate",
    "functionType": "remote",
    "serviceName": "my_ai_server",
    "endpoint": "/api/functions/generate",
    "parameters": ["String"],
    "return": "String"
}
```

**Flow:**
1. Function call initiated
2. HTTP request to remote server
3. Remote server executes function
4. Response returned via HTTP
5. Result passed back to caller

---

## Option 4: Scripting Languages (Lua, JavaScript, Python)

### How It Works
- Embed scripting language runtime (e.g., mlua for Lua, v8 for JavaScript)
- Functions written in scripts
- Executed by embedded interpreter
- Stored as text files

### Pros
- ✅ **No Compilation** - Write and run immediately
- ✅ **Fast Iteration** - Quick to modify and test
- ✅ **Easy to Write** - Simpler syntax for simple logic
- ✅ **Text-Based** - Easy to version control
- ✅ **Good for Prototyping** - Quick to experiment

### Cons
- ❌ **Performance Overhead** - Interpreter overhead
- ❌ **Security Concerns** - Requires sandboxing
- ❌ **Additional Dependencies** - Need to embed runtime
- ❌ **Limited Type Safety** - Runtime errors
- ❌ **Debugging Challenges** - Script debugging can be harder
- ❌ **Limited Ecosystem** - Can't use full Rust ecosystem

### Use Case
**Best for:** Simple, frequently-modified functions, quick prototyping

**Example:**
```json
{
    "name": "simple_transform",
    "functionType": "script",
    "scriptPath": "../dxn_public/scripts/transform.lua",
    "scriptLanguage": "lua",
    "parameters": ["String"],
    "return": "String"
}
```

**Lua Script Example:**
```lua
-- transform.lua
function simple_transform(input)
    return string.upper(input)
end
```

---

## Option 5: Hybrid Multi-Format (Recommended)

### How It Works
- Support multiple function types simultaneously
- WASM (default for shareability)
- Native Rust (for performance/customization)
- Remote (for distributed execution)
- Scripts (optional, for quick iteration)

### Architecture

```
Function Manager
    ├── WASM Executor (current)
    ├── Native Executor (new)
    ├── Remote Executor (new)
    └── Script Executor (optional)
```

### Pros
- ✅ **Maximum Flexibility** - Choose the right tool for each function
- ✅ **Backward Compatible** - Existing WASM functions continue to work
- ✅ **Performance When Needed** - Native for critical paths
- ✅ **Shareability When Needed** - WASM for distribution
- ✅ **Customization When Needed** - Native for server-specific logic
- ✅ **Distributed When Needed** - Remote for specialized servers
- ✅ **Quick Iteration** - Scripts for rapid development

### Cons
- ❌ **More Complex Implementation** - Multiple execution paths
- ❌ **Maintenance Overhead** - Need to maintain multiple executors
- ❌ **Decision Making** - Users need to choose appropriate type

### Use Case
**Best for:** Versatile system that supports all use cases

---

## Detailed Comparison Matrix

| Feature | WASM | Native | Remote | Script | Hybrid |
|---------|------|--------|--------|--------|--------|
| **Shareability** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Performance** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Security** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Customization** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Ease of Use** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Cross-Platform** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Debugging** | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Setup Complexity** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| **Distribution** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
