use std::io;
use actix_web::dev::ResourcePath;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use std::sync::{Mutex, RwLock};
use std::vec::Vec;
use std::collections::HashMap;
use crate::functions::models::{ParamTypes};

// ============================================================================
// API RESPONSE TYPES (Shared across controllers)
// ============================================================================

/// Standard API response wrapper for successful responses
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ApiMeta>,
}

/// Standard API response wrapper with success/error pattern (used by data controller)
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiResultResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorWithCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<DataApiMeta>,
}

/// Pagination metadata for data list responses (with optional fields)
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataApiMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<u32>,
}

/// Pagination metadata for list responses
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Unified pagination object for list endpoints (serializes as "pagination" in JSON).
#[derive(Serialize, Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Unified list response: data array + pagination. Use for all list endpoints.
#[derive(Serialize, Debug)]
pub struct ListResponse<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

/// API error response structure
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// API error with code field (used by data controller)
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorWithCode {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

// ============================================================================
// SCHEMA REQUEST TYPES
// ============================================================================

/// Request body for creating a new schema
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaRequest {
    pub name: String,
    #[serde(default = "default_db")]
    pub db: String,
    #[serde(default)]
    pub public: bool,
    /// Icon for the schema (emoji or icon identifier, e.g., "📦", "mdi-account")
    #[serde(default)]
    pub icon: Option<String>,
    pub fields: Vec<SchemaFieldRequest>,
}

/// Field definition for schema create/update requests
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaFieldRequest {
    pub name: String,
    pub datatype: String,
    #[serde(default)]
    pub primary: Option<bool>,
    #[serde(default)]
    pub secondary: Option<bool>,
    #[serde(default)]
    pub value: String,
}

/// Request body for updating a schema
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSchemaRequest {
    #[serde(default)]
    pub db: Option<String>,
    #[serde(default)]
    pub public: Option<bool>,
    /// Icon for the schema (emoji or icon identifier, e.g., "📦", "mdi-account")
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub fields: Option<Vec<SchemaFieldRequest>>,
}

/// Default database name for schemas
fn default_db() -> String {
    "public".to_string()
}

// ============================================================================
// SERVER CONFIGURATION TYPES
// ============================================================================

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
    #[serde(default)]
    pub(crate) layout: Option<String>,  // Layout file path (e.g., "global.layout.html")
    pub(crate) routes: Option<Vec<SystemServerRoute>>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Optional: if JSON keys are camelCase
pub struct FlattenRoutePath {
    pub(crate) name: String,
    pub(crate) file: String,
    #[serde(default)]
    pub(crate) layout: Option<String>,  // Layout file path
}
 