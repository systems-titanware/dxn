//use serde::Deserialize;
use actix_web::{ web,  HttpResponse, HttpRequest, Responder}; 
use crate::data::db::sqlite; 
use crate::data::db::sqlite::migrations;
use crate::data::db::sqlite::repository_schema;
use crate::data::db::sqlite::repository_events;
use crate::data::models::{QueryParams, EventType}; 
use crate::system::server::models::{ApiResultResponse, ApiErrorWithCode, DataApiMeta, ListResponse, Pagination};
 
use rusqlite::{Row, types::ValueRef, Result, Error as SqlError};
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
//If T is needed for trait bounds or methods but not a field: You can use std::marker::PhantomData<T> to explicitly tell the compiler that you are aware of the unused parameter and intend to use it to "act like" the struct owns a T. PhantomData takes up no memory space.
use std::collections::HashMap;
use serde_json::{json, Value, Map};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

pub(crate) fn remove_last_char(s: &str) -> &str {
    match s.char_indices().next_back() {
        Some((i, _)) => &s[..i], // Slice from the beginning up to the start of the last char
        None => s, // If the string is empty, return it as is
    }
}

pub(crate) fn get_object_from_path(full_path: &str) -> &str {
    let parts: Vec<&str> = full_path.split('/').collect();
    let slice = &parts[..4]; // slice1 will be &[20, 30, 40]

    let object = slice.last();
    let obj_str: &str = object.unwrap();
    return obj_str;
}

// ============================================================================
// SHARED VALIDATION & ERROR HELPERS (DRY)
// ============================================================================

/// Validates that a schema exists and is active (not deleted).
/// Returns `Some(HttpResponse)` with error if schema doesn't exist, is deleted, or on error.
/// Returns `None` if schema is valid, exists, and is active.
fn validate_schema_exists(schema_name: &str) -> Option<HttpResponse> {
    // First check if schema exists and is active
    match repository_schema::schema_exists(schema_name) {
        Ok(true) => None, // Schema exists and is active, continue
        Ok(false) => {
            // Check if it's deleted
            match repository_schema::is_schema_deleted(schema_name) {
                Ok(true) => Some(schema_deleted_response(schema_name)),
                _ => Some(schema_not_found_response(schema_name)),
            }
        }
        Err(e) => {
            eprintln!("Error checking schema existence: {}", e);
            Some(internal_error_response("Failed to validate schema"))
        }
    }
}

/// Creates a standardized "schema not found" error response
fn schema_not_found_response(schema_name: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ApiResultResponse::<Value> {
        success: false,
        data: None,
        error: Some(ApiErrorWithCode {
            code: "schema_not_found".to_string(),
            message: format!("Schema '{}' does not exist. Create it first via POST /api/schema", schema_name),
            details: None,
        }),
        meta: None,
    })
}

/// Creates a standardized "schema deleted" error response
fn schema_deleted_response(schema_name: &str) -> HttpResponse {
    HttpResponse::Gone().json(ApiResultResponse::<Value> {
        success: false,
        data: None,
        error: Some(ApiErrorWithCode {
            code: "schema_deleted".to_string(),
            message: format!("Schema '{}' has been deleted. Restore it via PUT /api/schema/{}/restore to access data.", schema_name, schema_name),
            details: None,
        }),
        meta: None,
    })
}

/// Creates a standardized "record not found" error response
fn record_not_found_response() -> HttpResponse {
    HttpResponse::NotFound().json(ApiResultResponse::<Value> {
        success: false,
        data: None,
        error: Some(ApiErrorWithCode {
            code: "not_found".to_string(),
            message: "Record not found".to_string(),
            details: None,
        }),
        meta: None,
    })
}

/// Creates a standardized internal error response
fn internal_error_response(message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ApiResultResponse::<Value> {
        success: false,
        data: None,
        error: Some(ApiErrorWithCode {
            code: "internal_error".to_string(),
            message: message.to_string(),
            details: None,
        }),
        meta: None,
    })
}

/// Retrieves the current state of a record for event sourcing.
/// Returns None if record doesn't exist or on error.
fn get_record_state(schema_name: &str, id: &str) -> Option<serde_json::Value> {
    let mapper = |row: &Row| row_to_json_value(row);
    
    match sqlite::repository::get("public".to_string(), schema_name.to_string(), id.to_string(), mapper) {
        Ok(state) => Some(state),
        Err(_) => None,
    }
}

// ============================================================================
// CRUD HANDLERS
// ============================================================================

// GET
/// Retrieves a single record by ID (supports both numeric and UUID formats)
/// Path: /{object_name}/{id}
pub async fn get(req: HttpRequest, path: web::Path<(String, String)>) -> impl Responder {
    let (object_name, id) = path.into_inner();
    
    // Validate schema exists (dynamic route support)
    if let Some(error_response) = validate_schema_exists(&object_name) {
        return error_response;
    }
    
    let mapper = |row: &Row| {
        row_to_json_value(row)
    };

    let result = sqlite::repository::get("public".to_string(), object_name.clone(), id, mapper);

    match result {
        Ok(content) => {
            HttpResponse::Ok().json(ApiResultResponse {
                success: true,
                data: Some(content),
                error: None,
                meta: None,
            })
        }
        Err(err) => {
            eprintln!("Error Getting Data: {}", err);
            match err {
                SqlError::QueryReturnedNoRows => record_not_found_response(),
                _ => internal_error_response("An internal error occurred while retrieving the record"),
            }
        }
    }
}

/// Maps a rusqlite::Row to a serde_json::Value::Object.
pub(crate) fn row_to_json_value(row: &Row) -> Result<Value> {
    let mut map = Map::new();
    // Get column names from the statement associated with the row
    let statement = row.as_ref();
    let column_names: Vec<String> = statement.column_names().iter().map(|&s| s.to_string()).collect();

    for (i, name) in column_names.iter().enumerate() {
        let value_ref = row.get_ref(i)?;
        let json_value = match value_ref {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(i) => Value::Number(serde_json::Number::from(i)),
            ValueRef::Real(f) => {
                // Represent as f64, handle potential precision issues if critical
                serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null)
            }
            ValueRef::Text(bytes) | ValueRef::Blob(bytes) => {
                // Attempt to interpret as UTF-8 string
                let text = String::from_utf8_lossy(bytes).into_owned();
                Value::String(text)
            }
        };
        map.insert(name.clone(), json_value);
    }

    Ok(Value::Object(map))
}
    
// LIST
pub async fn list(req: HttpRequest, query_params:  web::Query<QueryParams>) -> impl Responder {
    let object = get_object_from_path(req.path());
    
    // Validate schema exists (dynamic route support)
    if let Some(error_response) = validate_schema_exists(object) {
        return error_response;
    }
    
    let page_size = query_params.page_size.unwrap_or(10) as u32;
    let page = query_params.page.unwrap_or(1) as u32;
    let query = query_params.query.clone().unwrap_or_default();
    
    // Define the closure in the parent function/scope
    let mapper = |row: &Row| {
        row_to_json_value(row)
    };

    let items = sqlite::repository::list(
        "public".to_string(),
        object.to_string(),
        page_size,
        page,
        query,
        mapper,
    );
    
    match items {
        Ok(results) => {
            let total = results.len() as u32;
            let total_pages = if page_size > 0 {
                ((total as f64 + page_size as f64 - 1.0) / page_size as f64).ceil() as u32
            } else {
                1
            };
            HttpResponse::Ok().json(ListResponse {
                data: results,
                pagination: Pagination {
                    page,
                    page_size,
                    total,
                    total_pages,
                },
            })
        }
        Err(err) => {
            eprintln!("Error listing data: {:?}", err);
            internal_error_response("An internal error occurred while listing records")
        }
    }
}
 
// Define a generic struct for the request body
//FieldNames
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Payload<T>
{
    pub names: Vec<String>,
    pub values: Vec<String>,
    pub data: T
} 
// Define a custom error type for your trait
#[derive(Debug)]
pub enum MyError {
    FileNotFound,
    PermissionDenied,
    InvalidData(String),
    NotFound(String),
    Other(String),
}


// POST
/// Creates a new record and emits a Created event
/// 
/// # Event Sourcing Architecture
/// 
/// TODO: Move to event-first architecture:
/// Currently using dual-write pattern (write to data table, then emit event).
/// Future improvement: Emit event first (source of truth), then apply to read model via projection.
/// 
/// ```rust
/// // Current (dual-write):
/// repository::insert(...);  // Write to data table
/// repository_events::create_and_append_event(...);  // Then emit event
/// 
/// // Future (event-first):
/// let event = repository_events::create_and_append_event(...);  // Emit event first
/// apply_event_to_read_model(&event);  // Projection updates data table
/// ```
pub async fn post<T>(req: HttpRequest, payload: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder 
where
    T: std::fmt::Debug + Deserialize<'static> + Serialize + 'static, // Required traits for T
{
    // get object from path: /api/data/{object}
    let object = get_object_from_path(req.path()).to_string();
    
    // Validate schema exists (dynamic route support)
    if let Some(error_response) = validate_schema_exists(&object) {
        return error_response;
    }
    
    let body = payload.into_inner();

    let keys: Vec<String> = body.keys().cloned().collect();
    let values: Vec<Value> = body.values().cloned().collect();

    // TODO: Event-first - emit Created event here, then apply via projection
    let result = sqlite::repository::insert(
        "public".to_string(), 
        object.clone(), 
        keys, 
        values,
    );

    match result {
        Ok(new_id) => {
            // Emit Created event for event sourcing
            let aggregate_id = new_id.to_string();
            if let Err(e) = repository_events::create_and_append_event(
                &aggregate_id,
                &object,
                EventType::Created,
                json!(body),
                None,
                None, // TODO: Extract user_id from auth header
            ) {
                eprintln!("Warning: Failed to emit Created event: {}", e);
                // Don't fail the request, just log the warning
            }
            
            HttpResponse::Created().json(ApiResultResponse {
                success: true,
                data: Some(json!({
                    "id": new_id,
                    "object": object,
                    "attributes": body
                })),
                error: None,
                meta: None,
            })
        }
        Err(err) => {
            eprintln!("Error creating object: {}", err);
            internal_error_response("An internal error occurred while creating the record")
        }
    }
}

// PUT
/*
pub async fn put<T>(req: HttpRequest, payload: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder 
where
    T: std::fmt::Debug + Deserialize<'static> + Serialize + 'static, // Required traits for T
*/

// PUT
/// Updates an existing record by ID and emits an Updated event
/// Path: /{object_name}/{id}
/// 
/// # Event Sourcing Architecture
/// 
/// TODO: Move to event-first architecture (see POST handler for details)
pub async fn put<T>(req: HttpRequest, path: web::Path<(String, String)>, payload: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder 
where
    T: std::fmt::Debug + Deserialize<'static> + Serialize + 'static, // Required traits for T
{
    let (object_name, id) = path.into_inner();
    
    // Validate schema exists (dynamic route support)
    if let Some(error_response) = validate_schema_exists(&object_name) {
        return error_response;
    }
    
    let body = payload.into_inner();

    // Get previous state before update (for event sourcing)
    let previous_state = get_record_state(&object_name, &id);

    let keys: Vec<String> = body.keys().cloned().collect();
    let values: Vec<Value> = body.values().cloned().collect();

    // TODO: Event-first - emit Updated event here, then apply via projection
    let result = sqlite::repository::update(
        "public".to_string(), 
        object_name.clone(),
        id.clone(),
        keys, 
        values,
    );

    match result {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                record_not_found_response()
            } else {
                // Emit Updated event for event sourcing
                if let Err(e) = repository_events::create_and_append_event(
                    &id,
                    &object_name,
                    EventType::Updated,
                    json!(body),
                    previous_state,
                    None, // TODO: Extract user_id from auth header
                ) {
                    eprintln!("Warning: Failed to emit Updated event: {}", e);
                }
                
                HttpResponse::Ok().json(ApiResultResponse {
                    success: true,
                    data: Some(json!({
                        "id": id,
                        "object": object_name,
                        "updated": true
                    })),
                    error: None,
                    meta: None,
                })
            }
        }
        Err(err) => {
            eprintln!("Update error: {}", err);
            internal_error_response("An internal error occurred while updating the record")
        }
    }
}

// DELETE
/// Deletes a record by ID and emits a Deleted event
/// Path: /{object_name}/{id}
/// 
/// # Event Sourcing Architecture
/// 
/// TODO: Move to event-first architecture (see POST handler for details)
pub async fn delete(req: HttpRequest, path: web::Path<(String, String)>) -> impl Responder {
    let (object_name, id) = path.into_inner();
    
    // Validate schema exists (dynamic route support)
    if let Some(error_response) = validate_schema_exists(&object_name) {
        return error_response;
    }

    // Get previous state before delete (for event sourcing)
    let previous_state = get_record_state(&object_name, &id);

    // TODO: Event-first - emit Deleted event here, then apply via projection
    let delete = sqlite::repository::delete("public".to_string(), object_name.clone(), id.clone());

    match delete {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                record_not_found_response()
            } else {
                // Emit Deleted event for event sourcing
                if let Err(e) = repository_events::create_and_append_event(
                    &id,
                    &object_name,
                    EventType::Deleted,
                    json!({"deleted": true}),
                    previous_state,
                    None, // TODO: Extract user_id from auth header
                ) {
                    eprintln!("Warning: Failed to emit Deleted event: {}", e);
                }
                
                HttpResponse::Ok().json(ApiResultResponse {
                    success: true,
                    data: Some(json!({
                        "id": id,
                        "object": object_name,
                        "deleted": true
                    })),
                    error: None,
                    meta: None,
                })
            }
        }
        Err(err) => { 
            eprintln!("Delete error: {}", err);
            internal_error_response("An internal error occurred while deleting the record")
        }
    }
} 

// ============================================================================
// MIGRATION ROUTES
// ============================================================================

/// Request body for migration operations
#[derive(Debug, Deserialize, Serialize)]
pub struct MigrationRequest {
    /// Database name (e.g., "public", "private")
    pub db_name: Option<String>,
    /// Force apply (skip approval checks)
    pub force: Option<bool>,
    /// Migration ID (for single migration operations)
    pub migration_id: Option<String>,
}

/// Apply a specific migration by ID
/// 
/// POST /api/data/migrate/{migration_id}
/// Body: { "db_name": "public", "force": false }
pub async fn apply_migration_route(
    path: web::Path<String>,
    payload: web::Json<MigrationRequest>,
) -> impl Responder {
    let migration_id = path.into_inner();
    let db_name = payload.db_name.as_deref().unwrap_or("public");
    let force = payload.force.unwrap_or(false);

    // Load all migrations to find the one we want
    let migrations_list = match migrations::load_migrations() {
        Ok(ms) => ms,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to load migrations: {}", e)
            }));
        }
    };

    let migration = migrations_list.iter().find(|m| m.id == migration_id);
    
    match migration {
        Some(m) => {
            match migrations::apply_migration(db_name, m, force) {
                Ok(result) => {
                    match result {
                        migrations::MigrationResult::Applied => {
                            HttpResponse::Ok().json(json!({
                                "status": "success",
                                "message": format!("Migration '{}' applied successfully", migration_id),
                                "migration_id": migration_id
                            }))
                        }
                        migrations::MigrationResult::RequiresApproval { reason } => {
                            HttpResponse::BadRequest().json(json!({
                                "status": "requires_approval",
                                "message": reason,
                                "migration_id": migration_id,
                                "hint": "Set 'force': true in request body to apply this migration"
                            }))
                        }
                        migrations::MigrationResult::Failed { error } => {
                            HttpResponse::InternalServerError().json(json!({
                                "status": "failed",
                                "error": error,
                                "migration_id": migration_id
                            }))
                        }
                    }
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("Migration error: {}", e),
                        "migration_id": migration_id
                    }))
                }
            }
        }
        None => {
            HttpResponse::NotFound().json(json!({
                "error": format!("Migration '{}' not found", migration_id)
            }))
        }
    }
}

/// Apply all pending migrations
/// 
/// POST /api/data/migrate/all
/// Body: { "db_name": "public", "force": false }
pub async fn apply_all_migrations_route(
    payload: web::Json<MigrationRequest>,
) -> impl Responder {
    let db_name = payload.db_name.as_deref().unwrap_or("public");
    let force = payload.force.unwrap_or(false);

    match migrations::apply_all_pending(db_name, force) {
        Ok(results) => {
            let mut applied = Vec::new();
            let mut requires_approval = Vec::new();
            let mut failed = Vec::new();

            for (migration_id, result) in results {
                match result {
                    migrations::MigrationResult::Applied => {
                        applied.push(migration_id);
                    }
                    migrations::MigrationResult::RequiresApproval { reason } => {
                        requires_approval.push(json!({
                            "migration_id": migration_id,
                            "reason": reason
                        }));
                    }
                    migrations::MigrationResult::Failed { error } => {
                        failed.push(json!({
                            "migration_id": migration_id,
                            "error": error
                        }));
                    }
                }
            }

            HttpResponse::Ok().json(json!({
                "status": "completed",
                "applied": applied,
                "requires_approval": requires_approval,
                "failed": failed,
                "summary": {
                    "total": applied.len() + requires_approval.len() + failed.len(),
                    "applied": applied.len(),
                    "requires_approval": requires_approval.len(),
                    "failed": failed.len()
                }
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to apply migrations: {}", e)
            }))
        }
    }
}

/// List all migrations and their status
/// 
/// GET /api/data/migrate/list?db_name=public
pub async fn list_migrations_route(
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let db_name = query.get("db_name").map(|s| s.as_str()).unwrap_or("public");

    // Load all migrations
    let migrations_list = match migrations::load_migrations() {
        Ok(ms) => ms,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to load migrations: {}", e)
            }));
        }
    };

    // Get applied migrations
    let applied = match migrations::get_applied_migrations(db_name) {
        Ok(applied_ids) => applied_ids,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get applied migrations: {}", e)
            }));
        }
    };

    // Build response with status for each migration
    let migrations_with_status: Vec<serde_json::Value> = migrations_list
        .iter()
        .map(|m| {
            let is_applied = applied.contains(&m.id);
            json!({
                "id": m.id,
                "description": m.description,
                "created_at": m.created_at,
                "applied": is_applied,
                "requires_approval": migrations::requires_approval(m).is_some()
            })
        })
        .collect();

    HttpResponse::Ok().json(json!({
        "db_name": db_name,
        "migrations": migrations_with_status,
        "summary": {
            "total": migrations_list.len(),
            "applied": applied.len(),
            "pending": migrations_list.len() - applied.len()
        }
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    // Register migration routes FIRST (must come before catch-all to take precedence)
    cfg.service(
        web::scope("/migrate")
            .route("/list", web::get().to(list_migrations_route))
            .route("/list/", web::get().to(list_migrations_route))
            .route("/all", web::post().to(apply_all_migrations_route))
            .route("/all/", web::post().to(apply_all_migrations_route))
            .route("/{migration_id}", web::post().to(apply_migration_route))
            .route("/{migration_id}/", web::post().to(apply_migration_route))
    );

    // Dynamic catch-all routes for any schema
    // Schema existence is validated at runtime in each handler
    // Note: Both "" (no trailing slash) and "/" (trailing slash) routes are needed
    cfg.service(
        web::scope("/{object_name}")
            .route("/list", web::get().to(list))
            .route("/list/", web::get().to(list))
            .route("/{id}", web::get().to(get))
            .route("/{id}/", web::get().to(get))
            .route("", web::post().to(post::<HashMap<String, serde_json::Value>>))
            .route("/", web::post().to(post::<HashMap<String, serde_json::Value>>))
            .route("/{id}", web::put().to(put::<HashMap<String, serde_json::Value>>))
            .route("/{id}/", web::put().to(put::<HashMap<String, serde_json::Value>>))
            .route("/{id}", web::delete().to(delete))
            .route("/{id}/", web::delete().to(delete))
    );
}

#[cfg(test)]
#[path = "data.test.rs"]
mod tests;
