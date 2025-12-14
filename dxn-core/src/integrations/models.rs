use std::io;
use actix_web::dev::ResourcePath;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Mutex, RwLock};
use std::vec::Vec;
use std::collections::HashMap;
use std::fmt;
use std::error::Error;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemIntegrations { 
    pub(crate) public: Option<Vec<SystemIntegrationModel>>, 
    pub(crate) private: Option<Vec<SystemIntegrationModel>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemIntegrationModel {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) version: String,
    pub(crate) functions: Vec<SystemIntegrationFunction>
}
 
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemIntegrationFunction {
    pub(crate) name: String,
    //pub(crate) params: Option<Vec<String>>,
}


// 1. Define the custom error enum and derive Debug for easy printing during development.
#[derive(Debug)]
pub enum IntegrationError {
    InvalidInput(String),
    NotFound(String),
    RunTime(String), // Can wrap other standard errors
    Error(std::io::Error), // Can wrap other standard errors
}

// 2. Implement the Display trait for user-friendly error messages.
impl fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntegrationError::InvalidInput(msg) => write!(f, "Integration input error: {}", msg),
            IntegrationError::NotFound(msg) => write!(f, "Integration not found: {}", msg),
            IntegrationError::RunTime(e) => write!(f, "Integration runtime error: {}", e),
            IntegrationError::Error(e) => write!(f, "Integration error: {}", e),
        }
    }
}

impl Error for IntegrationError {}

// Optional: Implement `From` to easily convert other error types into your custom enum using the `?` operator.
impl From<std::io::Error> for IntegrationError {
    fn from(err: std::io::Error) -> IntegrationError {
        IntegrationError::Error(err)
    }
}

