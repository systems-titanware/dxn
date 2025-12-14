//use serde::Deserialize;
use actix_web::{ web,  HttpResponse, HttpRequest, Responder}; 
use crate::data::db::sqlite; 
use crate::system::models::{AppState}; 
use crate::data::models::{SystemData, QueryParams}; 
use crate::server::models::{SystemServer, SystemServerRoute};
use crate::integrations;
use rusqlite::{Row, types::ValueRef, Result};
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
//If T is needed for trait bounds or methods but not a field: You can use std::marker::PhantomData<T> to explicitly tell the compiler that you are aware of the unused parameter and intend to use it to "act like" the struct owns a T. PhantomData takes up no memory space.
use std::collections::HashMap;
use serde_json::{json, Value, Map};
use crate::server::constants::{SHARED_FILES_PATH, GLOBAL_ROUTES_PATH, GLOBAL_BASE_ROUTE_PATH};

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
            let path = format!("{}/{}/{}", SHARED_FILES_PATH, GLOBAL_ROUTES_PATH, item.1);
            let file = crate::files::manager::read_file(&path).unwrap_or(String::from("err"));
            
            // Process Markdown to html
            println!("Server file response: {:?}\n", file);
            let processed_markdown: std::result::Result<String, integrations::models::IntegrationError> = integrations::manager::run("parser", "parse", Some(&file));
            println!("Server markdown response: {:?}", processed_markdown);

            let content: &str = match processed_markdown {
                Ok(content) => {
                    // Return file content
                    return HttpResponse::Ok().content_type("text/html").body(content)
                },
                Err(err) => {
                    // Return error
                    println!("500: {:?}", err);
                    return HttpResponse::Ok().content_type("text/html").body(get_html_500())
                }
            };
        },
        Error => {
            // Return 404
            println!("404");
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
    let flattened_routes = flatten_routes(data.public);
    let actix_routes = convert_routes(flattened_routes);
    
    for service in actix_routes {
         cfg.service(service);
    }
    cfg.service(web::scope("").default_service(web::to(not_found)));
}


pub fn flatten_routes(data: Option<Vec<SystemServerRoute>>) -> HashMap<String, String> {
     let mut flattened: HashMap<String, String> = HashMap::new();

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


pub fn recursively_flatten_routes(route: SystemServerRoute, map: &mut HashMap<String, String>, parent_route: &str) {
    // Insert the current node's data into the map
    let path = if parent_route.is_empty() {
        route.name 
    } else {
        format!("{}/{}", parent_route, route.name) 
    }; 

    map.insert(path.clone(), route.file.clone());

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
fn convert_routes(mappped_routes: HashMap<String, String>) -> Vec<actix_web::Resource> {
    let mut route_map: Vec<actix_web::Resource> = Vec::new();
   
    for child in mappped_routes {
        route_map.push(web::resource(child.0).route(web::get().to(get)));
    }
    route_map
} 