use std::env;
use crate::parser::ast::Redirect;
use crate::shell::builtins::BuiltinStatus;

pub fn execute(args: &[String], _stdout: &Redirect) -> BuiltinStatus {
    let target = args.get(0).map(String::as_str).unwrap_or("~");
    let path = if target == "~" {
        env::var("HOME").unwrap_or_default()
    } else {
        target.to_string()
    };

    if let Err(_) = env::set_current_dir(&path) {
        println!("cd: {}: No such file or directory", target);
    }
    BuiltinStatus::Handled
}
