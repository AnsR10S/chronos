use std::env;

pub mod cd;
pub mod echo;
pub mod pwd;
pub mod type_cmd;

pub enum BuiltinStatus {
    Handled,
    NotHandled,
    Exit,
}

pub const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

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

pub fn execute(command: &str, args: &[&str]) -> BuiltinStatus {
    match command {
        "exit" => BuiltinStatus::Exit,
        "echo" => echo::execute(args),
        "type" => type_cmd::execute(args),
        "pwd" => pwd::execute(args),
        "cd" => cd::execute(args),
        _ => BuiltinStatus::NotHandled,
    }
}
