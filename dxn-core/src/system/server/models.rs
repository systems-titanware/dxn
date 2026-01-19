use std::io;
use actix_web::dev::ResourcePath;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Mutex, RwLock};
use std::vec::Vec;
use std::collections::HashMap;
use crate::functions::models::{ParamTypes};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemServer { 
    pub(crate) public: Option<Vec<SystemServerRoute>>, 
    pub(crate) private: Option<Vec<SystemServerRoute>>,
}
/*
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemServerModel {
    pub(crate) name: String,
    pub(crate) version: u32,
    pub(crate) routes: Vec<SystemDataModelRoute>
}
 */
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct SystemServerRoute {
    pub(crate) name: String,
    pub(crate) file: String,
    pub(crate) function: Option<String>,
    pub(crate) params: Option<Vec<serde_json::Value>>, // Add this!
    pub(crate) routes: Option<Vec<SystemServerRoute>>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct FlattenRoutePath {
    pub(crate) name: String,
    pub(crate) file: String,
    pub(crate) function: Option<String>,
    pub(crate) params: Option<Vec<serde_json::Value>>,
}
 