//! Events Controller
//! 
//! Provides API endpoints for querying the event store.
//! These endpoints allow clients to:
//! - View the history of changes to entities
//! - Query events by schema type
//! - Replay aggregate state from events

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::db::sqlite::repository_events;
use crate::data::models::EventQueryParams;
use crate::system::server::models::{ApiResponse, ApiMeta, ApiError};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Response for event list endpoints
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventListResponse {
    pub events: Vec<crate::data::models::Event>,
    pub total: u32,
}

/// Response for replay endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayResponse {
    pub aggregate_id: String,
    pub schema_name: String,
    pub current_state: Option<serde_json::Value>,
    pub event_count: u32,
}

/// Response for rebuild endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuildResponse {
    pub schema_name: String,
    pub records_rebuilt: u32,
    pub events_processed: u32,
    pub deleted_records: Vec<String>,
    pub success: bool,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// GET /api/events/recent
/// 
/// Returns the most recent events across all schemas.
/// Query params: limit (default 50)
pub async fn get_recent_events(
    query: web::Query<EventQueryParams>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50);
    
    match repository_events::get_recent_events(limit) {
        Ok(events) => {
            let total = events.len() as u32;
            HttpResponse::Ok().json(ApiResponse {
                data: events,
                meta: Some(ApiMeta {
                    page: 1,
                    page_size: limit,
                    total,
                    total_pages: 1,
                }),
            })
        }
        Err(err) => {
            eprintln!("Error fetching recent events: {}", err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error".to_string(),
                message: "Failed to fetch recent events".to_string(),
                details: None,
            })
        }
    }
}

/// GET /api/events/aggregate/{aggregate_id}
/// 
/// Returns all events for a specific entity.
pub async fn get_events_by_aggregate(
    path: web::Path<String>,
) -> impl Responder {
    let aggregate_id = path.into_inner();
    
    match repository_events::get_events_by_aggregate(&aggregate_id) {
        Ok(events) => {
            let total = events.len() as u32;
            HttpResponse::Ok().json(EventListResponse {
                events,
                total,
            })
        }
        Err(err) => {
            eprintln!("Error fetching events for aggregate {}: {}", aggregate_id, err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error".to_string(),
                message: "Failed to fetch events".to_string(),
                details: None,
            })
        }
    }
}

/// GET /api/events/schema/{schema_name}
/// 
/// Returns all events for a specific schema type.
/// Supports filtering via query params: since, until, event_type, limit, offset
pub async fn get_events_by_schema(
    path: web::Path<String>,
    query: web::Query<EventQueryParams>,
) -> impl Responder {
    let schema_name = path.into_inner();
    let query_params = query.into_inner();
    
    match repository_events::get_events_by_schema(&schema_name, Some(&query_params)) {
        Ok(events) => {
            let total = match repository_events::count_events_by_schema(&schema_name) {
                Ok(count) => count,
                Err(_) => events.len() as u32,
            };
            
            HttpResponse::Ok().json(json!({
                "schema": schema_name,
                "events": events,
                "total": total,
                "returned": events.len(),
            }))
        }
        Err(err) => {
            eprintln!("Error fetching events for schema {}: {}", schema_name, err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error".to_string(),
                message: "Failed to fetch events".to_string(),
                details: None,
            })
        }
    }
}

/// GET /api/events/{event_id}
/// 
/// Returns a single event by ID.
pub async fn get_event(
    path: web::Path<String>,
) -> impl Responder {
    let event_id = path.into_inner();
    
    match repository_events::get_event_by_id(&event_id) {
        Ok(event) => {
            HttpResponse::Ok().json(event)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            HttpResponse::NotFound().json(ApiError {
                error: "not_found".to_string(),
                message: format!("Event '{}' not found", event_id),
                details: None,
            })
        }
        Err(err) => {
            eprintln!("Error fetching event {}: {}", event_id, err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error".to_string(),
                message: "Failed to fetch event".to_string(),
                details: None,
            })
        }
    }
}

/// GET /api/events/replay/{aggregate_id}
/// 
/// Replays all events for an aggregate to reconstruct its current state.
/// Useful for debugging and verifying data integrity.
pub async fn replay_aggregate(
    path: web::Path<String>,
) -> impl Responder {
    let aggregate_id = path.into_inner();
    
    // Get event count
    let event_count = match repository_events::count_events_by_aggregate(&aggregate_id) {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    if event_count == 0 {
        return HttpResponse::NotFound().json(ApiError {
            error: "not_found".to_string(),
            message: format!("No events found for aggregate '{}'", aggregate_id),
            details: None,
        });
    }
    
    match repository_events::replay_aggregate(&aggregate_id) {
        Ok(state) => {
            // Get schema name from events
            let schema_name = repository_events::get_events_by_aggregate(&aggregate_id)
                .ok()
                .and_then(|events| events.first().map(|e| e.schema_name.clone()))
                .unwrap_or_default();
            
            HttpResponse::Ok().json(ReplayResponse {
                aggregate_id,
                schema_name,
                current_state: state,
                event_count,
            })
        }
        Err(err) => {
            eprintln!("Error replaying aggregate {}: {}", aggregate_id, err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error".to_string(),
                message: "Failed to replay aggregate".to_string(),
                details: None,
            })
        }
    }
}

/// POST /api/events/rebuild/{schema_name}
/// 
/// Rebuilds a schema's data table from events.
/// 
/// **WARNING**: This is a destructive operation that will replace all existing
/// data in the table with state derived from events. Use with caution.
/// 
/// This is useful for:
/// - Recovering from data corruption
/// - Verifying event store integrity
/// - Migrating to event-first architecture
pub async fn rebuild_schema(
    path: web::Path<String>,
) -> impl Responder {
    let schema_name = path.into_inner();
    
    // Check if any events exist for this schema
    let event_count = match repository_events::count_events_by_schema(&schema_name) {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    if event_count == 0 {
        return HttpResponse::NotFound().json(ApiError {
            error: "not_found".to_string(),
            message: format!("No events found for schema '{}'", schema_name),
            details: None,
        });
    }
    
    match repository_events::rebuild_schema_from_events(&schema_name, "public") {
        Ok(result) => {
            HttpResponse::Ok().json(RebuildResponse {
                schema_name,
                records_rebuilt: result.records_rebuilt,
                events_processed: result.events_processed,
                deleted_records: result.deleted_records,
                success: true,
            })
        }
        Err(err) => {
            eprintln!("Error rebuilding schema {}: {}", schema_name, err);
            HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error".to_string(),
                message: format!("Failed to rebuild schema: {}", err),
                details: None,
            })
        }
    }
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

/// Configures the events routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            // Recent events (must come before /{event_id} to avoid conflict)
            .route("/recent", web::get().to(get_recent_events))
            .route("/recent/", web::get().to(get_recent_events))
            
            // Events by aggregate
            .route("/aggregate/{aggregate_id}", web::get().to(get_events_by_aggregate))
            .route("/aggregate/{aggregate_id}/", web::get().to(get_events_by_aggregate))
            
            // Events by schema
            .route("/schema/{schema_name}", web::get().to(get_events_by_schema))
            .route("/schema/{schema_name}/", web::get().to(get_events_by_schema))
            
            // Replay aggregate
            .route("/replay/{aggregate_id}", web::get().to(replay_aggregate))
            .route("/replay/{aggregate_id}/", web::get().to(replay_aggregate))
            
            // Rebuild schema from events (destructive - use POST)
            .route("/rebuild/{schema_name}", web::post().to(rebuild_schema))
            .route("/rebuild/{schema_name}/", web::post().to(rebuild_schema))
            
            // Single event by ID (catch-all, must be last)
            .route("/{event_id}", web::get().to(get_event))
            .route("/{event_id}/", web::get().to(get_event))
    );
}
