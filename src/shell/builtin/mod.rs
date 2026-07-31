use std::env;
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use crate::parser::ast::Redirect;
use std::io::Write;
use std::fs::{File, OpenOptions};

pub mod cd;
pub mod echo;
pub mod pwd;
pub mod type_cmd;
pub mod complete;

pub enum BuiltinStatus {
    Handled,
    NotHandled,
    Exit,
}

pub const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd", "complete"];

pub fn completion_registry() -> &'static Mutex<HashMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn print_output(output: &str, stdout: &Redirect) {
    match stdout {
        Redirect::None => print!("{}", output),
        Redirect::Overwrite(path) => {
            if let Ok(mut f) = File::create(path) {
                let _ = f.write_all(output.as_bytes());
            }
        }
        Redirect::Append(path) => {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = f.write_all(output.as_bytes());
            }
        }
    }
}

pub fn find_executable(cmd: &str) -> Option<String> {
    let path_var = env::var("PATH").unwrap_or_default();

    for path in env::split_paths(&path_var) {
        let executable = path.join(cmd);

        if executable.is_file() {
            #[cfg(unix)]
            let is_exec = {
                use std::os::unix::fs::PermissionsExt;
                std::fs::metadata(&executable)
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
            };

            #[cfg(not(unix))]
            let is_exec = true;

            if is_exec {
                return Some(executable.display().to_string());
            }
        }
    }

    None
}

pub fn execute(command: &str, args: &[&str], stdout: &Redirect) -> BuiltinStatus {
    match command {
        "exit" => BuiltinStatus::Exit,
        "echo" => echo::execute(args, stdout),
        "type" => type_cmd::execute(args),
        "pwd" => pwd::execute(args),
        "cd" => cd::execute(args),
        "complete" => complete::execute(args, stdout),
        _ => BuiltinStatus::NotHandled,
    }
}
