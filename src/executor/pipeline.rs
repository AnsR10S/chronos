use crate::parser::parser;
use crate::shell::builtins;
use crate::executor::single::{is_builtin, capture_builtin};
use std::io::Write;
use std::process::{Command, Stdio, Child};

pub fn execute(pipe_chunks: Vec<Vec<String>>) -> bool {
    let mut children: Vec<Child> = Vec::new();
    let mut previous_stdout: Option<std::process::ChildStdout> = None;
    let mut pending_builtin_out: Option<String> = None;

    let total_commands = pipe_chunks.len();

    for (i, chunk) in pipe_chunks.into_iter().enumerate() {
        let is_last = i == total_commands - 1;

        if let Some(parsed_cmd) = parser::parse(chunk) {

            if is_builtin(&parsed_cmd.name) {
                if !is_last {
                    pending_builtin_out = Some(capture_builtin(&parsed_cmd.name, &parsed_cmd.args));
                    previous_stdout = None;
                } else {
                    let _ = builtins::execute(&parsed_cmd.name, &parsed_cmd.args, &parsed_cmd.stdout);
                }
            } else {
                let mut cmd = Command::new(&parsed_cmd.name);
                cmd.args(&parsed_cmd.args);

                if let Some(prev_out) = previous_stdout.take() {
                    cmd.stdin(Stdio::from(prev_out));
                } else if pending_builtin_out.is_some() {
                    cmd.stdin(Stdio::piped());
                } else {
                    cmd.stdin(Stdio::inherit());
                }

                if is_last {
                    cmd.stdout(Stdio::inherit());
                } else {
                    cmd.stdout(Stdio::piped());
                }

                if let Ok(mut child) = cmd.spawn() {
                    if let Some(text) = pending_builtin_out.take() {
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(text.as_bytes());
                        }
                    }

                    if !is_last {
                        previous_stdout = child.stdout.take();
                    }
                    children.push(child);
                } else {
                    eprintln!("{}: command not found", parsed_cmd.name);
                    break;
                }
            }
        }
    }

    for mut child in children {
        let _ = child.wait();
    }

    false
}
