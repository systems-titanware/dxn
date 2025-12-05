use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

// Define the functions you want to call.
fn function_a(data: &str) {
    println!("Running Function A with data: {}", data);
}

fn function_b(data: &str) {
    println!("Running Function B with data: {}", data);
}

fn init() {
    // 1. Define your function map (whitelist)
    let mut function_map: HashMap<&str, FunctionPointer> = HashMap::new();
    function_map.insert("func_a", function_a);
    function_map.insert("func_b", function_b);
    
    // 2. Sample JSON string
    let json_input = r#"{
        "action": "func_a",
        "payload": "some important data"
    }"#;

    // 3. Parse the JSON using serde_json
    let parsed_json: Value = serde_json::from_str(json_input)?;

    // 4. Extract the function name (action) and payload from the JSON
    if let (Some(action_name), Some(payload_value)) = (
        parsed_json["action"].as_str(),
        parsed_json["payload"].as_str(),
    ) {
        // 5. Look up the function in the map and execute it
        if let Some(function_to_run) = function_map.get(action_name) {
            println!("Lookup successful. Executing '{}'...", action_name);
            function_to_run(payload_value);
        } else {
            eprintln!("Error: Function '{}' not found in the whitelist!", action_name);
        }
    } else {
        eprintln!("Error: Invalid JSON format or missing fields.");
    }

    Ok(())
}
