use serde::{Deserialize, Serialize};
use std::vec::Vec;

use std::fmt;
use std::error::Error;

#[derive(Debug)] // Required for the Error trait
pub enum FunctionError {
    InvalidInput(String),
    NotFound(String),
    IoError(std::io::Error), // Can wrap other error types
    ExecutionError(String),
}

impl fmt::Display for FunctionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            FunctionError::NotFound(msg) => write!(f, "Function not found: {}", msg),
            FunctionError::IoError(e) => write!(f, "IO error: {}", e),
            FunctionError::ExecutionError(e) => write!(f, "Execution error: {}", e),
        }
    }
}

impl Error for FunctionError {}

impl From<std::io::Error> for FunctionError {
    fn from(err: std::io::Error) -> FunctionError {
        FunctionError::IoError(err)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnumParamTypes {
    pub format: String,
    pub data: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemFunctions { 
    pub(crate) public: Option<Vec<SystemFunctionModel>>, 
    pub(crate) private: Option<Vec<SystemFunctionModel>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FunctionType {
    Wasm,
    Native,
    Remote,
    Script,
}

fn default_function_type() -> FunctionType {
    FunctionType::Wasm
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemFunctionModel {
    pub(crate) name: String,
    #[serde(default = "default_function_type")]
    pub(crate) function_type: FunctionType,
    
    // For WASM functions (current)
    #[serde(default)]
    pub(crate) path: Option<String>,  // Path to .wasm file
    #[serde(default)]
    pub(crate) function_name: Option<String>,  // Exported function name
    
    // For Native functions
    #[serde(default)]
    pub(crate) library_path: Option<String>,  // Path to .so/.dylib
    #[serde(default)]
    pub(crate) symbol_name: Option<String>,  // Function symbol name
    
    // For Remote functions
    #[serde(default)]
    pub(crate) service_name: Option<String>,  // Service in mesh
    #[serde(default)]
    pub(crate) endpoint: Option<String>,     // Function endpoint URL
    
    // For Script functions
    #[serde(default)]
    pub(crate) script_path: Option<String>,  // Path to script file
    #[serde(default)]
    pub(crate) script_language: Option<String>, // "javascript", "typescript"
    
    pub(crate) version: u32,
    #[serde(default)]
    pub(crate) parameters: Option<Vec<String>>,  // Parameter types
    #[serde(default)]
    pub(crate) return_type: Option<String>,     // Return type
    
    // Legacy fields for backward compatibility
    #[serde(default)]
    pub(crate) params: Option<Vec<ParamTypes>>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemFunctionParameters{
    pub(crate) params: Vec<ParamTypes>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ParamTypes {
    Integer(i32),
    Float(f64),
    Text(String),
    Boolean(bool),
    Enum(EnumParamTypes),
    Tuple(i32, String), // An enum variant can also hold a tuple directly
}

// Implement
pub trait Initialize { 
    fn convert_to_tuple(&self) -> (i32, i32);
}

impl Initialize for ParamTypes {
    fn convert_to_tuple(&self) -> (i32, i32) {
        (0, 0)
    }
}

/*
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemDataModelField {
    pub(crate) name: String,
    pub(crate) datatype: String,
    pub(crate) value: String,
    pub(crate) primary: Option<bool>,
    pub(crate) secondary: Option<bool>
}
 */
 