use std::io::{self, Write};
// Import Command to spawn external processes
use std::process::Command;
// Import find_executable from your builtin module
use crate::shell::builtin::{self, BuiltinStatus, find_executable};

pub fn start() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let trimmed_input = user_input.trim();

        // Prevent crashing or weird behavior if the user just hits Enter
        if trimmed_input.is_empty() {
            continue;
        }

        let mut parts = trimmed_input.split_whitespace();
        let command = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();

        // Pass both the command and the arguments to the executor
        match builtin::execute(command, &args) {
            BuiltinStatus::Exit => break,
            BuiltinStatus::Handled => continue,
            BuiltinStatus::NotHandled => {
                // It's not a builtin, so we're checking if it's an external executable
                if find_executable(command).is_some() {
                    // Spawn the external process
                    let mut child = Command::new(command);
                    child.args(&args);

                    // Execute the process and wait for it to finish.
                    // Its output will automatically stream to the terminal.
                    match child.status() {
                        Ok(_) => {} // Process finished successfully
                        Err(e) => eprintln!("Failed to execute {}: {}", command, e),
                    }
                } else {
                    // If it's not a builtin AND not in the PATH, it truly doesn't exist
                    println!("{}: command not found", command);
                }
            }
        }
    }
}
