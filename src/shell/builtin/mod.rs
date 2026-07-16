pub mod echo;
// Declare the new type module (named type_cmd to avoid Rust keyword conflicts)
pub mod type_cmd;

pub enum BuiltinStatus {
    Handled,
    NotHandled,
    Exit,
}

// A static list of all currently supported builtin commands
pub const BUILTINS: &[&str] = &["exit", "echo", "type"];

pub fn execute(command: &str, args: &[&str]) -> BuiltinStatus {
    match command {
        "exit" => BuiltinStatus::Exit,
        "echo" => echo::execute(args),
        // Route the arguments to the dedicated type_cmd module
        "type" => type_cmd::execute(args),
        _ => BuiltinStatus::NotHandled,
    }
}
