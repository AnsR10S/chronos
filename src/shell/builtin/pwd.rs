use super::BuiltinStatus;
use std::env;

// We use _args with an underscore to tell Rust we know we aren't using the arguments
pub fn execute(_args: &[&str]) -> BuiltinStatus {
    if let Ok(dir) = env::current_dir() {
        println!("{}", dir.display());
    } else {
        eprintln!("pwd: error retrieving current directory");
    }

    BuiltinStatus::Handled
}
