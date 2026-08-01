use crate::parser::parser;
use crate::parser::ast::Redirect;
use crate::shell::builtin::{self, BuiltinStatus, find_executable};
use crate::executor::process;
use std::fs::{File, OpenOptions};
use std::io::Write;

pub fn execute(tokens: Vec<String>) -> bool {
    if let Some(mut parsed_cmd) = parser::parse(tokens) {

        let mut is_background = false;

        // Handle background process flag
        if parsed_cmd.args.last().map(|s| s.as_str()) == Some("&") {
            parsed_cmd.args.pop();
            is_background = true;
        }

        let args_refs: Vec<&str> = parsed_cmd.args.iter().map(|s| s.as_str()).collect();

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

        match builtin::execute(&parsed_cmd.name, &args_refs, &parsed_cmd.stdout) {
            BuiltinStatus::Exit => return true,
            BuiltinStatus::Handled => return false,
            BuiltinStatus::NotHandled => {

                if find_executable(&parsed_cmd.name).is_some() {
                    // Route to the correct runner based on the background flag
                    if is_background {
                        process::run_background(&parsed_cmd.name, &parsed_cmd.args, &parsed_cmd.stdout, &parsed_cmd.stderr);
                    } else {
                        process::run_external(&parsed_cmd.name, &parsed_cmd.args, &parsed_cmd.stdout, &parsed_cmd.stderr);
                    }
                } else {

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
    false
}
