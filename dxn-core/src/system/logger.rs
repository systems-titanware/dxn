use chrono::prelude::*;
use crate::system::files;

pub fn log(str: &str){
     // FILESYSTEM
    files::manager::add_dir("logs/data");
    files::manager::add_dir("logs/server");

    let formatted_date_time = Utc::now().to_string();
    //let formatted_date_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let formatted_date = Utc::now().format("%Y-%m-%d").to_string();

    let log_name: String = format!("log_{}.txt", formatted_date);
    let log_path = format!("logs/server/{}", log_name);
    let my_str: &str = &log_path; // my_str is a &str

    let content = format!("[{}] OK: {}\n", formatted_date_time, str);
    files::manager::add_content(&content, my_str);
}


pub fn log_error(str: &str){
     // FILESYSTEM
    files::manager::add_dir("logs/data");
    files::manager::add_dir("logs/server");

    let formatted_date_time = Utc::now().to_string();
    //let formatted_date_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let formatted_date = Utc::now().format("%Y-%m-%d").to_string();

    let log_name: String = format!("log_{}.txt", formatted_date);
    let log_path = format!("logs/server/{}", log_name);
    let my_str: &str = &log_path; // my_str is a &str

    let content = format!("[{}] ERR: {}\n", formatted_date_time, str);
    files::manager::add_content(&content, my_str);
}