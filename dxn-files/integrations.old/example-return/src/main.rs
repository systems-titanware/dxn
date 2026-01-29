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
impl Termination for Person {
    fn report(self) -> ExitCode {
        // Define how a Person struct translates to an exit code
        // e.g., 0 for success, non-zero for failure
          ExitCode::from(0)
    }
}

fn main() -> Result<Person> {
    let args: Vec<String> = env::args().collect();
//    HashMap<String, CommandFn> = HashMap::new();
 
    /*
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);

    for line in reader.lines() {
        let line = line?;
        // Process the line of data here
        println!("Processing line: {}", line); 
    }
 */
 
    let command = args[1].as_str();
    let input_args = if args.len() > 2 { args[2].as_str() } else { "" };
  
    //let input_args = args[2].as_str();
    let data: Person = serde_json::from_str(input_args).expect("Format of type person");

    greet(&data.name);
    Ok(data)
        
}
