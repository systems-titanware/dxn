//use serde::Deserialize;
use actix_web::{ web,  HttpResponse, HttpRequest, Responder}; 
use crate::data::db::sqlite; 
use crate::system::models::{AppState}; 
use crate::data::models::{SystemData, QueryParams}; 
use crate::system::server::models::{SystemServer, SystemServerRoute, FlattenRoutePath};
use crate::integrations;
use rusqlite::{Row, types::ValueRef, Result};
use std::fmt::Debug;
use crate::functions::models::{EnumParamTypes};

use serde::{Deserialize, Serialize};
//If T is needed for trait bounds or methods but not a field: You can use std::marker::PhantomData<T> to explicitly tell the compiler that you are aware of the unused parameter and intend to use it to "act like" the struct owns a T. PhantomData takes up no memory space.
use std::collections::HashMap;
use serde_json::{json, Value, Map};
use handlebars::Handlebars;
use crate::system::server::constants::{GLOBAL_ROUTES_PATH, GLOBAL_BASE_ROUTE_PATH};
use crate::system::files::dxn_files;

fn get_html_404() -> &'static str {
    r#"
        <!DOCTYPE html>
        <html><head><title>DXN</title></head>
        <body><h1>404</h1><p>This page could not be found.</p></body>
        </html>
    "#
}
fn get_html_500() -> &'static str {
    r#"
        <!DOCTYPE html>
        <html><head><title>DXN</title></head>
        <body><h1>500</h1><p>There was an error loading this page.</p></body>
        </html>
    "#
}

fn get_html_500_with_detail(detail: &str) -> String {
    let escaped = detail
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    format!(
        r#"<!DOCTYPE html>
<html><head><title>DXN</title></head>
<body><h1>500</h1><p>There was an error loading this page.</p><pre>{}</pre></body>
</html>"#,
        escaped
    )
}


#[derive(Serialize, Debug)]
struct Person {
    name: String,
    age: u8,
    is_student: bool,
}

// GET
pub async fn get(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Convert recurisve vector into flat hashmap of routes/file paths
    let flattened_routes = flatten_routes(data.system.server.public.clone());
    // Get the path of the requested route (removing '/server/')
    let full_path = req.path().replace(GLOBAL_BASE_ROUTE_PATH, "");
    // Find the matching KeyValue pair in the hashmap where the route path matches full_path
    let route_file = flattened_routes
        .iter()
        .filter(|&x| { 
            return *x.0 == full_path
        })
        .last();

    // Return the contents of the file referenced in the route filepath
    match route_file {
        Some(item) => {
            let route_item = item.1;

            // If route has file, read file
            // If route has function, run function

            // Read route file from startup-resolved dxn-files root.
            let routes_path = format!("{}/{}", GLOBAL_ROUTES_PATH, route_item.file);
            let page_content = match dxn_files::read_under_dxn_files_root(&data.dxn_files_root, &routes_path) {
                Ok(content) => content,
                Err(e) => {
                    let detail = format!(
                        "Failed to load route file.\nrelative path: {}\nproject_root: {}\ndxn_files_root: {}\nerror: {}",
                        routes_path,
                        data.project_root,
                        data.dxn_files_root,
                        e
                    );
                    crate::system::logger::log_error(&detail);
                    return HttpResponse::Ok()
                        .content_type("text/html")
                        .body(get_html_500_with_detail(&detail));
                }
            };

            // Check if layout is specified
            let final_content = if let Some(layout_file) = &route_item.layout {
                let layout_path = format!("{}/{}", GLOBAL_ROUTES_PATH, layout_file);
                let layout_template = match dxn_files::read_under_dxn_files_root(&data.dxn_files_root, &layout_path) {
                    Ok(template) => template,
                    Err(e) => {
                        let detail = format!(
                            "Failed to load layout file.\nrelative path: {}\nproject_root: {}\ndxn_files_root: {}\nerror: {}",
                            layout_path,
                            data.project_root,
                            data.dxn_files_root,
                            e
                        );
                        crate::system::logger::log_error(&detail);
                        return HttpResponse::Ok()
                            .content_type("text/html")
                            .body(get_html_500_with_detail(&detail));
                    }
                };

                // First, do simple string replacement for content to avoid Handlebars parsing Alpine.js
                let layout_with_content = layout_template.replace("{{{content}}}", &page_content);

                // Render template with Handlebars (only for title and other simple variables)
                let mut handlebars = Handlebars::new();
                handlebars.set_strict_mode(false); // Allow missing variables

                // Prepare template context (Handlebars expects serde_json::Value)
                let template_data = json!({
                    "title": route_item.name,
                    "route": {
                        "name": route_item.name,
                        "path": routes_path
                    }
                });

                // Render template (content is already inserted, so Handlebars won't parse Alpine.js)
                match handlebars.render_template(&layout_with_content, &template_data) {
                    Ok(rendered) => rendered,
                    Err(e) => {
                        crate::system::logger::log_error(format!("Template rendering error: {}", e).as_str());
                        page_content
                    }
                }
            } else {
                // No layout specified, return page content directly
                page_content
            };
            
            // Return rendered content
            return HttpResponse::Ok().content_type("text/html").body(final_content);
        },
        None => {
            return HttpResponse::Ok().content_type("text/html").body(get_html_404())
        }
    };
}

pub fn send_object_to_integration() {

    // run preprocessor
    // 2. Create an instance of the struct
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        is_student: true,
    };

    // 3. Serialize to a JSON string
    
    match serde_json::to_string(&person) {
        Ok(serialized_string) => {
            let value = integrations::manager::run("parser", "parse", Some(&serialized_string));
            println!("Server integration response: {:?}", value);
        }
        Err(e) => {
            eprintln!("Error during serialization: {}", e);
        }
    }
}
pub async fn not_found(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(get_html_404())
} 
 
pub fn config(cfg: &mut web::ServiceConfig, data: SystemServer) {
    let flattened_routes: HashMap<String, FlattenRoutePath> = flatten_routes(data.public);
    let actix_routes = convert_routes(flattened_routes);
    
    for service in actix_routes {
         cfg.service(service);
    }
    cfg.service(web::scope("").default_service(web::to(not_found)));
}


pub fn flatten_routes(data: Option<Vec<SystemServerRoute>>) -> HashMap<String, FlattenRoutePath> {
     let mut flattened: HashMap<String, FlattenRoutePath> = HashMap::new();

    match data {
        Some(item) => {
            for route in item {
                recursively_flatten_routes(route, &mut flattened, "");
            }
        },
        None => {}
    }
    
    return flattened;
}


pub fn recursively_flatten_routes(route: SystemServerRoute, map: &mut HashMap<String, FlattenRoutePath>, parent_route: &str) {
    // Insert the current node's data into the map
    let path = if parent_route.is_empty() {
        route.name.clone() 
    } else {
        format!("{}/{}", parent_route, route.name) 
    }; 

    let flattened_route = FlattenRoutePath {
        name: route.name,
        file: route.file.clone(),
        layout: route.layout.clone()
    };
    //map.insert(path.clone(), route.file.clone());

    
    map.insert(path.clone(), flattened_route);

    // Recursively call the method for each child
    match (&route.routes) {
        Some (child_routes) => {
            for child in child_routes {
                recursively_flatten_routes(child.clone(), map, &path);
            }
        },
        None => {}
    }
}
pub(crate) fn convert_routes(mappped_routes: HashMap<String, FlattenRoutePath>) -> Vec<actix_web::Resource> {
    let mut route_map: Vec<actix_web::Resource> = Vec::new();
   
    for child in mappped_routes {
        route_map.push(web::resource(child.0).route(web::get().to(get)));
    }
    route_map
}

#[cfg(test)]
#[path = "server.test.rs"]
mod tests; 