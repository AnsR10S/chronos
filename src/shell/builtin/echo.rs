use super::BuiltinStatus;

pub fn execute(args: &[&str]) -> BuiltinStatus {
    println!("{}", args.join(" "));
    BuiltinStatus::Handled
}
