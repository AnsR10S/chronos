use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    // Inject the newline directly into the formatted string
    let output = format!("{}\n", args.join(" "));
    print_output(&output, stdout);
    BuiltinStatus::Handled
}
