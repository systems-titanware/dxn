
/*
todo


use std::io;

use rusqlite::{params, Connection, Result, Row};
use serde_rusqlite::to_params;

//use crate::data::models::{Item, Person};
use crate::data::db::models::{DbColumn};

// TRAITS

// IMPLEMENTATIONS 

// FUNCTIONS

// Old

pub fn create_col_primary(name: String, col_type: String) -> DbColumn {
    DbColumn {
        name: name,
        col_type: col_type,
        value: None,
        primary_key: Some(true),
        secondary_key: None,
        nullable: false,
        unique: None
    }
}
pub fn create_col(name: String, col_type: String, nullable: bool) -> DbColumn {
    DbColumn {
        name: name,
        col_type: col_type,
        value: None,
        primary_key: None,
        secondary_key: None,
        nullable: nullable,
        unique: None
    }
} 
// New

pub fn create_dynamic_table(db_name: String, table_name: String, fields: Vec<DbColumn>) -> Result<()> {
    let conn = Connection::open(format!("{}.db", db_name))?;

    let mut columns = String::new();
    for (i, field) in fields.iter().enumerate() {
        if let Some(true) = field.primary_key {
            //id      BLOB PRIMARY KEY NOT NULL,
            columns.push_str(&format!("{} {} PRIMARY KEY,\n", field.name, field.col_type));
        }
        else if let Some(true) = field.secondary_key {
            columns.push_str(&format!("{} {} SECONDARY KEY,\n", field.name, field.col_type));
        }
        else if let Some(true) = field.unique {
            columns.push_str(&format!("{} {} UNIQUE,\n", field.name, field.col_type));
        }
        else if field.nullable == false  {
            columns.push_str(&format!("{} {} NOT NULL,\n", field.name, field.col_type));
        }
        else {
            columns.push_str(&format!("{} {},\n", field.name, field.col_type));
        }
    }
    // Remove \n
    columns.pop();
    // Remove , 
    columns.pop(); 

    let str = &format!(
        "CREATE TABLE IF NOT EXISTS {} (\n{})", table_name, columns);
    //println!("{}", str);
    conn.execute(str, [], )?;
    Ok(())
}
/*
pub fn create_table(db_name: String) -> Result<()> {
    let conn = Connection::open(format!("{}.db", db_name))?;
    
    let str = &format!(
        "create table if not exists {} (
            id integer primary key,
            name text not null,
            country text 
        )", db_name);
    conn.execute(str, [], )?;
    Ok(())
}
*/
pub fn insert<T>(db_name: String, table_name: String, object: T) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open(format!("{}.db", db_name))?;
    let query = format!("INSERT INTO {} (name, country) VALUES (?1, ?2)", table_name);
    // 2. Execute the query with data parameters bound securely
    conn.execute(
        &query,
        params![name, country],
    )
}

pub fn update(db_name: String, table_name: String, id: u32, object: T) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open(format!("{}.db", db_name))?;
    let query = format!("UPDATE {} SET name = ?1, country = ?2 WHERE id = ?3", table_name);

    // 2. Execute the query with data parameters bound securely
    conn.execute(
        &query,
        params![name, country, id],
    )
}

pub fn delete(db_name: String, table_name: String, id: u32) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open(format!("{}.db", db_name))?;
    let query = format!("DELETE FROM {} WHERE id = ?1", table_name);

    // Insert data into the 'person' table
    conn.execute(
        &query,
        params![id], // Bind parameters using the params! macro
    )
}

pub fn get<F, T>(db_name: String, table_name: String, id: u32, mapper: F) -> Result<T> 
where 
    F: FnMut(&Row) -> Result<T>,
    T: Sized,
{
    let conn = Connection::open(format!("{}.db", db_name))?;
    let query = format!("SELECT * FROM {} WHERE id = ?1", table_name);
    //let mut stmt = conn.prepare(query)?;
    
    conn.query_row(
        &query,
        [id], // Parameters
        mapper, // Mapper
    )
}

pub fn list<F, T>(db_name: String, table_name: String,page_size: u32, page: u32, query: String, mapper: F) -> Result<Vec<T>> 
where
    F: FnMut(&Row) -> Result<T>, // The trait bound for the closure
    T: Sized, // The return type of the closure
{
    let conn = Connection::open(format!("{}.db", table_name))?;

    let query = &format!("SELECT * FROM {}", table_name);
    let mut stmt = conn.prepare(query)?;

    // Pass the external closure to query_map
    let rows = stmt.query_map([], mapper)?;
    
    // Collect the results into a Vec using fallible_iterator
    rows.collect()
}

pub fn read_table(table_name: String) -> Result<()> {
    let conn = Connection::open(format!("{}.db", table_name))?;
    let str = &format!("SELECT * FROM {}", table_name);
    conn.execute(str, [], )?;
    Ok(())
}
     */