
use wasmtime::*;
use serde::Deserialize;
use std::fs;
use anyhow::Result;
use std::sync::{LazyLock, RwLock, Mutex};
use std::vec::Vec;
use crate::functions::models::{EnumParamTypes};
use crate::system::models::{SystemError};
 

use crate::functions::models::SystemFunctionModel;
#[derive(Debug)]
struct WasmModuleInstance {
    name: String,
    function_name: String,
    instance: Instance,
    func: Func,
    module: Module,
    bytes: Vec<u8> 
}

struct WasmModules {
    module: Option<Module>
}

/*
   let engine = Engine::default();
    let mut store = Store::new(&engine, ());
   
        // 3. Load the Wasm module bytes
        let module_bytes = fs::read(&module_config.path)?;
        let module = Module::from_binary(&engine, &module_bytes)?;

        // 4. Define host functions (if any are needed by the Wasm module)
        // In this example, we assume no imports are needed. For host calls
        // you would use Linker and define host functions here.
        let linker = Linker::new(&engine);
        // linker.func_wrap("host_env", "log_str", |caller: Caller<'_, ()>, ptr: i32, len: i32| { /* ... */ })?;

        // 5. Instantiate the module
        let instance: Instance = linker.instantiate(&mut store, &module)?;
        
         */

fn get_engine() -> Engine { Engine::default() }
static mut ENGINE: LazyLock<Engine> = LazyLock::new(get_engine);

/*      
fn get_linker() -> Linker<()> { 
    unsafe {
        Linker::new(&ENGINE) 
    }
}
static mut LINKER: LazyLock<Linker<()>> = LazyLock::new(get_linker);


fn get_instance() -> Module { LINKER.instantiate(&mut STORE, module) Module::new() }
static mut ENGINE: LazyLock<Engine> = LazyLock::new(get_engine);

*/

static mut PUBLIC_FUNCTIONS: Vec<WasmModuleInstance> = Vec::new();


pub fn init(functions: Vec<SystemFunctionModel>) -> Result<()> {
    // 1. Set up the Wasmtime environment
    //let engine = Engine::default();
    unsafe { 

    for module_config in functions {
        let mut store: Store<HostState> = Store::new(&ENGINE, HostState { call_count: 0 });
   
        println!("Loading module: {} -> {}, {}", module_config.name, module_config.function_name, &module_config.path);

        // 3. Load the Wasm module bytes
        let module_bytes = fs::read(&module_config.path)?;
        let module = Module::from_binary(&ENGINE, &module_bytes)?;

        // 4. Define host functions (if any are needed by the Wasm module)
        // In this example, we assume no imports are needed. For host calls
        // you would use Linker and define host functions here.
        let linker = Linker::new(&ENGINE);
        // linker.func_wrap("host_env", "log_str", |caller: Caller<'_, ()>, ptr: i32, len: i32| { /* ... */ })?;

        // 5. Instantiate the module
        let instance: Instance = linker.instantiate(&mut store, &module)?;
        
        let func = instance.get_func(&mut store, &module_config.function_name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", module_config.function_name))?;

        PUBLIC_FUNCTIONS.push(WasmModuleInstance { 
            name: module_config.name, 
            function_name: module_config.function_name, 
            instance, 
            func: func,
            module: module,
            bytes: module_bytes
        }); 
    }
    
    }
    Ok(())
}

pub fn get_function(name: &str) -> Option<&WasmModuleInstance> {
    unsafe {
        PUBLIC_FUNCTIONS
            .iter()
            .to_owned()
            .filter(|item| {
                item.name == name
            })
            .last()
    }
}

/// params: P = tuple = (i32, i32)
pub fn run<P, R>(name: &str, params: P) -> anyhow::Result<R> 
where 
    P: WasmParams,
    R: WasmResults
{
    // 1. Set up the Wasmtime environment
    let engine = Engine::default();
    unsafe {
        let func = get_function(name);
        match func {
            Some(result) => {
                let mut store: Store<HostState> = Store::new(&engine, HostState { call_count: 0 });
                let mut linker: Linker<HostState> = Linker::new(&engine);

                //let mut store: Store<()> = Store::new(unsafe { &ENGINE }, ());
                let module = Module::from_binary(&engine, &result.bytes)?;

                // 6. Get and call the target function
                let instance: Instance = linker.instantiate(&mut store, &module)?;
                let function: Func = instance.get_func(&mut store, &result.function_name)
                    .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", result.function_name))?;
                
                println!("functions::manager call_func {}", result.function_name);
             
                let call_func: TypedFunc<P, R> = function.typed::<P, R>(&mut store)?;

                println!("Calling function: {}...", result.function_name);

                let results = call_func.call(&mut store, params)?;

                return Ok(results)
            },
            None => {
                
            }
        }
        //Ok()W
        Err(anyhow::Error::msg("No matching function found"))
    }
}
 


 use std::sync::Arc;

// Define the host state that will be stored in the Store<T>
struct HostState {
    // any mutable host data needed during wasm execution
    call_count: u32,
}
/* 
// Function to initialize the engine and compile the module
fn initialize_wasm() -> Result<(Engine, Module)> {
    let engine = Engine::default(); // An Engine can be shared
    // A simple WASM module in WebAssembly Text format (WAT)
    let wat = r#"
        (module
            (func $hello (import "" "hello"))
            (func (export "run")
                (call $hello)
            )
        )
    "#;
    let module = Module::new(&engine, wat)?; // Modules are thread-safe and can be shared

    Ok((engine, module))
}

// Function to run the WASM function
fn run_wasm_function(engine: &Engine, module: &Module) -> Result<()> {
    // Create a new store for each execution context/instance
    let mut store = Store::new(engine, HostState { call_count: 0 });
    let mut linker = Linker::new(engine);

    // Define a host function and add it to the linker
    linker.func_wrap("", "hello", move |mut caller: Caller<'_, HostState>| {
        // Access and modify the host state within the store
        caller.data_mut().call_count += 1;
        println!("> Hello from WASM! Call count: {}", caller.data().call_count);
    })?;

    // Instantiate the module in the store
    let instance = linker.instantiate(&mut store, module)?;

    // Get the exported "run" function
    let run = instance.get_typed_func::<(), _>(&mut store, "run")?;

    // Call the WASM function
    println!("Calling wasm function 'run'...");
    run.call(&mut store, ())?;

    Ok(())
}

fn example() -> Result<()> {
    // 1. Initialize engine and module once
    let (engine, module) = initialize_wasm()?;

    // 2. Use the engine and module in different functions as needed
    // The Engine and Module are cheap to clone/share if needed across threads
    run_wasm_function(&engine, &module)?;
    run_wasm_function(&engine, &module)?; // Can run again, each with its own Store

    Ok(())
}
    */