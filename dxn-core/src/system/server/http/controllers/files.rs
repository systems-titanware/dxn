//! Files Controller
//! 
//! Provides API endpoints for file directory management and file operations.
//! Supports multiple storage providers (local, SFTP, S3, etc.)

use actix_web::{web, HttpResponse, HttpRequest, Responder};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::data::db::sqlite::repository_files;
use crate::data::models::{SystemFileDirectory, FileEntry, QueryParams};
use crate::system::server::models::{ApiResponse, ApiError, ListResponse, Pagination};
use crate::system::files::providers::{ProviderRegistry, ProviderError, local::LocalFileProvider};
use crate::system::models::AppState;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Request body for creating a new directory configuration
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirectoryRequest {
    pub name: String,
    #[serde(default = "default_provider")]
    pub provider: String,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

fn default_provider() -> String {
    "local".to_string()
}

/// Request body for updating a directory configuration
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDirectoryRequest {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// DTO for file list responses: domain FileEntry plus full URL for client access.
/// Keeps the core FileEntry unchanged so directory moves don't require updating stored data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntryDto {
    #[serde(flatten)]
    pub entry: FileEntry,
    /// Full URL (scheme + host + path) so the client can access this file or list this directory.
    pub full_path: String,
}

/// Response for listing files inside a directory (data + pagination + context).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListResponse {
    pub data: Vec<FileEntryDto>,
    pub pagination: Pagination,
    pub directory: String,
    pub path: String,
}

/// Query parameters for listing files via GET /api/files
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListQuery {
    /// Directory name (must match SystemFileDirectory.name)
    pub directory: String,
    /// Optional subpath inside the directory
    #[serde(default)]
    pub path: Option<String>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Build base URL (scheme + host) from the request for client-usable full_path values.
fn base_url_from_request(req: &HttpRequest) -> String {
    let conn = req.connection_info();
    format!("{}://{}", conn.scheme(), conn.host())
}

/// Map a FileEntry to FileEntryDto with full_path set from the current request.
fn entry_to_dto(entry: FileEntry, dir_name: &str, base_url: &str) -> FileEntryDto {
    let path_segment = entry.path.trim_start_matches('/');
    let full_path = if entry.is_directory {
        format!(
            "{}/api/files/{}/list/{}",
            base_url.trim_end_matches('/'),
            dir_name,
            path_segment
        )
    } else {
        format!(
            "{}/api/files/{}/read/{}",
            base_url.trim_end_matches('/'),
            dir_name,
            path_segment
        )
    };
    FileEntryDto {
        entry,
        full_path,
    }
}

/// Get or create a provider for the given directory configuration.
/// For the local provider, the directory base path must be inside `project_root`.
fn get_provider_for_directory(
    directory: &SystemFileDirectory,
    project_root: &str,
) -> Result<Box<dyn crate::system::files::providers::FileProvider>, ProviderError> {
    match directory.provider.as_str() {
        "local" => {
            // Keep all file storage under project root: project_root/dxn-files/<directory.path>
            let base_path = format!("{}/dxn-files{}", project_root, directory.path);
            let provider = LocalFileProvider::new_with_project_root(&base_path, project_root)?;
            Ok(Box::new(provider))
        }
        other => Err(ProviderError::ProviderUnavailable(
            format!("Unknown provider type: {}", other),
        )),
    }
}

/// Convert ProviderError to HTTP response
fn provider_error_response(err: ProviderError) -> HttpResponse {
    match err {
        ProviderError::NotFound(path) => {
            HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Path not found: {}", path),
                details: None,
            })
        }
        ProviderError::PermissionDenied(path) => {
            HttpResponse::Forbidden().json(ApiError {
                error: "permission_denied".to_string(),
                message: format!("Permission denied: {}", path),
                details: None,
            })
        }
        ProviderError::AlreadyExists(path) => {
            HttpResponse::Conflict().json(ApiError {
                error: "already_exists".to_string(),
                message: format!("Already exists: {}", path),
                details: None,
            })
        }
        ProviderError::InvalidPath(msg) => {
            HttpResponse::BadRequest().json(ApiError {
                error: "invalid_path".to_string(),
                message: msg,
                details: None,
            })
        }
        ProviderError::ProviderUnavailable(name) => {
            HttpResponse::ServiceUnavailable().json(ApiError {
                error: "provider_unavailable".to_string(),
                message: format!("Provider unavailable: {}", name),
                details: None,
            })
        }
        ProviderError::IoError(e) => {
            HttpResponse::InternalServerError().json(ApiError {
                error: "io_error".to_string(),
                message: format!("IO error: {}", e),
                details: None,
            })
        }
        ProviderError::Other(msg) => {
            HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error".to_string(),
                message: msg,
                details: None,
            })
        }
    }
}

// ============================================================================
// DIRECTORY CONFIGURATION HANDLERS
// ============================================================================

/// GET /api/files
/// 
/// Lists all configured file directories with pagination support.
pub async fn list_directories(
    query_params: web::Query<QueryParams>,
) -> impl Responder {
    let page = query_params.page.unwrap_or(1) as u32;
    let page_size = query_params.page_size.unwrap_or(20) as u32;
    
    match repository_files::get_all_directories() {
        Ok(directories) => {
            let total = directories.len() as u32;
            let total_pages = if page_size > 0 {
                ((total as f64) / (page_size as f64)).ceil() as u32
            } else {
                0
            };
            let start = ((page - 1) * page_size) as usize;
            let paginated: Vec<_> = directories.into_iter()
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
        Err(err) => {
            eprintln!("Error listing directories: {}", err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to list directories".to_string(),
                details: None,
            })
        }
    }
}

/// GET /api/files/{name}
/// 
/// Get a specific directory configuration by name.
pub async fn get_directory(
    path: web::Path<String>,
) -> impl Responder {
    let name = path.into_inner();
    
    match repository_files::get_directory_by_name(&name) {
        Ok(directory) => {
            HttpResponse::Ok().json(ApiResponse {
                data: directory,
                meta: None,
            })
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", name),
                details: None,
            })
        }
        Err(err) => {
            eprintln!("Error getting directory {}: {}", name, err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: None,
            })
        }
    }
}

/// POST /api/files
/// 
/// Create a new directory configuration.
pub async fn create_directory(
    data: web::Data<AppState>,
    body: web::Json<CreateDirectoryRequest>,
) -> impl Responder {
    let request = body.into_inner();
    
    // Validate name
    if request.name.is_empty() {
        return HttpResponse::BadRequest().json(ApiError {
            error: "validation_error".to_string(),
            message: "Directory name is required".to_string(),
            details: None,
        });
    }
    
    // Check if already exists
    match repository_files::directory_exists(&request.name) {
        Ok(true) => {
            return HttpResponse::Conflict().json(ApiError {
                error: "conflict".to_string(),
                message: format!("Directory '{}' already exists", request.name),
                details: None,
            });
        }
        Err(e) => {
            eprintln!("Error checking directory existence: {}", e);
        }
        _ => {}
    }
    
    let directory = SystemFileDirectory {
        name: request.name.clone(),
        provider: request.provider,
        path: request.path,
        icon: request.icon,
        source: Some("runtime".to_string()),
        config: request.config,
    };
    
    // Create the actual directory on the filesystem if using local provider
    if directory.provider == "local" {
        let provider = match get_provider_for_directory(&directory, &data.project_root) {
            Ok(p) => p,
            Err(e) => return provider_error_response(e),
        };
        
        if let Err(e) = provider.mkdir("") {
            // Ignore already exists errors
            match e {
                ProviderError::AlreadyExists(_) => {}
                _ => return provider_error_response(e),
            }
        }
    }
    
    match repository_files::insert_runtime_directory(&directory) {
        Ok(_) => {
            HttpResponse::Created().json(json!({
                "data": directory,
                "endpoints": {
                    "list": format!("/api/files/{}/list", directory.name),
                    "read": format!("/api/files/{}/read/{{path}}", directory.name),
                    "write": format!("/api/files/{}/write/{{path}}", directory.name),
                    "delete": format!("/api/files/{}/delete/{{path}}", directory.name),
                    "mkdir": format!("/api/files/{}/mkdir/{{path}}", directory.name),
                }
            }))
        }
        Err(err) => {
            eprintln!("Error creating directory: {}", err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to create directory".to_string(),
                details: Some(err.to_string()),
            })
        }
    }
}

/// PUT /api/files/{name}
/// 
/// Update a directory configuration. Config-sourced directories cannot be modified.
pub async fn update_directory(
    path: web::Path<String>,
    body: web::Json<UpdateDirectoryRequest>,
) -> impl Responder {
    let name = path.into_inner();
    let request = body.into_inner();
    
    // Get existing directory
    let existing = match repository_files::get_directory_by_name(&name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Check if config-sourced
    if existing.source == Some("config".to_string()) {
        return HttpResponse::Forbidden().json(ApiError {
            error: "forbidden".to_string(),
            message: "Config-sourced directories cannot be modified via API. Update config.json instead.".to_string(),
            details: None,
        });
    }
    
    let updated = SystemFileDirectory {
        name: existing.name.clone(),
        provider: request.provider.unwrap_or(existing.provider),
        path: request.path.unwrap_or(existing.path),
        icon: request.icon.or(existing.icon),
        source: existing.source,
        config: request.config.or(existing.config),
    };
    
    match repository_files::update_directory(&name, &updated) {
        Ok(_) => {
            HttpResponse::Ok().json(ApiResponse {
                data: updated,
                meta: None,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to update directory".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

/// DELETE /api/files/{name}
/// 
/// Delete a directory configuration. Config-sourced directories cannot be deleted.
/// Note: This does NOT delete the actual files on disk.
pub async fn delete_directory(
    path: web::Path<String>,
) -> impl Responder {
    let name = path.into_inner();
    
    // Get existing to check source
    match repository_files::get_directory_by_name(&name) {
        Ok(existing) => {
            if existing.source == Some("config".to_string()) {
                return HttpResponse::Forbidden().json(ApiError {
                    error: "forbidden".to_string(),
                    message: "Config-sourced directories cannot be deleted via API".to_string(),
                    details: None,
                });
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    }
    
    match repository_files::delete_directory(&name) {
        Ok(rows) if rows > 0 => {
            HttpResponse::Ok().json(json!({
                "deleted": true,
                "name": name
            }))
        }
        Ok(_) => {
            HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", name),
                details: None,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to delete directory".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// ============================================================================
// FILE OPERATION HANDLERS
// ============================================================================

/// GET /api/files/{name}/list
/// GET /api/files/{name}/list/{path:.*}
/// 
/// List files in a directory or subdirectory.
/// Each entry includes full_path (scheme + host + API path) for client access.
pub async fn list_files(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<(String, Option<String>)>,
) -> impl Responder {
    let (dir_name, sub_path) = path.into_inner();
    let file_path = sub_path.unwrap_or_default();
    let base_url = base_url_from_request(&req);

    // Get directory configuration
    let directory = match repository_files::get_directory_by_name(&dir_name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", dir_name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };

    // Get provider
    let provider = match get_provider_for_directory(&directory, &data.project_root) {
        Ok(p) => p,
        Err(e) => return provider_error_response(e),
    };

    // List files (single page: all entries; pagination for consistent shape)
    match provider.list(&file_path) {
        Ok(entries) => {
            let total = entries.len() as u32;
            let total_pages = if total > 0 { 1 } else { 0 };
            let data: Vec<FileEntryDto> = entries
                .into_iter()
                .map(|e| entry_to_dto(e, &dir_name, &base_url))
                .collect();
            HttpResponse::Ok().json(FileListResponse {
                data,
                pagination: Pagination {
                    page: 1,
                    page_size: total,
                    total,
                    total_pages,
                },
                directory: dir_name,
                path: file_path.clone(),
            })
        }
        Err(e) => provider_error_response(e),
    }
}

/// GET /api/files?directory={name}&path={optional/path}
///
/// Convenience wrapper around `list_files` using query parameters instead of path params.
pub async fn list_files_query(
    req: HttpRequest,
    data: web::Data<AppState>,
    query: web::Query<FileListQuery>,
) -> impl Responder {
    let dir_name = query.directory.clone();
    let sub_path = query.path.clone();
    list_files(req, data, web::Path::from((dir_name, sub_path))).await
}

/// GET /api/files/{name}/read/{path:.*}
/// 
/// Read file contents.
pub async fn read_file(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (dir_name, file_path) = path.into_inner();
    
    // Get directory configuration
    let directory = match repository_files::get_directory_by_name(&dir_name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", dir_name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Get provider
    let provider = match get_provider_for_directory(&directory, &data.project_root) {
        Ok(p) => p,
        Err(e) => return provider_error_response(e),
    };
    
    // Read file
    match provider.read(&file_path) {
        Ok(contents) => {
            // Get metadata for mime type
            let mime_type = provider.metadata(&file_path)
                .ok()
                .and_then(|m| m.mime_type)
                .unwrap_or_else(|| "application/octet-stream".to_string());
            
            HttpResponse::Ok()
                .content_type(mime_type)
                .body(contents)
        }
        Err(e) => provider_error_response(e),
    }
}

/// POST /api/files/{name}/write/{path:.*}
/// 
/// Write file contents.
pub async fn write_file(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
    body: web::Bytes,
) -> impl Responder {
    let (dir_name, file_path) = path.into_inner();
    
    // Get directory configuration
    let directory = match repository_files::get_directory_by_name(&dir_name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", dir_name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Get provider
    let provider = match get_provider_for_directory(&directory, &data.project_root) {
        Ok(p) => p,
        Err(e) => return provider_error_response(e),
    };
    
    // Write file
    match provider.write(&file_path, &body) {
        Ok(_) => {
            HttpResponse::Ok().json(json!({
                "success": true,
                "path": file_path,
                "size": body.len()
            }))
        }
        Err(e) => provider_error_response(e),
    }
}

/// DELETE /api/files/{name}/delete/{path:.*}
/// 
/// Delete a file or empty directory.
pub async fn delete_file(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (dir_name, file_path) = path.into_inner();
    
    // Get directory configuration
    let directory = match repository_files::get_directory_by_name(&dir_name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", dir_name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Get provider
    let provider = match get_provider_for_directory(&directory, &data.project_root) {
        Ok(p) => p,
        Err(e) => return provider_error_response(e),
    };
    
    // Delete file
    match provider.delete(&file_path) {
        Ok(_) => {
            HttpResponse::Ok().json(json!({
                "deleted": true,
                "path": file_path
            }))
        }
        Err(e) => provider_error_response(e),
    }
}

/// POST /api/files/{name}/mkdir/{path:.*}
/// 
/// Create a directory.
pub async fn mkdir(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (dir_name, dir_path) = path.into_inner();
    
    // Get directory configuration
    let directory = match repository_files::get_directory_by_name(&dir_name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", dir_name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Get provider
    let provider = match get_provider_for_directory(&directory, &data.project_root) {
        Ok(p) => p,
        Err(e) => return provider_error_response(e),
    };
    
    // Create directory
    match provider.mkdir(&dir_path) {
        Ok(_) => {
            HttpResponse::Created().json(json!({
                "created": true,
                "path": dir_path
            }))
        }
        Err(e) => provider_error_response(e),
    }
}

/// GET /api/files/{name}/metadata/{path:.*}
/// 
/// Get file or directory metadata.
pub async fn get_metadata(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (dir_name, file_path) = path.into_inner();
    
    // Get directory configuration
    let directory = match repository_files::get_directory_by_name(&dir_name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", dir_name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };
    
    // Get provider
    let provider = match get_provider_for_directory(&directory, &data.project_root) {
        Ok(p) => p,
        Err(e) => return provider_error_response(e),
    };
    
    // Get metadata
    match provider.metadata(&file_path) {
        Ok(entry) => {
            HttpResponse::Ok().json(ApiResponse {
                data: entry,
                meta: None,
            })
        }
        Err(e) => provider_error_response(e),
    }
}

// ============================================================================
// MULTIPART UPLOAD HANDLER
// ============================================================================

/// POST /api/files/upload
/// 
/// Upload a file via multipart/form-data.
/// 
/// Form fields:
/// - `directory`: The directory name (required)
/// - `path`: Optional subdirectory path (e.g., "images/photo.png")
/// - `file`: The file blob (required)
/// 
/// If `path` is not provided, the filename from the `file` field will be used.
pub async fn upload_file_multipart(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> impl Responder {
    let mut directory: Option<String> = None;
    let mut file_path: Option<String> = None;
    let mut file_bytes = BytesMut::new();
    let mut filename: Option<String> = None;

    // Parse multipart form data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let field_name = field.name().unwrap_or_default().to_string();

        if field_name == "file" {
            // Extract filename if available
            if let Some(content_disposition) = field.content_disposition() {
                if let Some(name) = content_disposition.get_filename() {
                    filename = Some(name.to_string());
                }
            }

            // Stream file bytes
            while let Ok(Some(chunk)) = field.try_next().await {
                file_bytes.extend_from_slice(&chunk);
            }
        } else if field_name == "directory" {
            // Read directory field
            let mut value = String::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                if let Ok(s) = std::str::from_utf8(&chunk) {
                    value.push_str(s);
                }
            }
            if !value.is_empty() {
                directory = Some(value);
            }
        } else if field_name == "path" {
            // Read path field
            let mut value = String::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                if let Ok(s) = std::str::from_utf8(&chunk) {
                    value.push_str(s);
                }
            }
            if !value.is_empty() {
                file_path = Some(value);
            }
        }
    }

    // Validate required fields
    if file_bytes.is_empty() {
        return HttpResponse::BadRequest().json(ApiError {
            error: "validation_error".to_string(),
            message: "No file uploaded (missing 'file' field)".to_string(),
            details: None,
        });
    }

    let dir_name = match directory {
        Some(d) if !d.is_empty() => d,
        _ => {
            return HttpResponse::BadRequest().json(ApiError {
                error: "validation_error".to_string(),
                message: "Missing 'directory' field".to_string(),
                details: None,
            });
        }
    };

    // Determine file path: always use the filename (no extra subpath)
    let final_path = match &filename {
        Some(f) if !f.is_empty() => f.clone(),
        _ => {
            return HttpResponse::BadRequest().json(ApiError {
                error: "validation_error".to_string(),
                message: "No filename available on uploaded file".to_string(),
                details: None,
            });
        }
    };

    // Get directory configuration
    let directory_config = match repository_files::get_directory_by_name(&dir_name) {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Directory '{}' not found", dir_name),
                details: None,
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error".to_string(),
                message: "Failed to get directory".to_string(),
                details: Some(e.to_string()),
            });
        }
    };

    // Get provider (must be inside project root)
    let provider = match get_provider_for_directory(&directory_config, &data.project_root) {
        Ok(p) => p,
        Err(e) => return provider_error_response(e),
    };

    // Write file
    let body = file_bytes.freeze();
    match provider.write(&final_path, &body) {
        Ok(_) => {
            HttpResponse::Ok().json(json!({
                "success": true,
                "directory": dir_name,
                "path": final_path,
                "size": body.len(),
                "filename": filename.clone().unwrap_or_default()
            }))
        }
        Err(e) => provider_error_response(e),
    }
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

/// Configures the files routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            // Directory configuration endpoints
            .route("/directories", web::get().to(list_directories))
            .route("/directories/", web::get().to(list_directories))
            .route("/directories", web::post().to(create_directory))
            .route("/directories/", web::post().to(create_directory))
            
            // File listing via query (directory + optional path)
            .route("", web::get().to(list_files_query))
            .route("/", web::get().to(list_files_query))
            
            // Multipart upload endpoint (must come before /{name} routes)
            .route("/upload", web::post().to(upload_file_multipart))
            .route("/upload/", web::post().to(upload_file_multipart))
            
            // File operation endpoints (must come before /{name} to avoid conflicts)
            .route("/{name}/list", web::get().to(list_files_root))
            .route("/{name}/list/", web::get().to(list_files_root))
            .route("/{name}/list/{path:.*}", web::get().to(list_files_path))
            
            .route("/{name}/read/{path:.*}", web::get().to(read_file))
            .route("/{name}/write/{path:.*}", web::post().to(write_file))
            .route("/{name}/delete/{path:.*}", web::delete().to(delete_file))
            .route("/{name}/mkdir/{path:.*}", web::post().to(mkdir))
            .route("/{name}/metadata/{path:.*}", web::get().to(get_metadata))
            
            // Single directory config endpoints (must be last)
            .route("/{name}", web::get().to(get_directory))
            .route("/{name}/", web::get().to(get_directory))
            .route("/{name}", web::put().to(update_directory))
            .route("/{name}/", web::put().to(update_directory))
            .route("/{name}", web::delete().to(delete_directory))
            .route("/{name}/", web::delete().to(delete_directory))
    );
}

/// Helper handler for listing files at root
async fn list_files_root(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let dir_name = path.into_inner();
    list_files(req, data, web::Path::from((dir_name, None))).await
}

/// Helper handler for listing files at a specific path
async fn list_files_path(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (dir_name, sub_path) = path.into_inner();
    list_files(req, data, web::Path::from((dir_name, Some(sub_path)))).await
}
