#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Print the shell prompt without a newline
    print!("$ ");

    // Flush standard output to ensure the prompt displays immediately
    // before waiting for the user's command
    io::stdout().flush().unwrap();

    // Allocate a mutable string buffer to store the incoming command
    let mut user_input = String::new();

    // Read standard input from the terminal and append it to the buffer
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    // Strip trailing whitespace and newline characters from the raw input
    let trimmed_input = user_input.trim();

    // Output the formatted error message for an unrecognized command
    println!("{}: command not found", trimmed_input);
}
