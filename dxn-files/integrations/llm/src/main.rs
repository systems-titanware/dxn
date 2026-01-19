fn main() {
    println!("Hello, llm!");
    let args: Vec<String> = env::args().collect();
    // args[0] is the program's path
    if args.len() > 1 {
        println!("The first argument is: {}", args[1]);
    } else {
        println!("No arguments provided.");
    }
}
