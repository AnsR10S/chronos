use super::BuiltinStatus;
use crate::parser::ast::Redirect;
use std::fs::{File, OpenOptions};
use std::io::Write;

// Accepts the stdout redirect target from the parser
pub fn execute(args: &[&str], stdout: &Redirect) -> BuiltinStatus {
    let output = args.join(" ");
    let final_output = format!("{}\n", output);

    // Route the output based on the parsed redirect
    match stdout {
        Redirect::None => {
            print!("{}", final_output);
        }
        Redirect::Overwrite(path) => {
            if let Ok(mut f) = File::create(path) {
                let _ = f.write_all(final_output.as_bytes());
            }
        }
        Redirect::Append(path) => {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = f.write_all(final_output.as_bytes());
            }
        }
    }

    BuiltinStatus::Handled
}
