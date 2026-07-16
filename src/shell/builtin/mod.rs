// Represents the execution state of a builtin command.
// This allows the REPL to know exactly how to proceed after a command is evaluated.
pub enum BuiltinStatus {
    Handled,
    NotHandled,
    Exit,
}

// Evaluates the raw command string to determine if it is a builtin.
pub fn execute(command: &str) -> BuiltinStatus {
    match command {
        // If the command is exactly "exit", signal the REPL to terminate
        "exit" => BuiltinStatus::Exit,
        // Fallback for anything else we don't recognize yet
        _ => BuiltinStatus::NotHandled,
    }
}
