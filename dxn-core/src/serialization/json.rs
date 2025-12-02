use crate::system::models::{System};
use std::fs::File;
use serde_json;
use std::error::Error;
use std::fs;
use std::io;
use serde::de::DeserializeOwned;


pub fn read_as_string() -> Result<String, std::io::Error> {
    let file_path = "./config.json".to_string();
    let file = File::open(file_path.clone()).expect("Failed to open file");

    let content = fs::read_to_string(file_path.clone())?;

    // Output the file content
    println!("Full file content:\n{}", content);
    Ok(content)
}

pub fn deserialize<T>(file_path: String) -> Result<T, serde_json::Error> 
where
    T: DeserializeOwned,
    {
    let file = File::open(file_path).expect("Failed to open file");

    // read_as_string();

    let data: T = serde_json::from_reader(file).expect("Failed to deserialize JSON from file");
    Ok(data)
}