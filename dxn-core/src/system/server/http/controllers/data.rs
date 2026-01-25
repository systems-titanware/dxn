//use serde::Deserialize;
use actix_web::{ web,  HttpResponse, HttpRequest, Responder}; 
use crate::data::db::sqlite; 
use crate::data::db::sqlite::migrations;
use crate::data::models::{SystemData, QueryParams}; 
 
use rusqlite::{Row, types::ValueRef, Result};
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
//If T is needed for trait bounds or methods but not a field: You can use std::marker::PhantomData<T> to explicitly tell the compiler that you are aware of the unused parameter and intend to use it to "act like" the struct owns a T. PhantomData takes up no memory space.
use std::collections::HashMap;
use serde_json::{json, Value, Map};

// HELPER FUNCTIONS
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

// GET
/// extract path info from "/users/{user_id}/{friend}" url
/// {user_id} - deserializes to a u32
pub async fn get(req: HttpRequest, path: web::Path<u32>) -> impl Responder {
    let id = path.into_inner();
    let mapper = |row: &Row| {
        row_to_json_value(row)
    };

    let object = get_object_from_path(req.path());
    let result = sqlite::repository::get("public".to_string(), object.to_string(), id, mapper);

    match result {
        Ok(content) => {
            HttpResponse::Ok().json(json!(content))
        }
        Err(err) => {
            let err_message = format!("Error Getting Data: {}", err);
            eprintln!("Error Getting Data: {}", err);
            HttpResponse::Ok().body(err_message)
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
    let mut page_size = 10;
    let mut page = 10;
    let mut query = &String::new();

    match query_params.page_size { Some(size) => { page_size = size }, None => {} }

    match query_params.page { Some(size) => { page = size }, None => {} }

    match &query_params.query { Some(qs) => { query = qs }, None => {} }
    
    // Define the closure in the parent function/scope
    let mapper = |row: &Row| {
        row_to_json_value(row)
    };
 
    let object = get_object_from_path(req.path());

    let items = sqlite::repository::list("public".to_string(), object.to_string(), 5, 5, "queryStr".to_string(), mapper);
    
    match items {
        Ok(content) => {
            HttpResponse::Ok().json(json!(content))
        }
        Err(err) => {
            eprintln!("{:?}", err);
            let error_string = err.to_string();
            HttpResponse::Ok().json(json!(error_string))
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


pub async fn post<T>(req: HttpRequest, payload: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder 
where
    T: std::fmt::Debug + Deserialize<'static> + Serialize + 'static, // Required traits for T
{
    // get object from path: /api/data/{object}
    let object = get_object_from_path(req.path());
    //Insert into db
    let result = sqlite::repository::insert(
        "public".to_string(), 
        object.to_string(), 
        payload.keys().cloned().collect(), 
        payload.values().cloned().collect()
    );
    println!("Table from db {}", object.to_string());
    match result {
        Ok(content) => {
            println!("Successful insertion. Content: {}", content);
        }
        Err(err) => {
            eprintln!("Error creating object: {}", err);
        }
    }

    let str = format!("Post {}, country {}", "&payload.data.name", "&payload.data.country");
    HttpResponse::Ok().body(str)
}

// PUT
/*
pub async fn put<T>(req: HttpRequest, payload: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder 
where
    T: std::fmt::Debug + Deserialize<'static> + Serialize + 'static, // Required traits for T
*/

pub async fn put<T>(req: HttpRequest, path: web::Path<String>, payload: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder 
where
    T: std::fmt::Debug + Deserialize<'static> + Serialize + 'static, // Required traits for T
{
    let id = path.into_inner();
    let object = get_object_from_path(req.path());

    let result = sqlite::repository::update(
        "public".to_string(), 
        object.to_string(),
        id.clone(),
        payload.keys().cloned().collect(), 
        payload.values().cloned().collect()
    );

    match(result) {
        Ok(content) => {
            println!("Update success {}", content);
        }
        Err(err) => {
            println!("Update error: {}", err);
        }
    }


    let str = format!("Update user_id {}!", id);
    HttpResponse::Ok().body(str)
}

// DELETE
//pub async fn delete(path: web::Path<(u32)>, body: web::Body<_>) -> impl Responder {
pub async fn delete(req: HttpRequest, path: web::Path<u32>) -> impl Responder {
    let id = path.into_inner();
    let object = get_object_from_path(req.path());

    let delete = sqlite::repository::delete("public".to_string(), object.to_string(), id);

    match (delete) {
        Ok(content) => {
            println!("Deleted {}", content);
        }
        Err(err) => { 
            println!("Delete error: {}", err);
        }
    }

    let str = format!("Delete user_id {}!", id);
    HttpResponse::Ok().body(str)
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

pub fn config(cfg: &mut web::ServiceConfig, data: SystemData) {
    // Register migration routes at the data scope level
    cfg.service(
        web::scope("/migrate")
            .route("/list", web::get().to(list_migrations_route))
            .route("/list/", web::get().to(list_migrations_route))
            .route("/all", web::post().to(apply_all_migrations_route))
            .route("/all/", web::post().to(apply_all_migrations_route))
            .route("/{migration_id}", web::post().to(apply_migration_route))
            .route("/{migration_id}/", web::post().to(apply_migration_route))
    );

    match data.public {
        Some(vec) => {
            // 'vec' is a Vec<SystemDataModel> here
            if vec.is_empty() {
                //println!("Vector is present but empty.");
            } 
            else {
                //println!("Setup API for object: {:?}", vec);
                for element in vec {
                    let api_path = format!("/{}", element.name);
                    cfg.service(
                        web::scope(&api_path)
                            .route("/list", web::get().to(list))
                            .route("/{id}", web::get().to(get))
                            .route("/", web::post().to(post::<HashMap<String, serde_json::Value>>))
                            .route("/{id}", web::put().to(put::<HashMap<String, serde_json::Value>>))
                            .route("/{id}", web::delete().to(delete))

                            .route("/list/", web::get().to(list))
                            .route("/{id}/", web::get().to(get))
                            .route("/", web::post().to(post::<HashMap<String, serde_json::Value>>))
                            .route("/{id}/", web::put().to(put::<HashMap<String, serde_json::Value>>))
                            .route("/{id}/", web::delete().to(delete))
                            //.route("/echo", web::post().to(echo))
                    );
                }
            }
        }
        None => {
            // println!("No vector present.");
        }
    } 
    
    /*
    let apiPath = format!("/data/{}", data.db_name.clone());

    cfg.service(
        web::scope(&apiPath)
            .route("/list", web::get().to(list))
            .route("/{id}", web::get().to(get))
            .route("/", web::post().to(post))
            .route("/{id}", web::put().to(put))
            .route("/{id}", web::delete().to(delete))
            //.route("/echo", web::post().to(echo))
    );
    */
}

#[cfg(test)]
#[path = "data.test.rs"]
mod tests;
