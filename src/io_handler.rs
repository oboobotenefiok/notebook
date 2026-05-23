use std::io::{self, Write};

pub fn io_handler(prompt: &str) -> String {
    println!("{}", prompt);
    print!("> ");
    io::stdout().flush().unwrap(); // WATCH AGAIN

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}
