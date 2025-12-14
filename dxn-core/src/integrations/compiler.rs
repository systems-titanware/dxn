use std::process::{Command,Output};

/// Load json of list of cun
/* all compiled and run integrations inherit a trait
Get
- Gets the instance of the integration running
Run
- Runs a function passing in an enum of Function Names of itself and on that function running, performing that function
Init
- Runs an init function based on the json config loaded in */

pub fn compile(project_path: &str) -> Result<Output, std::io::Error> {
    Command::new("cargo")
        .arg("build")
        .arg("--release") // This argument specifies a release build
       
        .current_dir(project_path) // Set the working directory for the command
        .output()
}

///Deprechated
pub fn run(project_path: &str, command: &str, args: Option<&str>) -> Output {
    // Share a command 
    Command::new("cargo")
        .arg("run")
        .arg(command) // This argument specifies a release build
        .args(args)
        .current_dir(project_path) // Set the working directory for the command
        .output()
        .expect("Failed to execute cargo run")
}