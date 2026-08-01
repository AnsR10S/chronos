use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    // Intercept history flags
    if let Some(flag) = args.get(0) {
        if flag == "-r" {
            if let Some(path) = args.get(1) {
                crate::shell::state::history::append_from_file(path);
            }
            return BuiltinStatus::Handled;
        } else if flag == "-w" {
            if let Some(path) = args.get(1) {
                crate::shell::state::history::write_to_file(path);
            }
            return BuiltinStatus::Handled;
        } else if flag == "-a" {
            if let Some(path) = args.get(1) {
                crate::shell::state::history::append_to_file(path);
            }
            return BuiltinStatus::Handled;
        }
    }

    let registry = crate::shell::state::history::history_registry().lock().unwrap();
    let total = registry.len();
    let mut limit = total;

    if let Some(arg) = args.get(0) {
        if let Ok(n) = arg.parse::<usize>() {
            limit = n;
        }
    }

    let start_idx = total.saturating_sub(limit);
    for (i, cmd) in registry.iter().enumerate().skip(start_idx) {
        let output = format!("{:>5} {}\n", i + 1, cmd);
        print_output(&output, stdout);
    }

    BuiltinStatus::Handled
}
