use crate::parser::ast::Redirect;
use crate::shell::builtins::{find_executable, print_output, BuiltinStatus, BUILTINS};

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    if let Some(target) = args.get(0) {
        if BUILTINS.contains(&target.as_str()) {
            print_output(&format!("{} is a shell builtin\n", target), stdout);
        } else if let Some(path) = find_executable(target) {
            print_output(&format!("{} is {}\n", target, path), stdout);
        } else {
            print_output(&format!("{}: not found\n", target), stdout);
        }
    }
    BuiltinStatus::Handled
}
