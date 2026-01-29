use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::system::models::AppState;
use crate::data::models::QueryParams;
use crate::system::server::http::controllers::server::flatten_routes;
use crate::system::server::models::FlattenRoutePath;
use crate::system::server::constants::GLOBAL_BASE_ROUTE_PATH;
use serde::{Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ServerRouteConfigDto {
    name: String,
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout: Option<String>,
    full_path: String,
    url: String,
}

fn build_server_route_dtos(
    req: &HttpRequest,
    flattened: &HashMap<String, FlattenRoutePath>,
) -> Vec<ServerRouteConfigDto> {
    let conn = req.connection_info();
    let scheme = conn.scheme();
    let host = conn.host();
    let base_url = format!("{}://{}", scheme, host);

    let base_route = GLOBAL_BASE_ROUTE_PATH.trim_end_matches('/');

    let mut dtos: Vec<ServerRouteConfigDto> = flattened
        .iter()
        .map(|(full_path, route)| {
            let url = format!(
                "{base}{route}/{path}",
                base = base_url.trim_end_matches('/'),
                route = base_route,
                path = full_path
            );

            ServerRouteConfigDto {
                name: route.name.clone(),
                file: route.file.clone(),
                layout: route.layout.clone(),
                full_path: full_path.clone(),
                url,
            }
        })
        .collect();

    dtos.sort_by(|a, b| a.full_path.cmp(&b.full_path));
    dtos
}

/// GET handler for listing public function models
/// 
/// Route: /api/config/functions/
/// Query params: page_size (optional), page (optional)
/// 
/// Returns: Paginated list of public function models
pub async fn get_functions(
    req: HttpRequest,
    query_params: web::Query<QueryParams>,
    data: web::Data<AppState>,
) -> impl Responder {
    let page_size = query_params.page_size.unwrap_or(10);
    let page = query_params.page.unwrap_or(1);
    
    let functions = match &data.system.functions.public {
        Some(vec) => vec.clone(),
        None => Vec::new(),
    };
    
    // Calculate pagination
    let start = ((page - 1) as usize) * (page_size as usize);
    
    let paginated_functions: Vec<_> = functions
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();
    
    let total = data.system.functions.public.as_ref()
        .map(|v| v.len())
        .unwrap_or(0);
    
    HttpResponse::Ok().json(json!({
        "data": paginated_functions,
        "pagination": {
            "page": page,
            "page_size": page_size,
            "total": total,
            "total_pages": (total as f64 / page_size as f64).ceil() as u32
        }
    }))
}

/// GET handler for listing public data models
/// 
/// Route: /api/config/data/
/// Query params: page_size (optional), page (optional)
/// 
/// Returns: Paginated list of public data models
pub async fn get_data(
    req: HttpRequest,
    query_params: web::Query<QueryParams>,
    data: web::Data<AppState>,
) -> impl Responder {
    let page_size = query_params.page_size.unwrap_or(10);
    let page = query_params.page.unwrap_or(1);
    
    let data_models = match &data.system.data.public {
        Some(vec) => vec.clone(),
        None => Vec::new(),
    };
    
    // Calculate pagination
    let start = ((page - 1) as usize) * (page_size as usize);
    
    let paginated_data: Vec<_> = data_models
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();
    
    let total = data.system.data.public.as_ref()
        .map(|v| v.len())
        .unwrap_or(0);
    
    HttpResponse::Ok().json(json!({
        "data": paginated_data,
        "pagination": {
            "page": page,
            "page_size": page_size,
            "total": total,
            "total_pages": (total as f64 / page_size as f64).ceil() as u32
        }
    }))
}

/// GET handler for listing public server routes
/// 
/// Route: /api/config/server/
/// Query params: page_size (optional), page (optional)
/// 
/// Returns: Paginated list of public server routes
pub async fn get_server_routes(
    req: HttpRequest,
    query_params: web::Query<QueryParams>,
    data: web::Data<AppState>,
) -> impl Responder {
    let page_size = query_params.page_size.unwrap_or(10) as usize;
    let page = query_params.page.unwrap_or(1) as usize;

    // 1) Flatten routes using existing server logic
    let flattened = flatten_routes(data.system.server.public.clone());

    // 2) Build DTOs with full_path and url
    let dtos = build_server_route_dtos(&req, &flattened);

    let total = dtos.len();

    // 3) Apply existing pagination semantics on DTO list
    let start = (page.saturating_sub(1)) * page_size;
    let paginated_routes: Vec<ServerRouteConfigDto> = dtos
        .into_iter()
        .skip(start)
        .take(page_size)
        .collect();

    HttpResponse::Ok().json(json!({
        "data": paginated_routes,
        "pagination": {
            "page": page,
            "page_size": page_size,
            "total": total,
            "total_pages": (total as f64 / page_size as f64).ceil() as u32
        }
    }))
}

/// Configure config routes
/// 
/// Sets up GET routes for accessing config data
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/functions", web::get().to(get_functions))
       .route("/functions/", web::get().to(get_functions))
       .route("/data", web::get().to(get_data))
       .route("/data/", web::get().to(get_data))
       .route("/server", web::get().to(get_server_routes))
       .route("/server/", web::get().to(get_server_routes));
}

#[cfg(test)]
#[path = "config.test.rs"]
mod tests;
