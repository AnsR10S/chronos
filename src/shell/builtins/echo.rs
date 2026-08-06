use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let output = format!("{}\n", args.join(" "));
    print_output(&output, stdout);
    BuiltinStatus::Handled
}
