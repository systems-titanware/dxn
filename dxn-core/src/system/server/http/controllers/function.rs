use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::functions::models::SystemFunctions;
use crate::functions::manager;
use crate::system::models::AppState;
use serde_json::{json, Value};
use std::collections::HashMap;

/// POST handler for function execution
/// 
/// Route pattern: /api/function/{function_name}
/// Body: JSON object that will be passed as parameters to the function
/// 
/// Example request:
///   POST /api/function/get_wallet_balance
///   Body: {"address": "0x1234...", "network": "mainnet", "balance": 1000}
/// 
/// Returns: Function result as JSON
pub async fn execute_function(
    app: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    payload: web::Json<HashMap<String, serde_json::Value>>,
) -> impl Responder {
    // Extract function name from path parameter
    let function_name = path.into_inner();
    println!("[DEBUG] execute_function called with function_name: '{}'", function_name);

    // Convert request body (HashMap) to parameters array
    let body_map = payload.into_inner();
    let params: Vec<serde_json::Value> = body_map.into_values().collect();

    println!("[DEBUG] Calling function '{}' with {} parameters", function_name, params.len());

    // Call the function via manager (resolve paths under project_root/dxn-files)
    match manager::call_function(&function_name, &params, Some(&app.project_root)).await {
        Ok(result) => {
            // Check if result contains an error field
            if let Some(error) = result.get("error") {
                HttpResponse::BadRequest().json(result)
            } else {
                HttpResponse::Ok().json(result)
            }
        },
        Err(crate::functions::models::FunctionError::NotFound(name)) => {
            HttpResponse::NotFound().json(json!({
                "error": format!("Function '{}' not found", name)
            }))
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Function execution error: {:?}", e)
            }))
        }
    }
}

/// Configure function routes
/// 
/// Sets up POST routes for each function in the public functions list
/// Route pattern: /api/function/{function_name}
pub fn config(cfg: &mut web::ServiceConfig, data: SystemFunctions) {
    println!("[DEBUG] Function config called, public functions: {:?}", 
        data.public.as_ref().map(|v| v.len()));
    
    match data.public {
        Some(vec) => {
            if !vec.is_empty() {
                println!("[DEBUG] Registering function routes with {} functions", vec.len());
                // Don't add another scope - it's already set in main.rs
                // Just add the routes directly
                cfg.route("/{function_name}", web::post().to(execute_function));
                cfg.route("/{function_name}/", web::post().to(execute_function));
            } else {
                println!("[DEBUG] WARNING: Functions list is empty, route not registered!");
            }
        }
        None => {
            println!("[DEBUG] WARNING: No public functions configured, route not registered!");
        }
    }
}

#[cfg(test)]
#[path = "function.test.rs"]
mod tests;
