use crate::parser::ast::{Command, Redirect};

pub fn parse(tokens: Vec<String>) -> Option<Command> {
    if tokens.is_empty() {
        return None;
    }

    let name = tokens[0].clone();
    let mut args = Vec::new();
    let mut stdout = Redirect::None;
    let mut stderr = Redirect::None;

    let mut iter = tokens.into_iter().skip(1);

    while let Some(token) = iter.next() {
        match token.as_str() {
            ">" | "1>" => {
                if let Some(file) = iter.next() { stdout = Redirect::Overwrite(file); }
            }
            ">>" | "1>>" => {
                if let Some(file) = iter.next() { stdout = Redirect::Append(file); }
            }
            "2>" => {
                if let Some(file) = iter.next() { stderr = Redirect::Overwrite(file); }
            }
            "2>>" => {
                if let Some(file) = iter.next() { stderr = Redirect::Append(file); }
            }
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
