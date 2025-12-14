fn greet(str: &str) {
    println!("Hello, {}!", str);
}
fn calculate_sum(val1: i32, val2: i32) {
    println!("calculate_sum {}", val1+val2);
}

fn parse (data: Person) {
    println!("{:?}", data);
}
use std::io::{Error, ErrorKind};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use serde_json::Result; // Use serde_json::Result for convenient error handling
use std::process::Termination;

// 1. Annotate the struct with the Serialize derive macro
#[derive(Deserialize, Serialize, Debug)]
struct Person {
    name: String,
    age: u8,
    is_student: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let none_string: Option<String> = None;

    //args[0] is 'run'
    //args[1] is <command>
    let command = args[1].as_str();
    //args[2] is optional args

    let parser = pulldown_cmark::Parser::new(args[2].as_str());
    // Write to a new String buffer.
    let mut html_output = String::new();
    let result = pulldown_cmark::html::push_html(&mut html_output, parser);
   
    //println!("{:?}", arguments);
    println!("{:?}", html_output);
}
