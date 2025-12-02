use std::io;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Mutex, RwLock};
use std::vec::Vec;

pub struct DbColumn {
    pub(crate) name: String,
    pub(crate) value: Option<String>,
    pub(crate) col_type: String,
    pub(crate) primary_key: Option<bool>,
    pub(crate) secondary_key: Option<bool>,
    pub(crate) nullable: bool,
    pub(crate) unique: Option<bool>
}