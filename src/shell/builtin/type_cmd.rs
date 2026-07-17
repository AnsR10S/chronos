use super::{find_executable, BuiltinStatus, BUILTINS};

pub fn execute(args: &[&str]) -> BuiltinStatus {
    if let Some(target) = args.first() {
        if BUILTINS.contains(target) {
            println!("{} is a shell builtin", target);
        }
        else if let Some(path) = find_executable(target) {
            println!("{} is {}", target, path);
        }
        else {
            println!("{}: not found", target);
        }
    }

    BuiltinStatus::Handled
}
