use std::fs::{File, OpenOptions};
use std::process::{Command as StdCommand, Stdio};
use crate::parser::ast::Redirect;

// Spawns external system processes and maps their file descriptors natively via the OS
pub fn run_external(command: &str, args: &[String], stdout: &Redirect, stderr: &Redirect) {
    let mut cmd = StdCommand::new(command);
    cmd.args(args);

    // Map Standard Output
    match stdout {
        Redirect::None => {} // Inherits the terminal's default stdout
        Redirect::Overwrite(path) => {
            if let Ok(f) = File::create(path) { cmd.stdout(Stdio::from(f)); }
        }
        Redirect::Append(path) => {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(path) {
                cmd.stdout(Stdio::from(f));
            }
        }
    }

    // Map Standard Error
    match stderr {
        Redirect::None => {} // Inherits the terminal's default stderr
        Redirect::Overwrite(path) => {
            if let Ok(f) = File::create(path) { cmd.stderr(Stdio::from(f)); }
        }
        Redirect::Append(path) => {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(path) {
                cmd.stderr(Stdio::from(f));
            }
        }
    }

    // Execute the process and wait for it to finish
    if let Ok(mut child) = cmd.spawn() {
        let _ = child.wait();
    }
}
