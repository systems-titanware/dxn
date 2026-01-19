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
    pub(crate) version: String,

    // For local integrations (backward compatible)
    pub(crate) path: String,

    // For remote integrations
    pub(crate) integration_type: IntegrationType,
    pub(crate) service_name: String,
    pub(crate) url: Option<String>,

    pub(crate) functions: Vec<SystemIntegrationFunction>
}
 
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemIntegrationFunction {
    pub(crate) name: String,
    //pub(crate) params: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum IntegrationType{
    Local,
    Remote
}

// New: Service Mesh Configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceMeshConfig {
    #[serde(default)]
    pub(crate) registry_url: Option<String>,
    #[serde(default)]
    pub(crate) local_services: Option<Vec<LocalService>>,
    #[serde(default)]
    pub(crate) public_services: Option<Vec<PublicServiceConfig>>,
    #[serde(default)]
    pub(crate) discovery_interval: Option<u64>, // seconds
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocalService {
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) service_type: String,  // "ai", "wallet", "vault", etc.
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) auth: Option<ServiceAuth>,
    #[serde(default)]
    pub(crate) health_check: Option<String>, // Health check endpoint
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PublicServiceConfig {
    pub(crate) name: String,
    pub(crate) discover_from: String, // "registry"
    pub(crate) filter: ServiceFilter,
    #[serde(default)]
    pub(crate) preferred_versions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceFilter {
    #[serde(default)]
    pub(crate) service_type: Option<String>,
    #[serde(default)]
    pub(crate) public: Option<bool>,
    #[serde(default)]
    pub(crate) capabilities: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) min_version: Option<String>,
    #[serde(default)]
    pub(crate) owner: Option<String>, // UUID
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceAuth {
    pub(crate) auth_type: String, // "oauth", "api_key", "none"
    #[serde(default)]
    pub(crate) client_id: Option<String>,
    #[serde(default)]
    pub(crate) token: Option<String>,
    #[serde(default)]
    pub(crate) scopes: Option<Vec<String>>,
}


// Service Registry Models
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceRegistryEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) service_type: String,
    pub(crate) public: bool,
    pub(crate) owner: String,
    pub(crate) capabilities: Vec<String>,
    pub(crate) version: String,
    pub(crate) health: ServiceHealth,
    pub(crate) endpoints: ServiceEndpoints,
    #[serde(default)]
    pub(crate) auth_required: bool,
    #[serde(default)]
    pub(crate) scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ServiceHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceEndpoints {
    pub(crate) api: String,
    pub(crate) discovery: String,
    pub(crate) health: Option<String>,
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

