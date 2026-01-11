
use wasmtime::*;
use serde::Deserialize;
use std::fs;
use anyhow::Result;
use std::vec::Vec;

use crate::functions::models::SystemFunctionModel;

// Define the structure of the JSON configuration
#[derive(Deserialize, Debug)]
struct WasmModuleConfig {
    name: String,
    path: String,
    function_name: String,
}


#[derive(Deserialize, Debug)]
struct Config {
    modules: Vec<WasmModuleConfig>,
}

pub fn run(functions: Vec<SystemFunctionModel>) -> Result<()> {
    // 1. Read and parse the JSON configuration
   // let config_data = fs::read_to_string("config.json")?;
   // let config: Config = serde_json::from_str(&config_data)?;

    println!("FUNCTIONS: RUN");
    // 2. Set up the Wasmtime environment
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    
    println!("LOADING WASM MOD: {:?}", functions);

    for module_config in functions {
        println!("Loading module: {}", module_config.name);

        // 3. Load the Wasm module bytes
        let module_bytes = fs::read(&module_config.path)?;
        let module = Module::from_binary(&engine, &module_bytes)?;
        println!("Loading module2: {}", module_config.name);

        // 4. Define host functions (if any are needed by the Wasm module)
        // In this example, we assume no imports are needed. For host calls
        // you would use Linker and define host functions here.
        let linker = Linker::new(&engine);
        // linker.func_wrap("host_env", "log_str", |caller: Caller<'_, ()>, ptr: i32, len: i32| { /* ... */ })?;

        // 5. Instantiate the module
        let instance = linker.instantiate(&mut store, &module)?;

        // 6. Get and call the target function
        let func = instance.get_func(&mut store, &module_config.function_name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", module_config.function_name))?;

        // Assume the function takes no arguments and returns nothing for simplicity (fn() -> ())
        // The actual signature should match your Wasm module's function.
        let call_func = func.typed::<(), ()>(&mut store)?;
        
        /*
        
        // E. Get the exported function and call it
        // We specify the signature (i32, i32) -> i32
        let add_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add")?;

        let arg1 = 10;
        let arg2 = 20;
        let result = add_func.call(&mut store, (arg1, arg2))?;
        */


        println!("Calling function: {}...", module_config.function_name);
        call_func.call(&mut store, ())?;
        println!("Function called successfully.\n");
    }

    Ok(())
}
 