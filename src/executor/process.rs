use std::fs::{File, OpenOptions};
use std::process::{Command as StdCommand, Stdio};
use crate::parser::ast::Redirect;

pub fn run_external(command: &str, args: &[String], stdout: &Redirect, stderr: &Redirect) -> bool {
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

    match cmd.spawn() {
        Ok(mut child) => {
            match child.wait() {
                Ok(status) => status.success(),
                Err(e) => {
                    eprintln!("process failed to wait: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("failed to execute process: {}", e);
            false
        }
    }
}

pub fn run_background(command: &str, args: &[String], stdout: &Redirect, stderr: &Redirect) -> bool {
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

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            let mut full_cmd = command.to_string();

            if !args.is_empty() {
                full_cmd.push_str(" ");
                full_cmd.push_str(&args.join(" "));
            }
            full_cmd.push_str(" &");

            let job_id = crate::shell::state::jobs::add_job(child, full_cmd);
            println!("[{}] {}", job_id, pid);

            // Background commands are considered successfully "executed"
            // once they are spawned, as we cannot block the shell to wait for them.
            true
        }
        Err(e) => {
            eprintln!("failed to execute background process: {}", e);
            false
        }
    }
}
