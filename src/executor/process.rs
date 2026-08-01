use std::fs::{File, OpenOptions};
use std::process::{Command as StdCommand, Stdio};
use crate::parser::ast::Redirect;

pub fn run_external(command: &str, args: &[String], stdout: &Redirect, stderr: &Redirect) {
    let mut cmd = StdCommand::new(command);
    cmd.args(args);

    match stdout {
        Redirect::None => {}
        Redirect::Overwrite(path) => {
            if let Ok(f) = File::create(path) { cmd.stdout(Stdio::from(f)); }
        }
        Redirect::Append(path) => {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(path) {
                cmd.stdout(Stdio::from(f));
            }
        }
    }

    match stderr {
        Redirect::None => {}
        Redirect::Overwrite(path) => {
            if let Ok(f) = File::create(path) { cmd.stderr(Stdio::from(f)); }
        }
        Redirect::Append(path) => {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(path) {
                cmd.stderr(Stdio::from(f));
            }
        }
    }

    if let Ok(mut child) = cmd.spawn() {
        let _ = child.wait();
    }
}

// Spawns the process and immediately returns control to the shell!
pub fn run_background(command: &str, args: &[String], stdout: &Redirect, stderr: &Redirect) {
    let mut cmd = StdCommand::new(command);
    cmd.args(args);

    match stdout {
        Redirect::None => {}
        Redirect::Overwrite(path) => {
            if let Ok(f) = File::create(path) { cmd.stdout(Stdio::from(f)); }
        }
        Redirect::Append(path) => {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(path) {
                cmd.stdout(Stdio::from(f));
            }
        }
    }

    match stderr {
        Redirect::None => {}
        Redirect::Overwrite(path) => {
            if let Ok(f) = File::create(path) { cmd.stderr(Stdio::from(f)); }
        }
        Redirect::Append(path) => {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(path) {
                cmd.stderr(Stdio::from(f));
            }
        }
    }

    // we spawn but we DO NOT call wait()
    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            let mut full_cmd = command.to_string();

            if !args.is_empty() {
                full_cmd.push_str(" ");
                full_cmd.push_str(&args.join(" "));
            }
            full_cmd.push_str(" &");

            // Register it in our state manager
            let job_id = crate::shell::state::jobs::add_job(child, full_cmd);
            println!("[{}] {}", job_id, pid);
        }
        Err(e) => {
            eprintln!("failed to execute background process: {}", e);
        }
    }
}
