use rustyscript::{Script, Module};
use tokio; // For asynchronous operations

pub async fn run_script() -> impl actix_web::Responder{
    // Load the transpiled JavaScript module
    let module = Module::from_file("_/files/functions/my_function.js")
        .await?;

    // Create a new script runtime
    let mut script = Script::new();

    // Load the module into the script
    script.load_module(module).await?;

    // Call the exported TypeScript function (now JavaScript)
    let result: String = script.call_function("greet", ("World",)).await?;

    println!("{}", result); // Output: Hello, World!

    Ok(())
}