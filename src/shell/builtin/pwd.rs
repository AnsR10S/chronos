use super::BuiltinStatus;
use std::env;

pub fn execute(_args: &[&str]) -> BuiltinStatus {
    if let Ok(dir) = env::current_dir() {
        println!("{}", dir.display());
    } else {
        eprintln!("pwd: error retrieving current directory");
    }

    BuiltinStatus::Handled
}
