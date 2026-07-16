use std::io::{self, Write};
use crate::shell::builtin::{self, BuiltinStatus};

pub fn start() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let trimmed_input = user_input.trim();

        let mut parts = trimmed_input.split_whitespace();
        let command = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();

        match builtin::execute(command, &args) {
            BuiltinStatus::Exit => break,
            BuiltinStatus::Handled => continue,
            BuiltinStatus::NotHandled => {
                println!("{}: command not found", command);
            }
        }
    }
}
