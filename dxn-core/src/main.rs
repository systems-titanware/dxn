mod data;
mod server;
mod serialization;
mod system;
mod files;

use uuid::Uuid;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::sync::{Mutex, RwLock};
use rusqlite::{params, Connection, Result};
use data::db::sqlite::*;
use chrono::prelude::*;

use crate::data::models::{SystemData, SystemDataModel, SystemDataModelField};

use crate::system::models::{AppState, System};
use crate::data::db::models::{DbColumn};

async fn welcome(data: web::Data<AppState>) -> impl Responder {
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
        println!("add_field {} {}", field.name, field.datatype);
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
    println!("create_database");
    
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
                    init_db("public".to_string(), element);
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

    // FILESYSTEM
    files::manager::add_dir("logs/data");
    files::manager::add_dir("logs/server");

    let date_as_string = Utc::now().to_string();
    let log_name: String = format!("log_{}.txt", date_as_string);
    let log_path = format!("logs/server/{}", log_name);
    let my_str: &str = &log_path; // my_str is a &str

    files::manager::add_content("Main start\n", my_str);
    match(files::manager::read_file("subfolder2/test1.txt")) {
        Err(err) => {

        }
        Ok(content) => {
            println!("Worked {:?}", content);
        }
    }

    // DATA

    /// DB
    let file_path = "./config.json".to_string();
    
    let system_data = serialization::json::deserialize::<System>(file_path);
    //println!("SystemData: {:?}", system_data);
    
    let app = web::Data::new(AppState {
        app_name: String::from("dxnet"),
        counter: RwLock::new(0),
        db_name: String::from("person"),
        // TODO: Fix below to match if let, else
        system: system_data.unwrap(),
        uuid: Uuid::now_v7()
    });

    // Create DB
    create_database(app.system.data.clone());
    
    // move counter into the closure
    HttpServer::new(move || {
        App::new()
            // Configure app state
            .app_data(app.clone())
            // Configure routes from my_module under a specific scope
            .service(web::scope("/api/data")
                .configure(|cfg| { server::http::controllers::data::config(cfg, app.system.data.clone())})
            )
            
            // Add default route
            .route("/", web::get().to(welcome))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}