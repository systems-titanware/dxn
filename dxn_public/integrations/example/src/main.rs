fn greet(str: &str) {
    println!("Hello, {}!", str);
}
fn calculate_sum(val1: i32, val2: i32) {
    println!("calculate_sum {}", val1+val2);
}

fn parse (data: Person) {
    println!("{:?}", data);
}
use serde::{Deserialize, Serialize};
use serde_json::Result; // Use serde_json::Result for convenient error handling

// 1. Annotate the struct with the Serialize derive macro
#[derive(Deserialize, Serialize, Debug)]
struct Person {
    name: String,
    age: u8,
    is_student: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
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
 
    match command {
        "greet" => {
            if args.len() > 2 {
                greet(&args[2]);
            } else {
                println!("Usage: main_app greet <name>");
            }
        }
        "parse" => { 
            if !&input_args.is_empty() {
                let input_args = args[2].as_str();
                let data: Person = serde_json::from_str(input_args).expect("Format of type person");

                greet(&data.name);
            } else {
                println!("ARGS: {:?}", args.len());
                println!("Usage: main_app sum <num1> <num2>");
            }
        },
        "sum" => { 
            if args.len() > 2 {
                let input_args = args[2].as_str();
                let cmd_args = input_args.split(",");
                println!("{:?}", cmd_args);
                if let (Ok(num1), Ok(num2)) = (args[0].parse::<i32>(), args[1].parse::<i32>()) {
                    calculate_sum(num1, num2);
                } else {
                    println!("Invalid numbers for sum.");
                }
            } else {
                println!("ARGS: {:?}", args.len());
                println!("Usage: main_app sum <num1> <num2>");
            }
        }
        _ => println!("Unknown command: {}", command),
    }
}
