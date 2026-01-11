use serde::{Deserialize, Serialize};
use std::vec::Vec;

use std::fmt;
use std::error::Error;

#[derive(Debug)] // Required for the Error trait
enum FunctionError {
    InvalidInput(String),
    NotFound,
    IoError(std::io::Error), // Can wrap other error types
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct EnumParamTypes {
    pub format: String,
    // TODO: Need to change this
    pub data: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemFunctions { 
    pub(crate) public: Option<Vec<SystemFunctionModel>>, 
    pub(crate) private: Option<Vec<SystemFunctionModel>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemFunctionModel {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) function_name: String,
    pub(crate) version: u32,
    pub(crate) params: Option<Vec<ParamTypes>>
    //pub(crate) params: SystemFunctionParameters
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
 