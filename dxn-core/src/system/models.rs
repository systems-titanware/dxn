use std::io;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Mutex, RwLock};
use std::vec::Vec;
use crate::data::models::{SystemData};

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
    pub(crate) data: SystemData
}
