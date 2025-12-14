use std::io::Stderr;
use std::process::Command;
use crate::integrations::models::{SystemIntegrations, SystemIntegrationModel, IntegrationError};

// tcp_client_app/src/main.rs
use dxn_shared::{RequestMessage, ResponseMessage};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;

/// Load json of list of cun
/* all compiled and run integrations inherit a trait
Get
- Gets the instance of the integration running
Run
- Runs a function passing in an enum of Function Names of itself and on that function running, performing that function
Init
- Runs an init function based on the json config loaded in */
use crate::integrations::constants::SHARED_FILES_PATH;

static mut PUBLIC_INTEGRATIONS: Vec<SystemIntegrationModel> = Vec::new();

pub fn get(integration_name: &str) -> Option<SystemIntegrationModel> {
    unsafe {
        PUBLIC_INTEGRATIONS
            .iter()
            .to_owned()
            .filter(|item| {
                item.name == integration_name
            })
            .last()
            .cloned()
    }
}
pub fn run(integration_name: &str, command: &str, args: Option<&str>) -> Result<String, IntegrationError>{
    unsafe {
        let matched_integration = PUBLIC_INTEGRATIONS
        .iter()
        .to_owned()
        .filter(|item| {
            item.name == integration_name
        })
        .last();
        
        match matched_integration {
            Some(integration) => {
                // Get path of integration
                let path: String = format!("{}/{}", SHARED_FILES_PATH, integration.path);
                // Run integration
              
                let output = crate::integrations::compiler::run(&path, command, args);

                if output.status.success() {
                    //TODO: Change to logger 
                    crate::system::logger::log(format!("cargo run success: {}", integration.path).as_str());
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    //eprintln!("cargo run failed: {}", integration.path);
                    //eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                    crate::system::logger::log_error(format!("cargo run failed: {}", integration.path).as_str());
                    Err(IntegrationError::RunTime(format!("cargo run failed: {}", integration.path)))
                } 
            },
            None => {
                crate::system::logger::log_error(format!("No integration found: {}", integration_name).as_str());
                Err(IntegrationError::NotFound(format!("cargo not found: {}", integration_name)))
            }
        }
    }
    
}


pub async fn run_new(integration_name: &str, command: &str, args: Option<&str>) {
    unsafe {
        let matched_integration = PUBLIC_INTEGRATIONS
        .iter()
        .to_owned()
        .filter(|item| {
            item.name == integration_name
        })
        .last();
        
        println!("RUN {:?}", matched_integration);
        match matched_integration {
            Some(integration) => {
                println!("Some ");
                match run_tcp().await {
                    Ok(content) => {
                        println!("TCP RESULT: {:?}", content);
                    },
                    Err(err ) => {
                        eprintln!("TCP Err: {:?}", err);
                    }
                }
                /*
                let path: String = format!("{}/{}", SHARED_FILES_PATH, integration.path);
                // run function of integration
                let output = crate::integrations::compiler::run(&path, command, args);

                if output.status.success() {
                    println!("cargo run successful in {}", integration.path);
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    eprintln!("cargo run failed in {}", integration.path);
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                */
            },
            None => {
                println!("No integration found: {}", integration_name);
            }
        }
    }
    
}

pub fn init(integration: SystemIntegrationModel) {
    let path: String = format!("{}/{}", SHARED_FILES_PATH, &integration.path);
    // Create list of integration run-times
    // Threads that operations can execute on
    let output = crate::integrations::compiler::compile(&path);

    //let run: std::process::Output = crate::integrations::compiler::run(&path, "main", None);
    match(output) {
        Ok(status) => {
            println!("cargo build & run successful in {}", path);
            unsafe { PUBLIC_INTEGRATIONS.push(integration) };
        },
        Err(err) => {
        eprintln!("cargo build failed in {}:\n{:?}", path, err);

        }
    }
}



async fn run_tcp() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("0.0.0.0:8080").await?;
    println!("Client connected to server!");

    let request = RequestMessage {
        id: 1,
        payload: "Hello from the client!".to_string(),
    };

    // Serialize the request
    let request_json = serde_json::to_vec(&request)?;

    // Send the request
    stream.write_all(&request_json).await?;
    println!("Sent request: {:?}", request);

    // Read the response
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    let response: ResponseMessage = serde_json::from_slice(&buf[..n])?;
    
    println!("Received response: {:?}", response);

    Ok(())
}