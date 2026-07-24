use std::io::{self, Write};
use crate::lexer::lexer::tokenize;
// We now route execution through our single transaction executor
use crate::executor::single;

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

        // Pass raw input through your lexical analyzer
        let tokens = tokenize(trimmed_input);

        // Safety check: if tokenization resulted in nothing, skip
        if tokens.is_empty() {
            continue;
        }

        // Hand over the parsed tokens to the executor
        // If execute() returns true, it means the user triggered the "exit" builtin.
        if single::execute(tokens) {
            break;
        }
    }
}
