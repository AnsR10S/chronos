// Import find_executable from the parent module
use super::{find_executable, BuiltinStatus, BUILTINS};

pub fn execute(args: &[&str]) -> BuiltinStatus {
    // Grab the first argument provided to the `type` command
    if let Some(target) = args.first() {
        // Check if it's a known builtin
        if BUILTINS.contains(target) {
            println!("{} is a shell builtin", target);
        }
        // If not a builtin, search the PATH for an executable
        else if let Some(path) = find_executable(target) {
            println!("{} is {}", target, path);
        }
        // If neither, it does not exist
        else {
            println!("{}: not found", target);
        }
    }

    BuiltinStatus::Handled
}
