use super::{BuiltinStatus, BUILTINS};

pub fn execute(args: &[&str]) -> BuiltinStatus {
    // Grab the first argument provided to the `type` command
    if let Some(target) = args.first() {
        // Check if this target exists in our static BUILTINS array
        if BUILTINS.contains(target) {
            println!("{} is a shell builtin", target);
        } else {
            // If it's not a builtin (and we aren't checking executables yet)
            println!("{}: not found", target);
        }
    }

    BuiltinStatus::Handled
}
