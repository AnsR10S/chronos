use super::BuiltinStatus;

// Executes the echo command by joining arguments and printing them
pub fn execute(args: &[&str]) -> BuiltinStatus {
    println!("{}", args.join(" "));
    BuiltinStatus::Handled
}
