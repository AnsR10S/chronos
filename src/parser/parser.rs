use crate::parser::ast::{Command, Redirect};

// Transforms a raw flat list of tokens into a structured Command
pub fn parse(tokens: Vec<String>) -> Option<Command> {
    if tokens.is_empty() {
        return None;
    }

    // The first token is always the command executable/builtin
    let name = tokens[0].clone();
    let mut args = Vec::new();
    let mut stdout = Redirect::None;
    let mut stderr = Redirect::None;

    // Use an iterator to cleanly skip over redirection operators and grab their target files
    let mut iter = tokens.into_iter().skip(1);

    while let Some(token) = iter.next() {
        match token.as_str() {
            // Standard Output (FD 1) Overwrite
            ">" | "1>" => {
                if let Some(file) = iter.next() { stdout = Redirect::Overwrite(file); }
            }
            // Standard Output (FD 1) Append
            ">>" | "1>>" => {
                if let Some(file) = iter.next() { stdout = Redirect::Append(file); }
            }
            // Standard Error (FD 2) Overwrite
            "2>" => {
                if let Some(file) = iter.next() { stderr = Redirect::Overwrite(file); }
            }
            // Standard Error (FD 2) Append
            "2>>" => {
                if let Some(file) = iter.next() { stderr = Redirect::Append(file); }
            }
            // If it's not a known operator, it's a standard argument
            _ => {
                args.push(token);
            }
        }
    }

    Some(Command {
        name,
        args,
        stdout,
        stderr,
    })
}
