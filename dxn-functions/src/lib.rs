use serde::Deserialize;
//use dxn_core::integrations::{manager};
//use dxn_core::system::files::{manager};

#[derive(Deserialize)]
struct Config {
    multiplier: u32,
    offset: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn no_params() {
    let result = (10 * 1) + 5;
    println!("COMPUTED RESULT {}", result);
    // Your Rust logic here
    // In a real scenario you would interact with memory to pass complex data
}

#[unsafe(no_mangle)]
pub extern "C" fn no_params_with_result() -> i32 {
    let result = (10 * 1) + 5;
    println!("COMPUTED RESULT {}", result);
    result
    // Your Rust logic here
    // In a real scenario you would interact with memory to pass complex data
}
 
#[unsafe(no_mangle)]
pub extern "C" fn serialized_params(json_data: String) -> Result<String, std::io::Error> {
    // Deserialize the input JSON string
    let config: Config = serde_json::from_str(&json_data)?;

    // Perform some logic (replace with your actual library logic)
    let result = (10 * config.multiplier) + config.offset;

    // Return a result
    Ok(format!("Computed result: {}", result))
}

#[unsafe(no_mangle)]
pub extern "C" fn typed_params(left: i32, right: i32) -> i32 {
    left + right
}



#[unsafe(no_mangle)]
pub extern "C" fn parse_markdown(path: String) -> String {
    println!("parse_markdown {}", path);
    /*
    // Placeholder implementation for parsing markdown
    let file = crate::system::files::manager::read_file(&path).unwrap_or(String::from("err"));
    
    // Process Markdown to html
    let processed_markdown: std::result::Result<String, integrations::models::IntegrationError> = integrations::manager::run("parser", "parse", Some(&file));
    processed_markdown.unwrap_or(String::from("err"));
    //format!("Parsed markdown: {}", markdown)
    */

}

/*
 UPDATE ABOVE FUNCTION TO THE BELOW

#[plugin_fn]
pub fn process_data(json_data: String) -> FnResult<String> {
    // Deserialize the input JSON string
    let config: Config = serde_json::from_str(&json_data)?;

    // Perform some logic (replace with your actual library logic)
    let result = (10 * config.multiplier) + config.offset;

    // Return a result
    Ok(format!("Computed result: {}", result))
}


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
 */