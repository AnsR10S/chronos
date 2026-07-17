use std::io::{self, Write};
use std::process::Command;
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

        if trimmed_input.is_empty() {
            continue;
        }

        let mut parts = trimmed_input.split_whitespace();
        let command = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();

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
