use std::io::{self, Write};
use std::process::Command;
use crate::shell::builtin::{self, BuiltinStatus, find_executable};
// Import the tokenizer
use crate::lexer::lexer::tokenize;

pub fn start() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let trimmed_input = user_input.trim();

        if trimmed_input.is_empty() {
            continue;
        }

        // Pass the input through your lexical analyzer
        let tokens = tokenize(trimmed_input);

        // Safety check: if tokenization resulted in nothing, skip
        if tokens.is_empty() {
            continue;
        }

        // Extract the command (first token) and args (the rest) as &str
        let command = tokens[0].as_str();
        let args: Vec<&str> = tokens.iter().skip(1).map(|s| s.as_str()).collect();

        match builtin::execute(command, &args) {
            BuiltinStatus::Exit => break,
            BuiltinStatus::Handled => continue,
            BuiltinStatus::NotHandled => {
                if find_executable(command).is_some() {
                    let mut child = Command::new(command);
                    child.args(&args);

                    match child.status() {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to execute {}: {}", command, e),
                    }
                } else {
                    println!("{}: command not found", command);
                }
            }
        }
    }
}
