use crate::parser::ast::Redirect;
use crate::shell::builtin::{completion_registry, print_output, BuiltinStatus};

pub fn execute(args: &[&str], stdout: &Redirect) -> BuiltinStatus {
    // If the user just types `complete` with no arguments, we do nothing.
    if args.is_empty() {
        return BuiltinStatus::Handled;
    }

    // REGISTER A SCRIPT (-C)
    if args.len() >= 3 && args[0] == "-C" {
        let script_path = args[1];
        let target_cmd = args[2];

        // Lock the global Mutex so we safely get write access
        // Unwrap it (if the lock fails, the thread panics, which is fine here)
        let mut registry = completion_registry().lock().unwrap();

        // Insert the mapping into our HashMap
        // Example: registry.insert("git", "/path/to/git_completer.sh")
        registry.insert(target_cmd.to_string(), script_path.to_string());

    // PRINT A SCRIPT SPECIFICATION (-p)
    } else if args.len() >= 2 && args[0] == "-p" {
        let target_cmd = args[1];

        // Lock the registry to read the HashMap safely
        let registry = completion_registry().lock().unwrap();

        // Check if the target_command exists in our registry
        if let Some(script_path) = registry.get(target_cmd) {
            let output = format!("complete -C '{}' {}\n", script_path, target_cmd);
            print_output(&output, stdout);
        } else {
            // Not found. Print the standard error string
            let output = format!("complete: {}: no completion specification\n", target_cmd);
            print_output(&output, stdout);
        }

    // UNREGISTER/REMOVE A SCRIPT (-r)
    } else if args.len() >= 2 && args[0] == "-r" {
        let target_cmd = args[1];

        // Lock the registry and remove the entry, effectively unregistering it
        let mut registry = completion_registry().lock().unwrap();
        registry.remove(target_cmd);
    }

    // Tell the executor that the command was successfully handled natively
    BuiltinStatus::Handled
}
