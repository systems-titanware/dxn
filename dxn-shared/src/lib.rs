// Shared types for DXN functions
// This crate contains only types/interfaces, no implementation
// Can be used by both dxn-core and function crates without circular dependencies

use serde::{Deserialize, Serialize};

// WASM memory helpers (for WASM function crates)
// Note: These functions are designed for use in WASM modules
pub mod wasm_memory;

/// Function request payload for remote functions
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionRequest {
    pub params: serde_json::Value,
}

/// Function response payload for remote functions
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl FunctionResponse {
    pub fn success(result: serde_json::Value) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error),
        }
    }
}

