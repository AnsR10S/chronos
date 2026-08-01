use std::env;
use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};

pub fn execute(_args: &[String], stdout: &Redirect) -> BuiltinStatus {
    if let Ok(dir) = env::current_dir() {
        print_output(&format!("{}\n", dir.display()), stdout);
    }
    BuiltinStatus::Handled
}
