use crate::parser::ast::Redirect;
use crate::shell::builtins::BuiltinStatus;
use crate::shell::state::variables;

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }
    true
}

pub fn execute(args: &[String], _stdout: &Redirect) -> BuiltinStatus {
    if args.is_empty() {
        return BuiltinStatus::Handled;
    }

    if args[0] == "-p" {
        for arg in args.iter().skip(1) {
            if let Some(val) = variables::get_variable(arg) {
                println!("declare -- {}=\"{}\"", arg, val);
            } else {
                println!("declare: {}: not found", arg);
            }
        }
        return BuiltinStatus::Handled;
    }

    for arg in args {
        if let Some((name, value)) = arg.split_once('=') {
            if is_valid_identifier(name) {
                variables::set_variable(name.to_string(), value.to_string());
            } else {
                eprintln!("declare: `{}': not a valid identifier", arg);
            }
        } else {
            if is_valid_identifier(arg) {
                variables::set_variable(arg.to_string(), String::new());
            } else {
                eprintln!("declare: `{}': not a valid identifier", arg);
            }
        }
    }

    BuiltinStatus::Handled
}
