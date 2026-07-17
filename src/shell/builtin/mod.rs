use std::env;

pub mod echo;
pub mod type_cmd;

pub enum BuiltinStatus {
    Handled,
    NotHandled,
    Exit,
}

pub const BUILTINS: &[&str] = &["exit", "echo", "type"];

// Helper function to search the PATH environment variable for an executable file
pub fn find_executable(cmd: &str) -> Option<String> {
    // Fetch the PATH variable, defaulting to an empty string if not set
    let path_var = env::var("PATH").unwrap_or_default();

    // env::split_paths automatically handles the correct delimiter (: on Linux, ; on Windows)
    for path in env::split_paths(&path_var) {
        // Create the full path by appending the command name to the directory
        let executable = path.join(cmd);

        // Check if a file actually exists at this location
        if executable.is_file() {
            // For the Linux-based CodeCrafters tester, verify execute permissions
            #[cfg(unix)]
            let is_exec = {
                use std::os::unix::fs::PermissionsExt;
                std::fs::metadata(&executable)
                    .map(|m| m.permissions().mode() & 0o111 != 0) // Checks the executable bit
                    .unwrap_or(false)
            };

            // For your local Windows environment, just assume it's executable if it exists
            #[cfg(not(unix))]
            let is_exec = true;

            // If it exists and is executable, return the absolute path as a String
            if is_exec {
                return Some(executable.display().to_string());
            }
        }
    }

    // Return None if we looped through all PATH directories and found nothing
    None
}

pub fn execute(command: &str, args: &[&str]) -> BuiltinStatus {
    match command {
        "exit" => BuiltinStatus::Exit,
        "echo" => echo::execute(args),
        "type" => type_cmd::execute(args),
        _ => BuiltinStatus::NotHandled,
    }
}
