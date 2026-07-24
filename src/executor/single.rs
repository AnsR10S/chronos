use crate::parser::parser;
use crate::parser::ast::Redirect;
use crate::shell::builtin::{self, BuiltinStatus, find_executable};
use crate::executor::process;
use std::fs::{File, OpenOptions};
use std::io::Write;

// Orchestrates the pipeline: Parser -> Route to Builtin OR External Process
pub fn execute(tokens: Vec<String>) -> bool {
    if let Some(parsed_cmd) = parser::parse(tokens) {

        // Map Vec<String> to Vec<&str> to keep compatibility with your current builtin signature
        let args_refs: Vec<&str> = parsed_cmd.args.iter().map(|s| s.as_str()).collect();

        // Eagerly create or open files for redirection BEFORE execution
        // This ensures the files exist even if a builtin command produces no output.
        match &parsed_cmd.stdout {
            Redirect::None => {}
            Redirect::Overwrite(path) => { let _ = File::create(path); }
            Redirect::Append(path) => { let _ = OpenOptions::new().create(true).append(true).open(path); }
        }

        match &parsed_cmd.stderr {
            Redirect::None => {}
            Redirect::Overwrite(path) => { let _ = File::create(path); }
            Redirect::Append(path) => { let _ = OpenOptions::new().create(true).append(true).open(path); }
        }

        // Check if it's a builtin command first
        // Pass `&parsed_cmd.stdout` to the builtin executor so commands like `echo` can write to files
        match builtin::execute(&parsed_cmd.name, &args_refs, &parsed_cmd.stdout) {
            BuiltinStatus::Exit => return true,
            BuiltinStatus::Handled => return false,
            BuiltinStatus::NotHandled => {

                // If it's not a builtin, look for it in the system PATH
                if find_executable(&parsed_cmd.name).is_some() {
                    process::run_external(&parsed_cmd.name, &parsed_cmd.args, &parsed_cmd.stdout, &parsed_cmd.stderr);
                } else {

                    // Handle shell-level "command not found" errors, respecting stderr redirections natively
                    let error_msg = format!("{}: command not found\n", parsed_cmd.name);

                    match &parsed_cmd.stderr {
                        Redirect::None => eprint!("{}", error_msg),
                        Redirect::Overwrite(path) => {
                            if let Ok(mut f) = File::create(path) { let _ = f.write_all(error_msg.as_bytes()); }
                        }
                        Redirect::Append(path) => {
                            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                                let _ = f.write_all(error_msg.as_bytes());
                            }
                        }
                    }
                }
            }
        }
    }
    // Return false to keep the REPL loop running
    false
}
