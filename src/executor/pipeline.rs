use crate::parser::parser;
use crate::shell::builtin;
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

            // Handle built-ins inside a pipeline
            if is_builtin(&parsed_cmd.name) {
                if !is_last {
                    // Capture the builtin output so it can be passed as stdin to the next command
                    pending_builtin_out = Some(capture_builtin(&parsed_cmd.name, &parsed_cmd.args));
                    previous_stdout = None;
                } else {
                    let args_refs: Vec<&str> = parsed_cmd.args.iter().map(|s| s.as_str()).collect();
                    let _ = builtin::execute(&parsed_cmd.name, &args_refs, &parsed_cmd.stdout);
                }
            } else {
                let mut cmd = Command::new(&parsed_cmd.name);
                cmd.args(&parsed_cmd.args);

                // Connect stdin to the previous command's stdout
                if let Some(prev_out) = previous_stdout.take() {
                    cmd.stdin(Stdio::from(prev_out));
                } else if pending_builtin_out.is_some() {
                    cmd.stdin(Stdio::piped()); // We will manually write the pending string to this
                } else {
                    cmd.stdin(Stdio::inherit());
                }

                // Connect stdout to the next command's stdin (unless it's the last command)
                if is_last {
                    cmd.stdout(Stdio::inherit());
                } else {
                    cmd.stdout(Stdio::piped());
                }

                if let Ok(mut child) = cmd.spawn() {
                    // Write any captured builtin string into the stdin of this external process
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

    // Wait for all processes in the pipeline to finish execution
    for mut child in children {
        let _ = child.wait();
    }

    false
}
