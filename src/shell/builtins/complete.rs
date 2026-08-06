use crate::parser::ast::Redirect;
use crate::shell::builtins::{completion_registry, print_output, BuiltinStatus};

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    if args.len() >= 3 && args[0] == "-C" {
        let script_path = &args[1];
        let target_cmd = &args[2];

        let mut registry = completion_registry().lock().unwrap();
        registry.insert(target_cmd.clone(), script_path.clone());

    } else if args.len() >= 2 && args[0] == "-p" {
        let target_cmd = &args[1];
        let registry = completion_registry().lock().unwrap();

        if let Some(script_path) = registry.get(target_cmd) {
            let output = format!("complete -C '{}' {}\n", script_path, target_cmd);
            print_output(&output, stdout);
        } else {
            let output = format!("complete: {}: no completion specification\n", target_cmd);
            print_output(&output, stdout);
        }

    } else if args.len() >= 2 && args[0] == "-r" {
        let target_cmd = &args[1];
        let mut registry = completion_registry().lock().unwrap();
        registry.remove(target_cmd);
    }

    BuiltinStatus::Handled
}
