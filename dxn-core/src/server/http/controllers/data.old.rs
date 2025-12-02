//use serde::Deserialize;
use actix_web::{get, post, web, App, HttpResponse, HttpRequest, HttpServer, Responder};
use std::sync::{Mutex, RwLock};
use crate::data::db::sqlite::*;
use crate::data::db::DbColumn;
use crate::data::models::{Person,QueryParams, AppState,SystemData, SystemDataModel};
use std::error::Error;
use rusqlite::{Row};
use serde::{Deserialize, Serialize};
use dxn_proc_macros::FieldNames; // Import your custom derive macro
//If T is needed for trait bounds or methods but not a field: You can use std::marker::PhantomData<T> to explicitly tell the compiler that you are aware of the unused parameter and intend to use it to "act like" the struct owns a T. PhantomData takes up no memory space.
use std::marker::PhantomData;
use std::collections::HashMap;


// HELPER FUNCTIONS
fn remove_last_char(s: &str) -> &str {
    match s.char_indices().next_back() {
        Some((i, _)) => &s[..i], // Slice from the beginning up to the start of the last char
        None => s, // If the string is empty, return it as is
    }
}

fn get_object_from_path(mut full_path: &str) -> &str {
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
    let mapper = |row: &Row| Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            country: row.get(2)?,
        });

    let object = get_object_from_path(req.path());
    let result = sqlite::get("public".to_string(), object.to_string(), id, mapper);

    match result {
        Ok(content) => {
            HttpResponse::Ok().json(content)
        }
        Err(err) => {
            let errMessage = format!("Error Getting Data: {}", err);
            eprintln!("Error Getting Data: {}", err);
            HttpResponse::Ok().body(errMessage)
        }
    }
}

// LIST
pub async fn list(req: HttpRequest, mut query_params:  web::Query<QueryParams>) -> impl Responder {
    let mut page_size = 10;
    let mut page = 10;
    let mut query = &String::new();

    match query_params.page_size { Some(size) => { page_size = size }, None => {} }

    match query_params.page { Some(size) => { page = size }, None => {} }

    match &query_params.query { Some(qs) => { query = qs }, None => {} }
    
    // Define the closure in the parent function/scope
    let person_mapper = |row: &Row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            country: row.get(2)?,
        })
    };
    let object = get_object_from_path(req.path());

    let people = sqlite::list("public".to_string(), object.to_string(), 5, 5, "queryStr".to_string(), person_mapper);
   
    let data = serde_json::json!(people.unwrap());
    //let str = format!("List page_size {}, page {}, query {}, persons {}!", page_size, page, query);
    HttpResponse::Ok().json(data)
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
    Other(String),
}

/* 
pub trait PayloadProperties<T: PayloadProperties> {
    fn getProperties(&self) -> Vec<String>;
    fn setProperties(&mut self, obj: T) -> Result<(), MyError>;
}
impl<T: PayloadProperties> PayloadProperties for Payload<T: PayloadProperties> {
    
    fn getProperties(&self) -> Vec<String> {
        let vec: Vec<String> = Vec::new();
        vec
    }

    fn setProperties(&mut self, obj: T) -> Result<(), MyError> {
        self.names = Vec::new();
        self.values = Vec::new();
        Ok(())
    }
}
     */
 
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub async fn post<T>(req: HttpRequest, payload: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder 
where
    T: std::fmt::Debug + Deserialize<'static> + Serialize + 'static, // Required traits for T
{
    let object = get_object_from_path(req.path());
    let result = sqlite::insert(
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

    let result = sqlite::update(
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

    let delete = sqlite::delete("public".to_string(), object.to_string(), id);

    match (delete) {
        Ok(content) => {
            println!("Deleted {}", id);
        }
        Err(err) => { 
            println!("Delete error: {}", err);
        }
    }

    let str = format!("Delete user_id {}!", id);
    HttpResponse::Ok().body(str)
} 


pub fn config(cfg: &mut web::ServiceConfig, data: SystemData) {
    println!("Setup public data API");
    match data.public {
        Some(vec) => {
            // 'vec' is a Vec<SystemDataModel> here
            if vec.is_empty() {
                //println!("Vector is present but empty.");
            } 
            else {
                //println!("Setup API for object: {:?}", vec);
                for element in vec {
                    let apiPath = format!("/{}", element.name);
                    println!("Setup API for object: {:?}", element.name);
                    cfg.service(
                        web::scope(&apiPath)
                            .route("/list", web::get().to(list))
                            .route("/{id}", web::get().to(get))
                            .route("/", web::post().to(post::<HashMap<String, serde_json::Value>>))
                            //Person
                            .route("/{id}", web::put().to(put::<HashMap<String, serde_json::Value>>))
                            .route("/{id}", web::delete().to(delete))
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
