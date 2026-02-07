//! Schema Controller
//! 
//! This controller provides API endpoints for managing data schemas at runtime.
//! It supports the hybrid approach where schemas can be created, updated, and
//! deleted via API calls in addition to being defined in config.json.
//! 
//! Soft Delete Support:
//! - DELETE /api/schema/{name} - Soft deletes by default (can be restored)
//! - DELETE /api/schema/{name}?cascade=true - Hard deletes schema AND data
//! - PUT /api/schema/{name}/restore - Restores a soft-deleted schema

use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::data::models::{SystemDataModel, SystemDataModelField, QueryParams, SchemaStatus};
use crate::data::db::sqlite::{repository_schema, repository};
use crate::data::db::models::DbColumn;
use crate::system::server::models::{
    ApiResponse, ApiError, ListResponse, Pagination,
    CreateSchemaRequest, SchemaFieldRequest, UpdateSchemaRequest,
};

// ============================================================================
// QUERY PARAMS
// ============================================================================

/// Query parameters for listing schemas
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaListParams {
    #[serde(flatten)]
    pub pagination: QueryParams,
    /// Include soft-deleted schemas in results
    #[serde(default)]
    pub include_deleted: Option<bool>,
}

/// Query parameters for delete operation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSchemaParams {
    /// If true, permanently deletes schema AND all associated data
    #[serde(default)]
    pub cascade: Option<bool>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// GET /api/schema
/// 
/// Lists all registered schemas with pagination support.
/// 
/// Query params:
/// - `page` - Page number (default: 1)
/// - `page_size` - Items per page (default: 10)
/// - `include_deleted` - If true, includes soft-deleted schemas (default: false)
pub async fn list_schemas(
    query_params: web::Query<SchemaListParams>,
) -> impl Responder {
    let page = query_params.pagination.page.unwrap_or(1) as u32;
    let page_size = query_params.pagination.page_size.unwrap_or(10) as u32;
    let include_deleted = query_params.include_deleted.unwrap_or(false);
    
    match repository_schema::get_all_schemas_with_deleted(include_deleted) {
        Ok(schemas) => {
            let total = schemas.len() as u32;
            let total_pages = if page_size > 0 {
                (total as f64 / page_size as f64).ceil() as u32
            } else {
                0
            };
            let start = ((page - 1) * page_size) as usize;
            let paginated: Vec<_> = schemas.into_iter()
                .skip(start)
                .take(page_size as usize)
                .collect();
            HttpResponse::Ok().json(ListResponse {
                data: paginated,
                pagination: Pagination {
                    page,
                    page_size,
                    total,
                    total_pages,
                },
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to retrieve schemas".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

/// GET /api/schema/{name}
/// 
/// Retrieves a single schema by name.
pub async fn get_schema(
    path: web::Path<String>,
) -> impl Responder {
    let name = path.into_inner();
    
    match repository_schema::get_schema_by_name(&name) {
        Ok(schema) => {
            HttpResponse::Ok().json(ApiResponse {
                data: schema,
                meta: None,
            })
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Schema '{}' not found", name),
                details: None,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to retrieve schema".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

/// POST /api/schema
/// 
/// Creates a new runtime schema. This will:
/// 1. Register the schema in the schema repository
/// 2. Create the actual database table
/// 
/// Returns the created schema with its endpoints.
pub async fn create_schema(
    body: web::Json<CreateSchemaRequest>,
) -> impl Responder {
    let request = body.into_inner();
    
    // Validate schema name
    if request.name.is_empty() {
        return HttpResponse::BadRequest().json(ApiError {
            error: "validation_error".to_string(),
            message: "Schema name is required".to_string(),
            details: None,
        });
    }
    
    // Check if schema already exists
    match repository_schema::schema_exists(&request.name) {
        Ok(true) => {
            return HttpResponse::Conflict().json(ApiError {
                error: "conflict".to_string(),
                message: format!("Schema '{}' already exists", request.name),
                details: None,
            });
        }
        Err(e) => {
            // Table might not exist yet, continue
            eprintln!("Warning checking schema existence: {}", e);
        }
        _ => {}
    }
    
    // Convert request to SystemDataModel
    let schema = SystemDataModel {
        name: request.name.clone(),
        version: 1,
        db: request.db.clone(),
        public: request.public,
        deleted_at: None,
        status: SchemaStatus::Active,
        source: Some("runtime".to_string()),
        icon: request.icon.clone(),
        fields: request.fields.iter().map(|f| SystemDataModelField {
            name: f.name.clone(),
            datatype: f.datatype.clone(),
            value: f.value.clone(),
            primary: f.primary,
            secondary: f.secondary,
        }).collect(),
    };
    
    // Register in schema repository
    if let Err(e) = repository_schema::insert_runtime_schema(&schema) {
        return HttpResponse::InternalServerError().json(ApiError {
            error: "database_error".to_string(),
            message: "Failed to register schema".to_string(),
            details: Some(e.to_string()),
        });
    }
    
    // Create the actual database table
    let columns = build_columns_from_schema(&schema);
    if let Err(e) = repository::create_dynamic_table(
        schema.db.clone(),
        schema.name.clone(),
        columns,
    ) {
        // Rollback: delete from repository
        let _ = repository_schema::delete_schema(&schema.name);
        
        return HttpResponse::InternalServerError().json(ApiError {
            error: "table_creation_error".to_string(),
            message: "Failed to create database table".to_string(),
            details: Some(e.to_string()),
        });
    }
    
    // Return success with schema info and endpoints
    HttpResponse::Created().json(json!({
        "data": {
            "name": schema.name,
            "version": schema.version,
            "db": schema.db,
            "public": schema.public,
            "icon": schema.icon,
            "source": "runtime",
            "fields": schema.fields,
            "endpoints": {
                "list": format!("/api/data/{}", schema.name),
                "get": format!("/api/data/{}/{{id}}", schema.name),
                "create": format!("/api/data/{}", schema.name),
                "update": format!("/api/data/{}/{{id}}", schema.name),
                "delete": format!("/api/data/{}/{{id}}", schema.name)
            }
        }
    }))
}

/// PUT /api/schema/{name}
/// 
/// Updates an existing schema. Only runtime schemas can be updated.
/// Config-sourced schemas are managed through config.json.
pub async fn update_schema(
    path: web::Path<String>,
    body: web::Json<UpdateSchemaRequest>,
) -> impl Responder {
    let name = path.into_inner();
    let request = body.into_inner();
    
    // Get existing schema
    let existing = match repository_schema::get_schema_by_name(&name) {
        Ok(s) => s,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Schema '{}' not found", name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to retrieve schema".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Check if it's a config schema (can't be modified via API)
    if existing.source == Some("config".to_string()) {
        return HttpResponse::Forbidden().json(ApiError {
            error: "forbidden".to_string(),
            message: "Config-sourced schemas cannot be modified via API. Update config.json instead.".to_string(),
            details: None,
        });
    }
    
    // Build updated schema
    let updated_schema = SystemDataModel {
        name: existing.name.clone(),
        version: existing.version + 1,
        db: request.db.unwrap_or(existing.db),
        deleted_at: None,
        status: existing.status,
        public: request.public.unwrap_or(existing.public),
        source: existing.source,
        icon: request.icon.or(existing.icon),
        fields: request.fields.map(|f| {
            f.into_iter().map(|field| SystemDataModelField {
                name: field.name,
                datatype: field.datatype,
                value: field.value,
                primary: field.primary,
                secondary: field.secondary,
            }).collect()
        }).unwrap_or(existing.fields),
    };
    
    // Update in repository
    if let Err(e) = repository_schema::update_schema(&name, &updated_schema) {
        return HttpResponse::InternalServerError().json(ApiError {
            error: "database_error".to_string(),
            message: "Failed to update schema".to_string(),
            details: Some(e.to_string()),
        });
    }
    
    HttpResponse::Ok().json(ApiResponse {
        data: updated_schema,
        meta: None,
    })
}

/// DELETE /api/schema/{name}
/// 
/// Deletes a runtime schema. Config-sourced schemas cannot be deleted via API.
/// 
/// Query params:
/// - `cascade=false` (default): Soft delete - schema can be restored, data preserved
/// - `cascade=true`: Hard delete - permanently removes schema AND drops data table
pub async fn delete_schema(
    path: web::Path<String>,
    query_params: web::Query<DeleteSchemaParams>,
) -> impl Responder {
    let name = path.into_inner();
    let cascade = query_params.cascade.unwrap_or(false);
    
    // Get existing schema to check source (include deleted for cascade operations)
    let existing = match repository_schema::get_schema_by_name_include_deleted(&name) {
        Ok(s) => s,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Schema '{}' not found", name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to retrieve schema".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Check if it's a config schema
    if existing.source == Some("config".to_string()) {
        return HttpResponse::Forbidden().json(ApiError {
            error: "forbidden".to_string(),
            message: "Config-sourced schemas cannot be deleted via API. Remove from config.json instead.".to_string(),
            details: None,
        });
    }
    
    // Check if already deleted (for non-cascade)
    if !cascade && existing.status == SchemaStatus::Deleted {
        return HttpResponse::Conflict().json(ApiError {
            error: "already_deleted".to_string(),
            message: format!("Schema '{}' is already deleted. Use cascade=true for permanent deletion or restore it first.", name),
            details: None,
        });
    }
    
    if cascade {
        // Hard delete: remove schema AND drop the data table
        let db_name = existing.db.clone();
        
        // First, drop the data table
        if let Err(e) = repository::drop_table(db_name.clone(), name.clone()) {
            eprintln!("Warning: Failed to drop table '{}': {}", name, e);
            // Continue anyway - table might not exist
        }
        
        // Then, permanently delete from schema repository
        match repository_schema::hard_delete_schema(&name) {
            Ok(rows) if rows > 0 => {
                HttpResponse::Ok().json(json!({
                    "data": {
                        "deleted": true,
                        "name": name,
                        "cascade": true,
                        "table_dropped": true,
                        "permanent": true
                    }
                }))
            }
            Ok(_) => {
                HttpResponse::NotFound().json(ApiError {
                    error: "not_found".to_string(),
                    message: format!("Schema '{}' not found", name),
                    details: None,
                })
            }
            Err(e) => {
                HttpResponse::InternalServerError().json(ApiError {
                    error: "database_error".to_string(),
                    message: "Failed to delete schema".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    } else {
        // Soft delete: mark as deleted, preserve data
        match repository_schema::soft_delete_schema(&name) {
            Ok(rows) if rows > 0 => {
                HttpResponse::Ok().json(json!({
                    "data": {
                        "deleted": true,
                        "name": name,
                        "cascade": false,
                        "restorable": true,
                        "note": "Schema soft-deleted. Data preserved. Use PUT /api/schema/{name}/restore to restore."
                    }
                }))
            }
            Ok(_) => {
                HttpResponse::NotFound().json(ApiError {
                    error: "not_found".to_string(),
                    message: format!("Schema '{}' not found or already deleted", name),
                    details: None,
                })
            }
            Err(e) => {
                HttpResponse::InternalServerError().json(ApiError {
                    error: "database_error".to_string(),
                    message: "Failed to delete schema".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }
}

/// PUT /api/schema/{name}/restore
/// 
/// Restores a soft-deleted schema. Only works on schemas with status='deleted'.
pub async fn restore_schema(
    path: web::Path<String>,
) -> impl Responder {
    let name = path.into_inner();
    
    // Check if schema exists and is deleted
    match repository_schema::get_schema_by_name_include_deleted(&name) {
        Ok(schema) => {
            if schema.status != SchemaStatus::Deleted {
                return HttpResponse::Conflict().json(ApiError {
                    error: "not_deleted".to_string(),
                    message: format!("Schema '{}' is not deleted and cannot be restored", name),
                    details: None,
                });
            }
            
            // Check if it's a config schema (shouldn't happen but just in case)
            if schema.source == Some("config".to_string()) {
                return HttpResponse::Forbidden().json(ApiError {
                    error: "forbidden".to_string(),
                    message: "Config-sourced schemas cannot be restored via API.".to_string(),
                    details: None,
                });
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Schema '{}' not found", name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to retrieve schema".to_string(),
                details: Some(e.to_string()),
            });
        }
    }
    
    // Restore the schema
    match repository_schema::restore_schema(&name) {
        Ok(rows) if rows > 0 => {
            // Get the restored schema to return
            match repository_schema::get_schema_by_name(&name) {
                Ok(restored) => {
                    HttpResponse::Ok().json(json!({
                        "data": {
                            "restored": true,
                            "schema": restored
                        }
                    }))
                }
                Err(_) => {
                    HttpResponse::Ok().json(json!({
                        "data": {
                            "restored": true,
                            "name": name
                        }
                    }))
                }
            }
        }
        Ok(_) => {
            HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Schema '{}' not found or not in deleted state", name),
                details: None,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to restore schema".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Converts a SystemDataModel to a vector of DbColumn definitions
fn build_columns_from_schema(schema: &SystemDataModel) -> Vec<DbColumn> {
    let mut columns: Vec<DbColumn> = Vec::new();
    
    // Find primary key field
    let primary_field = schema.fields.iter()
        .find(|f| f.primary == Some(true));
    
    match primary_field {
        Some(field) => {
            columns.push(repository::create_col_primary(
                field.name.clone(),
                map_datatype(&field.datatype),
            ));
        }
        None => {
            // Add default id primary key with TEXT type for UUID v7 support
            columns.push(DbColumn {
                name: "id".to_string(),
                col_type: "TEXT".to_string(),
                value: None,
                primary_key: Some(true),
                secondary_key: None,
                nullable: false,
                unique: None,
                default: None,
                autoincrement: None, // UUIDs don't autoincrement
                check: None,
            });
        }
    }
    
    // Add non-primary fields
    for field in &schema.fields {
        if field.primary != Some(true) {
            columns.push(repository::create_col(
                field.name.clone(),
                map_datatype(&field.datatype),
                true, // nullable by default
            ));
        }
    }
    
    columns
}

/// Maps common datatype names to SQLite types
fn map_datatype(datatype: &str) -> String {
    match datatype.to_lowercase().as_str() {
        "string" | "text" | "varchar" => "TEXT".to_string(),
        "int" | "integer" | "number" => "INTEGER".to_string(),
        "float" | "double" | "real" | "decimal" => "REAL".to_string(),
        "bool" | "boolean" => "INTEGER".to_string(), // SQLite uses 0/1
        "blob" | "binary" => "BLOB".to_string(),
        "json" => "TEXT".to_string(), // Store JSON as text
        "date" | "datetime" | "timestamp" => "TEXT".to_string(), // ISO8601 strings
        other => other.to_uppercase(), // Pass through as-is
    }
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

/// Configures routes for the schema controller
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route("", web::get().to(list_schemas))
        .route("", web::post().to(create_schema))
        .route("/{name}", web::get().to(get_schema))
        .route("/{name}", web::put().to(update_schema))
        .route("/{name}", web::delete().to(delete_schema))
        .route("/{name}/restore", web::put().to(restore_schema));
}
