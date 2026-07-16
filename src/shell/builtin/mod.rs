// Declare the new echo module
pub mod echo;

pub enum BuiltinStatus {
    Handled,
    NotHandled,
    Exit,
}

pub fn execute(command: &str, args: &[&str]) -> BuiltinStatus {
    match command {
        "exit" => BuiltinStatus::Exit,
        // Route the arguments to the dedicated echo module
        "echo" => echo::execute(args),
        _ => BuiltinStatus::NotHandled,
    }
}
