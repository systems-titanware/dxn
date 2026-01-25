// Function executors for different function types

use crate::functions::models::{SystemFunctionModel, FunctionError};
use wasmtime::*;
use std::fs;
use std::sync::LazyLock;
use libloading::{Library, Symbol};
use serde_json;

fn get_engine() -> Engine { Engine::default() }
static mut ENGINE: LazyLock<Engine> = LazyLock::new(get_engine);

struct HostState {
    call_count: u32,
}

// WASM Executor - Manual memory management with packed i64 return values
pub async fn execute_wasm(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    use wasmtime::Val;
    
    let path = function.path.as_ref()
        .ok_or_else(|| FunctionError::InvalidInput("WASM path not specified".to_string()))?;
    
    // Use function_name if provided, otherwise default to name
    let function_name = function.function_name.as_ref()
        .unwrap_or(&function.name);
    
    println!("[DEBUG] Attempting to read WASM file from: {}", path);
    
    unsafe {
        let engine = &ENGINE;
        let mut store: Store<HostState> = Store::new(engine, HostState { call_count: 0 });
        
        // Load WASM module
        let module_bytes = fs::read(path)
            .map_err(|e| FunctionError::IoError(e))?;
        let module = Module::from_binary(engine, &module_bytes)
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to load WASM module: {:?}", e)))?;
        
        // Create linker and instantiate
        let mut linker = Linker::new(engine);
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to instantiate WASM module: {:?}", e)))?;
        
        // Get function
        let wasm_func = instance.get_func(&mut store, function_name)
            .ok_or_else(|| FunctionError::NotFound(format!("Function '{}' not found in WASM module", function_name)))?;
        
        // Serialize params as JSON string
        let json_params = serde_json::to_string(params)
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to serialize params: {}", e)))?;
        
        // Get memory from instance (WASM module must export 'memory')
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| FunctionError::ExecutionError("WASM module doesn't export 'memory'".to_string()))?;
        
        // Write input JSON string to WASM memory
        // Using fixed offset 1024 for input (WASM functions read from this location)
        let input_bytes = json_params.as_bytes();
        let input_len = input_bytes.len();
        let input_ptr = 1024;
        memory.write(&mut store, input_ptr, input_bytes)
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to write input to WASM memory: {:?}", e)))?;
        
        // WASM functions return (ptr, len) packed as i64
        // Lower 32 bits = ptr, Upper 32 bits = len
        let mut results = [Val::I64(0)];
        
        // Call function
        wasm_func.call(
            &mut store,
            &[Val::I32(input_ptr as i32), Val::I32(input_len as i32)],
            &mut results
        ).map_err(|e| FunctionError::ExecutionError(format!("WASM call error: {:?}", e)))?;
        
        // Extract result pointer and length from packed i64
        let (result_ptr, result_len) = match results[0] {
            Val::I64(packed) => {
                let ptr = (packed & 0xFFFFFFFF) as usize;  // Lower 32 bits
                let len = ((packed >> 32) & 0xFFFFFFFF) as usize;  // Upper 32 bits
                (ptr, len)
            },
            _ => return Err(FunctionError::ExecutionError("Unexpected return type from WASM function (expected i64)".to_string())),
        };
        
        // Read result string from WASM memory
        // memory.read() writes to a buffer, so we need to create a buffer first
        let mut result_bytes = vec![0u8; result_len];
        memory.read(&store, result_ptr, result_bytes.as_mut_slice())
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to read result from WASM memory: {:?}", e)))?;
        
        let result_str = String::from_utf8(result_bytes)
            .map_err(|e| FunctionError::ExecutionError(format!("Invalid UTF-8 in result: {:?}", e)))?;
        
        // Try to parse result as JSON, otherwise return as string
        match serde_json::from_str::<serde_json::Value>(&result_str) {
            Ok(json) => Ok(json),
            Err(_) => Ok(serde_json::json!(result_str)),
        }
    }
}

// Native Executor
pub async fn execute_native(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    let library_path = function.library_path.as_ref()
        .ok_or_else(|| FunctionError::InvalidInput("Library path not specified".to_string()))?;
    
    // Use symbol_name if provided, otherwise default to name
    let symbol_name = function.symbol_name.as_ref()
        .unwrap_or(&function.name);
    
    unsafe {
        // Load dynamic library
        let library = Library::new(library_path)
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to load library: {}", e)))?;
        
        // For string-based functions (most common)
        // In a full implementation, you'd need type information to call correctly
        let json_params = serde_json::to_string(params)
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to serialize params: {}", e)))?;
        
        // Call function with String -> String signature
        let func: Symbol<unsafe extern "C" fn(String) -> String> = library.get(symbol_name.as_bytes())
            .map_err(|e| FunctionError::ExecutionError(format!("Failed to get symbol '{}': {}", symbol_name, e)))?;
        
        let result = func(json_params);
        
        // Try to parse result as JSON, otherwise return as string
        match serde_json::from_str::<serde_json::Value>(&result) {
            Ok(json) => Ok(json),
            Err(_) => Ok(serde_json::json!(result)),
        }
    }
}

// Remote Executor
pub async fn execute_remote(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    let service_name = function.service_name.as_ref()
        .ok_or_else(|| FunctionError::InvalidInput("Service name not specified".to_string()))?;
    
    let endpoint = function.endpoint.as_ref()
        .ok_or_else(|| FunctionError::InvalidInput("Endpoint not specified".to_string()))?;
    
    // TODO: Resolve service URL from service mesh
    // For now, use a simple mapping based on service name
    let service_url = match service_name.as_str() {
        "my_ai" => "http://127.0.0.1:8081",
        _ => return Err(FunctionError::NotFound(format!("Service '{}' not found", service_name))),
    };
    
    let client = reqwest::Client::new();
    let url = format!("{}{}", service_url, endpoint);
    
    let request_body = serde_json::json!({
        "params": params
    });
    
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| FunctionError::ExecutionError(format!("HTTP request failed: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(FunctionError::ExecutionError(format!(
            "Remote function returned error: {}", response.status()
        )));
    }
    
    let result: serde_json::Value = response.json().await
        .map_err(|e| FunctionError::ExecutionError(format!("Failed to parse response: {}", e)))?;
    
    // Extract result from FunctionResponse format
    if let Some(result_value) = result.get("result") {
        Ok(result_value.clone())
    } else {
        Ok(result)
    }
}

// Script Executor (JavaScript/TypeScript)
pub async fn execute_script(
    function: &SystemFunctionModel,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    let script_path = function.script_path.as_ref()
        .ok_or_else(|| FunctionError::InvalidInput("Script path not specified".to_string()))?;
    
    // Use function_name if provided, otherwise default to name
    let function_name = function.function_name.as_ref()
        .unwrap_or(&function.name);
    
    let script_language = function.script_language.as_ref()
        .unwrap_or(&"javascript".to_string());
    
    #[cfg(feature = "script-support")]
    {
        match script_language.as_str() {
            "javascript" | "typescript" | "js" | "ts" => {
                execute_javascript_script(script_path, function_name, params).await
            },
            _ => Err(FunctionError::ExecutionError(format!(
                "Unsupported script language: {}. Supported: javascript, typescript", script_language
            ))),
        }
    }
    
    #[cfg(not(feature = "script-support"))]
    {
        Err(FunctionError::ExecutionError(
            "Script support is not enabled. Build with --features script-support".to_string()
        ))
    }
}

#[cfg(feature = "script-support")]
async fn execute_javascript_script(
    script_path: &str,
    function_name: &str,
    params: &[serde_json::Value]
) -> Result<serde_json::Value, FunctionError> {
    use rquickjs::{Context, Runtime};
    
    let script_content = fs::read_to_string(script_path)
        .map_err(|e| FunctionError::IoError(e))?;
    
    // Transpile TypeScript to JavaScript if needed
    let js_code = if script_path.ends_with(".ts") || script_path.ends_with(".tsx") {
        transpile_typescript(&script_content)?
    } else {
        script_content
    };
    
    // Create QuickJS runtime
    let rt = Runtime::new()
        .map_err(|e| FunctionError::ExecutionError(format!("Failed to create JS runtime: {:?}", e)))?;
    
    let ctx = Context::full(&rt)
        .map_err(|e| FunctionError::ExecutionError(format!("Failed to create JS context: {:?}", e)))?;
    
    // Execute script to load functions
    ctx.with(|ctx| {
        ctx.eval::<_, ()>(&js_code)
            .map_err(|e| FunctionError::ExecutionError(format!("JS script error: {:?}", e)))?;
        
        // Try to get function from exports first (ES modules), then global scope
        let func = if let Ok(exports) = ctx.global().get::<_, rquickjs::Object>("exports") {
            // ES module style: exports.functionName
            exports.get::<_, rquickjs::Function>(function_name)
                .or_else(|_| {
                    // Try default export
                    exports.get::<_, rquickjs::Function>("default")
                })
        } else {
            // CommonJS or global function
            ctx.global().get::<_, rquickjs::Function>(function_name)
        }
        .map_err(|e| FunctionError::ExecutionError(format!(
            "Function '{}' not found in script: {:?}", function_name, e
        )))?;
        
        // Convert params to JS values
        let js_params: Vec<rquickjs::Value> = params.iter()
            .map(|v| json_to_js_value(ctx, v))
            .collect();
        
        // Call function - rquickjs Function::call expects a tuple
        // Handle different argument counts
        let result = match js_params.len() {
            0 => func.call(()),
            1 => func.call((js_params[0].clone(),)),
            2 => func.call((js_params[0].clone(), js_params[1].clone())),
            3 => func.call((js_params[0].clone(), js_params[1].clone(), js_params[2].clone())),
            _ => {
                // For many params, create an array and use apply
                let args_array = ctx.new_array().unwrap();
                for (i, param) in js_params.iter().enumerate() {
                    args_array.set(i as u32, param.clone()).unwrap();
                }
                // Use Function.apply with array
                func.call((args_array,))
            }
        }
        .map_err(|e| FunctionError::ExecutionError(format!("JS execution error: {:?}", e)))?;
        
        // Convert result back to JSON
        js_value_to_json(result)
    })
}

#[cfg(feature = "script-support")]
fn transpile_typescript(ts_code: &str) -> Result<String, FunctionError> {
    use swc_common::{SourceMap, FileName, DUMMY_SP};
    use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
    use swc_ecma_transforms::typescript;
    use swc_ecma_codegen::{text_writer::JsWriter, Emitter, Config};
    
    let cm = SourceMap::default();
    let fm = cm.new_source_file(FileName::Anon, ts_code.into());
    let syntax = Syntax::Typescript(TsConfig {
        tsx: false,
        decorators: false,
        ..Default::default()
    });
    
    let mut parser = Parser::new_from(StringInput::from(&*fm), syntax);
    let module = parser.parse_module()
        .map_err(|e| FunctionError::ExecutionError(format!("TypeScript parse error: {:?}", e)))?;
    
    // Strip TypeScript types
    let module = typescript::strip(DUMMY_SP, module);
    
    // Generate JavaScript
    let mut buf = Vec::new();
    let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
    let mut emitter = Emitter {
        cfg: Config {
            minify: false,
            ..Default::default()
        },
        cm,
        wr: writer,
    };
    
    emitter.emit_module(&module)
        .map_err(|e| FunctionError::ExecutionError(format!("TypeScript emit error: {:?}", e)))?;
    
    String::from_utf8(buf)
        .map_err(|e| FunctionError::ExecutionError(format!("Invalid UTF-8: {}", e)))
}

#[cfg(feature = "script-support")]
fn json_to_js_value(ctx: &rquickjs::Context, value: &serde_json::Value) -> rquickjs::Value {
    match value {
        serde_json::Value::Null => rquickjs::Value::Null,
        serde_json::Value::Bool(b) => rquickjs::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rquickjs::Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                rquickjs::Value::Float(f)
            } else {
                rquickjs::Value::Null
            }
        },
        serde_json::Value::String(s) => {
            rquickjs::Value::String(ctx.new_string(s).unwrap())
        },
        serde_json::Value::Array(arr) => {
            let array = ctx.new_array().unwrap();
            for (i, item) in arr.iter().enumerate() {
                array.set(i as u32, json_to_js_value(ctx, item)).unwrap();
            }
            rquickjs::Value::Array(array)
        },
        serde_json::Value::Object(obj) => {
            let object = ctx.new_object().unwrap();
            for (k, v) in obj.iter() {
                object.set(k, json_to_js_value(ctx, v)).unwrap();
            }
            rquickjs::Value::Object(object)
        },
    }
}

#[cfg(feature = "script-support")]
fn js_value_to_json(value: rquickjs::Value) -> Result<serde_json::Value, FunctionError> {
    match value {
        rquickjs::Value::Null | rquickjs::Value::Undefined => Ok(serde_json::Value::Null),
        rquickjs::Value::Bool(b) => Ok(serde_json::Value::Bool(b)),
        rquickjs::Value::Int(i) => Ok(serde_json::Value::Number(i.into())),
        rquickjs::Value::Float(f) => {
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .ok_or_else(|| FunctionError::ExecutionError("Invalid number".to_string()))
        },
        rquickjs::Value::String(s) => {
            Ok(serde_json::Value::String(s.to_string().unwrap()))
        },
        rquickjs::Value::Array(a) => {
            let mut arr = Vec::new();
            let len = a.len().unwrap_or(0);
            for i in 0..len {
                if let Ok(val) = a.get::<_, rquickjs::Value>(i) {
                    arr.push(js_value_to_json(val)?);
                }
            }
            Ok(serde_json::Value::Array(arr))
        },
        rquickjs::Value::Object(o) => {
            let mut map = serde_json::Map::new();
            let keys = o.keys().unwrap_or_default();
            for key in keys {
                let key_str = key.to_string().unwrap();
                if let Ok(val) = o.get::<_, rquickjs::Value>(&key) {
                    map.insert(key_str, js_value_to_json(val)?);
                }
            }
            Ok(serde_json::Value::Object(map))
        },
        _ => {
            // Try to convert to string as fallback
            if let Ok(s) = value.to_string() {
                Ok(serde_json::Value::String(s))
            } else {
                Err(FunctionError::ExecutionError("Unsupported JS value type".to_string()))
            }
        },
    }
}

