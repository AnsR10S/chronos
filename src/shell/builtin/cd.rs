use super::BuiltinStatus;
use std::env;

pub fn execute(args: &[&str]) -> BuiltinStatus {
    let target = args.first().copied().unwrap_or("~");

    let path = if target == "~" {
        env::var("HOME").unwrap_or_default()
    } else {
        target.to_string()
    };

    if env::set_current_dir(&path).is_err() {
        println!("cd: {}: No such file or directory", target);
    }

    BuiltinStatus::Handled
}
