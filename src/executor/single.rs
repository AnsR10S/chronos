use crate::parser::parser;
use crate::parser::ast::Redirect;
use crate::shell::builtin::{self, BuiltinStatus, find_executable};
use crate::executor::process;
use std::fs::{File, OpenOptions};
use std::io::Write;

pub fn is_builtin(cmd: &str) -> bool {
    ["echo", "exit", "type", "pwd", "cd", "complete", "jobs", "history", "declare"].contains(&cmd)
}

pub fn capture_builtin(command: &str, args: &[String]) -> String {
    match command {
        "echo" => format!("{}\n", args.join(" ")),
        "pwd" => std::env::current_dir().ok().map(|d| format!("{}\n", d.display())).unwrap_or_default(),
        "type" => {
            if let Some(target) = args.get(0) {
                if is_builtin(target) {
                    format!("{} is a shell builtin\n", target)
                } else if let Some(path) = crate::shell::builtin::find_executable(target) {
                    format!("{} is {}\n", target, path)
                } else {
                    format!("{}: not found\n", target)
                }
            } else {
                String::new()
            }
        }
        "history" => { // Capture history output for pipelines
            let registry = crate::shell::state::history::history_registry().lock().unwrap();
            let total = registry.len();
            let mut limit = total;

            if let Some(arg) = args.get(0) {
                if let Ok(n) = arg.parse::<usize>() {
                    limit = n;
                }
            }

            let start_idx = total.saturating_sub(limit);
            let mut out = String::new();
            for (i, cmd) in registry.iter().enumerate().skip(start_idx) {
                out.push_str(&format!("{:>5} {}\n", i + 1, cmd));
            }
            out
        }
        _ => String::new(),
    }
}

pub fn execute(tokens: Vec<String>) -> bool {
    if let Some(mut parsed_cmd) = parser::parse(tokens) {

        let mut is_background = false;

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
