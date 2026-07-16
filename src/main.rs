#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // The core REPL (Read-Eval-Print Loop) of the shell.
    // This loop runs indefinitely, continuously polling for new commands.
    loop{
        print!("$ ");

        io::stdout().flush().unwrap();

        let mut user_input = String::new();

        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let trimmed_input = user_input.trim();

        println!("{}: command not found", trimmed_input);
    }
}
