mod system;
mod data;
mod integrations;
mod functions;
//mod functions;

use uuid::Uuid;
use actix_cors::Cors;
use actix_web::http::header;
use std::env;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::body::BoxBody;
use std::sync::{Mutex, RwLock};
use rusqlite::{params, Connection, Result};
use data::db::sqlite::*;

use crate::data::models::{SystemData, SystemDataModel, SystemDataModelField};

use crate::functions::models::SystemFunctions;
use crate::system::server::models::{SystemServer, SystemServerRoute};
use crate::integrations::models::{SystemIntegrations, SystemIntegrationModel};

use crate::system::logger;
use crate::system::models::{AppState, System};
use crate::data::db::models::{DbColumn};

async fn stats(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name; // <- get app_name
    // Mutex example
    //let mut counter = data.counter.lock().unwrap();
    let mut counter = data.counter.write().unwrap();
    *counter += 1;

    let name = format!("Welcome to the {app_name} API, request number {counter}"); // <- response with app_name


    HttpResponse::Ok().body(name)
}

fn init_db(db_name: String, model: SystemDataModel) -> Result<()> {
    let mut columns: Vec<DbColumn> = Vec::new();
    
    // Filter for primary
    let primary: Option<SystemDataModelField> = model.fields
        .iter() // Get an iterator over references to items
        .filter(|item| item.primary == Some(true)) // Filter based on the Option<bool> field
        .cloned() // Clone the items to get owned copies in the new Vec
        .collect::<Vec<SystemDataModelField>>() // Collect the results into a new Vec<Item>
        .pop(); // Get last primary

    match(primary) {
        Some(primary_field) => {
            columns.push(repository::create_col_primary(primary_field.name, primary_field.datatype));
            //do nothing
        },
        None => {
            //Insert id as primary
            columns.push(repository::create_col_primary("id".to_string(), "number".to_string()));
        }
    } 
    // Filter for primary
    let standard_fields: Vec<SystemDataModelField> = model.fields
        .iter() // Get an iterator over references to items
        .filter(|item| item.primary == Some(false) || item.primary == None) // Filter based on the Option<bool> field
        .cloned() // Clone the items to get owned copies in the new Vec
        .collect::<Vec<SystemDataModelField>>();

    
    for field in standard_fields {
        columns.push(repository::create_col(field.name, field.datatype, true));
    }
    /*
    columns.push(sqlite::create_col_primary(String::from("id"), String::from("integer")));
    columns.push(sqlite::create_col(String::from("name"), String::from("TEXT"), false));
    columns.push(sqlite::create_col(String::from("country"), String::from("TEXT"), true));
    */
    repository::create_dynamic_table(db_name.clone(), model.name, columns);
    Ok(())
}

fn create_database(data: SystemData) -> Result<()> {
    match data.public {
        Some(vec) => {
            // 'vec' is a Vec<SystemDataModel> here
            if vec.is_empty() {
                //println!("Vector is present but empty.");
            } 
            else {
                //println!("Setup API for object: {:?}", vec);
                for element in vec {
                    //println!("Setup DB for object: {:?}", element.name);
                    let _ = init_db("public".to_string(), element);
                }
            }
        }
        None => {
            // println!("No vector present.");
        }
    }
    Ok(())
}

fn create_functions(functions: SystemFunctions)  -> Result<()> {
    match functions.public {
        Some(vec) => {
            // 'vec' is a Vec<SystemDataModel> here
            if vec.is_empty() {
                //println!("Vector is present but empty.");
            } 
            else {
            //crate::functions::manager::
                crate::functions::manager::init(vec.clone());
                //crate::functions::runner::run(vec);
            }
        }
        None => {
            // println!("No vector present.");
        }
    }
    Ok(())
}

fn init_integration(integration_name: String, model: SystemIntegrationModel) -> Result<()> {
    crate::integrations::manager::init(model);
    Ok(())
}

fn create_integrations(data: SystemIntegrations) -> Result<()> {
    match data.public {
        Some(vec) => {
            // 'vec' is a Vec<SystemDataModel> here
            if vec.is_empty() {
                //println!("Vector is present but empty.");
            } 
            else {
                //println!("Setup API for object: {:?}", vec);
                for element in vec {
                    //println!("Setup DB for object: {:?}", element.name);
                    let _ = init_integration("public".to_string(), element);
                }
            }
        }
        None => {
            // println!("No vector present.");
        }
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    logger::log("App initialized");
    // DATA
    let file_path = "./config.json".to_string();
    
    let system_data = system::serialization::json::deserialize::<System>(file_path);
    
    let app = web::Data::new(AppState {
        app_name: String::from("dxnet"),
        counter: RwLock::new(0),
        db_name: String::from("person"),
        // TODO: Fix below to match if let, else
        system: system_data.unwrap(),
        uuid: Uuid::now_v7()
    });


    /// DB
    // Note: config.json is now loaded above and used for system initialization
 
    // Create DB
    println!("Init -> Database");
    create_database(app.system.data.clone()); 
    
    println!("Init -> Integrations");
    create_integrations(app.system.integrations.clone()); 

    println!("Init -> Functions");
    create_functions(app.system.functions.clone());

    println!("Init -> Server");
 
    // Determine environment: default to "development" if DXN_ENV is not set
    let env = env::var("DXN_ENV").unwrap_or_else(|_| "development".to_string());
    let is_dev = env == "development";

    HttpServer::new(move || {
        // Configure CORS based on environment
        let cors = if is_dev {
            // In development, allow any origin for easier local testing (web, simulator, devices)
            Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
                .max_age(3600)
        } else {
            // In non-development environments, restrict origins explicitly.
            // Adjust allowed origins as needed for your deployment.
            Cors::default()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
                .supports_credentials()
                .max_age(3600)
        };

        App::new()
            .wrap(cors)
            // Configure app state
            .app_data(app.clone())
            // Configure routes from my_module under a specific scope
            .service(web::scope("/api/data")
                .configure(|cfg| { system::server::http::controllers::data::config(cfg, app.system.data.clone())})
            )
            .service(web::scope("/api/function")
                .configure(|cfg| { system::server::http::controllers::function::config(cfg, app.system.functions.clone())})
            )
            .service(web::scope("/api/config")
                .configure(|cfg| { system::server::http::controllers::config::config(cfg)})
            )
            .service(web::scope("/server")
                .configure(|cfg| system::server::http::controllers::server::config(cfg, app.system.server.clone()))
            )
            
            
            // Add default route
            .route("/_.stats", web::get().to(stats))
            .route("/", web::get().to(DXN))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
 

}





/*
     Server::build()
    .configure( |cfg : &mut ServiceConfig| {
        cfg.bind( "site1", "0.0.0.0:8000").expect("bind failed");
        cfg.bind( "site2", "0.0.0.0:9000").expect("bind failed");
        cfg.apply(callback).expect("Failed to configure HTTP service");
        Ok(())
    }).expect("Unable to configure")
    .run().expect("Failed to start HTTP server");
    
fn callback(runtime : &mut ServiceRuntime)  {
    runtime.service("site1",
        HttpService::build()
            .finish(
                App::new()
                    .wrap(middleware::DefaultHeaders::new().header(http::header::CACHE_CONTROL, "no-cache"))
                    //.wrap(middleware::Compress::default())  // <-- enabling this would prevent chunked response.
                    .wrap(middleware::Logger::default())
                    .service(
                        web::resource("/iot/events")
                            .route(web::get().to(outbound))
                            .route(web::put().to_async(inbound))
                    )
            ) 
    );

    runtime.service("site2",
        HttpService::build()
            .finish(
                App::new()
                    .service(
                        web::resource("/")
                            .route(web::get().to(|| HttpResponse::Ok()))
                    )
            ) 
    );
}
 */
async fn DXN(data: web::Data<AppState>) -> impl Responder {
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>DXN</title>
        </head>
        <body>
        </body>
        </html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html_content)
}
