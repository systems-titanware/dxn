use std::io;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Mutex, RwLock};
use std::vec::Vec;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemData { 
    pub(crate) public: Option<Vec<SystemDataModel>>, 
    pub(crate) private: Option<Vec<SystemDataModel>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemDataModel {
    pub(crate) name: String,
    pub(crate) version: u32,
    pub(crate) fields: Vec<SystemDataModelField>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemDataModelField {
    pub(crate) name: String,
    pub(crate) datatype: String,
    pub(crate) value: String,
    pub(crate) primary: Option<bool>,
    pub(crate) secondary: Option<bool>
}


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemDataModelMetadata {
    pub(crate) name: String,
    pub(crate) version: u32,
    pub(crate) fields: String
}

#[derive(Deserialize, Serialize)]
pub struct QueryParams {
    pub(crate) page_size: Option<u8>,
    pub(crate) page: Option<u8>,
    pub(crate) query: Option<String>,
}