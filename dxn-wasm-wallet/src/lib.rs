// WASM Wallet Functions Example
// This crate compiles to WASM and can be shared across platforms

use dxn_shared::wasm_memory::*;
use serde::{Deserialize, Serialize};

// Define structs internally (no external dependencies!)
#[derive(Deserialize, Serialize, Debug)]
struct WalletConfig {
    address: String,
    network: String,
    balance: u64,
}

#[derive(Serialize, Debug)]
struct WalletBalance {
    address: String,
    balance: u64,
    network: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct TransactionRequest {
    from: String,
    to: String,
    amount: u64,
    gas_limit: Option<u64>,
}

#[derive(Serialize, Debug)]
struct TransactionResult {
    success: bool,
    transaction_id: Option<String>,
    error: Option<String>,
}

/// WASM function that accepts a WalletConfig struct via JSON
/// 
/// Signature: (ptr: i32, len: i32) -> i64
/// The executor calls this with:
/// - ptr: pointer to JSON string in WASM memory (offset 1024)
/// - len: length of JSON string
/// 
/// Returns:
/// - i64: packed (ptr, len) where ptr points to result JSON in WASM memory
#[no_mangle]
pub extern "C" fn get_wallet_balance(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Read and deserialize JSON into struct
        let config: WalletConfig = match deserialize_json(json_ptr, json_len) {
            Ok(cfg) => cfg,
            Err(e) => {
                // Return error as JSON
                return write_error(&e);
            }
        };
        
        // Use struct fields directly (type-safe!)
        let balance = WalletBalance {
            address: config.address.clone(),
            balance: config.balance,
            network: config.network.clone(),
        };
        
        // Serialize and write result
        serialize_and_write(&balance)
    }
}

/// WASM function that accepts a TransactionRequest struct via JSON
#[no_mangle]
pub extern "C" fn create_transaction(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Read and deserialize JSON into struct
        let request: TransactionRequest = match deserialize_json(json_ptr, json_len) {
            Ok(req) => req,
            Err(e) => {
                return write_error(&e);
            }
        };
        
        // Process the transaction (your business logic)
        let transaction_id = format!("tx_{}_{}", request.from, request.amount);
        
        let result = TransactionResult {
            success: true,
            transaction_id: Some(transaction_id),
            error: None,
        };
        
        // Serialize and write result
        serialize_and_write(&result)
    }
}

/// WASM function that validates an address (accepts JSON string)
#[no_mangle]
pub extern "C" fn validate_address(json_ptr: i32, json_len: i32) -> i64 {
    unsafe {
        // Read JSON string
        let json_str = match read_json_from_memory(json_ptr, json_len) {
            Ok(s) => s,
            Err(e) => {
                return write_error(&e);
            }
        };
        
        // Parse address from JSON (could be just a string or {"address": "..."})
        let address = match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(v) => {
                if let Some(addr) = v.get("address") {
                    addr.as_str().unwrap_or("")
                } else if v.is_string() {
                    v.as_str().unwrap_or("")
                } else {
                    ""
                }
            },
            Err(_) => {
                // Try as plain string
                json_str.trim_matches('"')
            }
        };
        
        // Simple validation: check if address starts with "0x" and has length > 10
        let is_valid = address.starts_with("0x") && address.len() > 10;
        let result = serde_json::json!({
            "valid": is_valid,
            "address": address
        });
        
        serialize_and_write(&result)
    }
}

