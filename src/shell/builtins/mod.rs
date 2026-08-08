use std::collections::{HashSet, HashMap};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use crate::parser::ast::Redirect;

pub mod cd;
pub mod echo;
pub mod pwd;
pub mod type_cmd;
pub mod complete;
pub mod jobs;
pub mod history;
pub mod declare;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub enum BuiltinStatus {
    Handled,
    NotHandled,
    Exit,
}

pub const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd", "complete", "jobs", "history", "declare"];

pub fn completion_registry() -> &'static Mutex<HashMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn print_output(output: &str, stdout: &Redirect) {
    match stdout {
        Redirect::None => {
            print!("{}", output);
            let _ = std::io::stdout().flush();
        }
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

        // Standard check (works for Linux, or if user explicitly typed "notepad.exe")
        if executable.is_file() {
            #[cfg(unix)]
            let is_exec = std::fs::metadata(&executable)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false);

            #[cfg(not(unix))]
            let is_exec = true;

            if is_exec {
                return Some(executable.display().to_string());
            }
        }

        // Windows-specific check: if the exact name isn't found, try appending .exe
        #[cfg(windows)]
        {
            let mut exe_with_ext = executable.clone();
            exe_with_ext.set_extension("exe");
            if exe_with_ext.is_file() {
                return Some(exe_with_ext.display().to_string());
            }
        }
    }

    None
}

pub fn autocomplete(prefix: &str) -> Vec<String> {
    let mut matches = HashSet::new();

    for &cmd in BUILTINS {
        if cmd.starts_with(prefix) {
            matches.insert(cmd.to_string());
        }
    }

    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.starts_with(prefix) {
                            let path = entry.path();

                            if path.is_file() {
                                #[cfg(unix)]
                                let is_exec = fs::metadata(&path)
                                    .map(|m| m.permissions().mode() & 0o111 != 0)
                                    .unwrap_or(false);

                                #[cfg(not(unix))]
                                let is_exec = true;

                                if is_exec {
                                    matches.insert(file_name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut matches_vec: Vec<String> = matches.into_iter().collect();
    matches_vec.sort();
    matches_vec
}

pub fn get_completer(cmd: &str) -> Option<String> {
    let registry = completion_registry().lock().unwrap();
    registry.get(cmd).cloned()
}

pub fn execute(command: &str, args: &[String], stdout: &Redirect) -> BuiltinStatus {
    match command {
        "exit" => BuiltinStatus::Exit,
        "echo" => echo::execute(args, stdout),
        "pwd" => pwd::execute(args, stdout),
        "cd" => cd::execute(args, stdout),
        "type" => type_cmd::execute(args, stdout),
        "complete" => complete::execute(args, stdout),
        "jobs" => jobs::execute(args, stdout),
        "history" => history::execute(args, stdout),
        "declare" => declare::execute(args, stdout),
        _ => BuiltinStatus::NotHandled,
    }
}
