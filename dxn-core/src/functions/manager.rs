use wasmtime::*;
use std::fs;
use anyhow::Result;
use std::sync::{LazyLock, Mutex};
use crate::functions::models::{SystemFunctionModel, FunctionType, FunctionError};
use crate::functions::executors::*;
use crate::system::files::dxn_files;

// Store all registered functions
static mut PUBLIC_FUNCTIONS: Mutex<Vec<SystemFunctionModel>> = Mutex::new(Vec::new());

fn get_engine() -> Engine { Engine::default() }
static mut ENGINE: LazyLock<Engine> = LazyLock::new(get_engine);

struct HostState {
    call_count: u32,
}

/// Initialize all functions from config
pub fn init(functions: Vec<SystemFunctionModel>) -> Result<()> {
    unsafe {
        let mut funcs = PUBLIC_FUNCTIONS.lock().unwrap();
        println!("[DEBUG] Initializing {} functions:", functions.len());
        for func in &functions {
            println!("[DEBUG]   - Registered function: '{}' (type: {:?})", func.name, func.function_type);
        }
        *funcs = functions;
    }
    Ok(())
}

/// Get a function by name
pub fn get_function(name: &str) -> Option<SystemFunctionModel> {
    unsafe {
        let funcs = PUBLIC_FUNCTIONS.lock().unwrap();
        println!("[DEBUG] Looking for function: '{}'", name);
        println!("[DEBUG] Available functions: {:?}", funcs.iter().map(|f| &f.name).collect::<Vec<_>>());
        funcs.iter().find(|f| f.name == name).cloned()
    }
}

/// Resolve a path from config under project_root/dxn-files when project_root is set.
fn resolve_path(project_root: Option<&str>, config_path: Option<&String>) -> Option<String> {
    let root = project_root?;
    let path = config_path?;
    dxn_files::resolve_under_dxn_files(root, path)
        .ok()
        .map(|pb| pb.to_string_lossy().into_owned())
}

/// Unified function call API - routes to appropriate executor based on function type.
/// When project_root is Some, path/script_path/library_path are resolved under project_root/dxn-files.
pub async fn call_function(
    name: &str,
    params: &[serde_json::Value],
    project_root: Option<&str>,
) -> Result<serde_json::Value, FunctionError> {
    let function = get_function(name)
        .ok_or_else(|| FunctionError::NotFound(name.to_string()))?;

    let path_override = resolve_path(project_root, function.path.as_ref());
    let script_path_override = resolve_path(project_root, function.script_path.as_ref());
    let library_path_override = resolve_path(project_root, function.library_path.as_ref());

    match function.function_type {
        FunctionType::Wasm => {
            execute_wasm(&function, params, path_override.as_deref()).await
        }
        FunctionType::Native => {
            execute_native(&function, params, library_path_override.as_deref()).await
        }
        FunctionType::Remote => execute_remote(&function, params).await,
        FunctionType::Script => {
            execute_script(&function, params, script_path_override.as_deref()).await
        }
    }
}
/*
/// Legacy run function for backward compatibility (WASM only)

pub fn run<P, R>(name: &str, params: P) -> anyhow::Result<R> 
where 
    P: WasmParams,
    R: WasmResults
{
    let function = get_function(name)
        .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", name))?;
    
    if function.function_type != FunctionType::Wasm {
        return Err(anyhow::anyhow!("Function '{}' is not a WASM function", name));
    }
    
    // Convert params to serde_json::Value for unified API
    let json_params = vec![serde_json::to_value(params)?];
    
    // Call WASM executor (synchronous for legacy API)
    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(execute_wasm(&function, &json_params))?;
    
    // Convert back to expected type
    serde_json::from_value(result)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize result: {}", e))
}
*/

/// Legacy run function for backward compatibility (WASM only)
/// Note: This function uses the old TypedFunc approach which doesn't support String types
/// For new code, use call_function() instead
pub fn run<P, R>(name: &str, params: P) -> anyhow::Result<R> 
where 
    P: WasmParams,
    R: WasmResults
{
    let function = get_function(name)
        .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", name))?;
    
    if function.function_type != FunctionType::Wasm {
        return Err(anyhow::anyhow!("Function '{}' is not a WASM function", name));
    }
    
    // Get path and function_name from the function model
    let path = function.path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("WASM path not specified for function '{}'", name))?;
    // Use function_name if provided, otherwise default to name
    let function_name = function.function_name.as_ref()
        .unwrap_or(&function.name);
    
    // Set up the Wasmtime environment
    let engine = Engine::default();
    unsafe {
        let mut store: Store<HostState> = Store::new(&engine, HostState { call_count: 0 });
        let mut linker: Linker<HostState> = Linker::new(&engine);

        // Load WASM module from file
        let module_bytes = fs::read(path)?;
        let module = Module::from_binary(&engine, &module_bytes)?;

        // Instantiate and get function
        let instance: Instance = linker.instantiate(&mut store, &module)?;
        let wasm_func: Func = instance.get_func(&mut store, function_name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found in WASM module", function_name))?;
        
        println!("functions::manager call_func {}", function_name);
     
        let call_func: TypedFunc<P, R> = wasm_func.typed::<P, R>(&mut store)?;

        println!("Calling function: {}...", function_name);

        let results = call_func.call(&mut store, params)?;

        Ok(results)
    }
}