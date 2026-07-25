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
