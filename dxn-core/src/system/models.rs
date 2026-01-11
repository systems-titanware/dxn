use std::io;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Mutex, RwLock};
use std::vec::Vec;
use crate::data::models::{SystemData};
use crate::functions::models::{SystemFunctions};
use crate::system::server::models::{SystemServer};
use crate::integrations::models::{SystemIntegrations};
use std::fmt;
use std::error::Error;

// This struct represents state
#[derive(Deserialize, Serialize, Debug)]
pub struct AppState {
    pub(crate) app_name: String,
    pub(crate) counter: RwLock<u32>,
    pub(crate) db_name: String,
    pub(crate) system: System,
    pub(crate) uuid: Uuid
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct System { 
    pub(crate) data: SystemData,
    pub(crate) server: SystemServer,
    pub(crate) integrations: SystemIntegrations,
    pub(crate) functions: SystemFunctions
}
use wasmtime::Error as WasmtimeError;

#[derive(Debug)] // Required for the Error trait
pub enum SystemError {
    InvalidInput(String),
    NotFound,
    IO(std::io::Error), // Can wrap other error types
    Wasm(wasmtime::Error), // Can wrap other error types
}

// Traits
impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SystemError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SystemError::NotFound => write!(f, "Item not found"),
            SystemError::IO(err) => write!(f, "IO error: {}", err),
            SystemError::Wasm(err) => write!(f, "Wasm Error: {}", err),
        }
    }
}

// Implementations
impl Error for SystemError {}

impl From<io::Error> for SystemError {
    fn from(err: io::Error) -> SystemError {
        SystemError::IO(err)
    }
}

impl From<wasmtime::Error> for SystemError {
    fn from(err: wasmtime::Error) -> SystemError {
        SystemError::Wasm(err)
    }
}