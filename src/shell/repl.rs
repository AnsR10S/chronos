use std::io::{self, Write};
// Import your new builtin module so the REPL can access the execute function
use crate::shell::builtin::{self, BuiltinStatus};

pub fn start() {
    // The core Read-Eval-Print Loop
    loop {
        // Print the prompt and flush the buffer
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read standard input
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let trimmed_input = user_input.trim();

        // Pass the command to the middleware pipeline for evaluation
        match builtin::execute(trimmed_input) {
            // Terminate the infinite loop if the user typed "exit"
            BuiltinStatus::Exit => break,
            // Skip the rest of the loop if a builtin executed successfully
            // (useful for future commands like 'echo' or 'pwd')
            BuiltinStatus::Handled => continue,
            // Fallback to the default error handling if it wasn't a builtin
            BuiltinStatus::NotHandled => {
                println!("{}: command not found", trimmed_input);
            }
        }
    }
}
