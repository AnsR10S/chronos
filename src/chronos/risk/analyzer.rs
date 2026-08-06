// The core vocabulary of the Chronos Risk Engine
#[derive(Debug, PartialEq)]
pub enum RiskLevel {
    Safe,
    StateChanging,
    Destructive,
}

pub fn analyze_command(command_name: &str) -> RiskLevel {
    match command_name {
        // Harmless read-only or environment commands
        "echo" | "pwd" | "ls" | "cd" | "type" | "history" | "jobs" | "cat" | "grep" => RiskLevel::Safe,

        // Commands that create or alter data, but don't blindly destroy it
        "touch" | "mkdir" | "cp" | "mv" | "declare" | "export" => RiskLevel::StateChanging,

        // Commands that wipe data from the disk
        "rm" | "rmdir" => RiskLevel::Destructive,

        // Default to Safe for unknown external commands for now (we'll tighten this later!)
        _ => RiskLevel::Safe,
    }
}
