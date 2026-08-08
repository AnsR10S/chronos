use crate::parser::ast::Command;

#[derive(Debug, PartialEq)]
pub enum RiskLevel {
    Safe,
    StateChanging,
    Destructive,
    Unknown, // Added Unknown for safety
}

#[derive(Debug, PartialEq)]
pub enum RiskReason {
    ReadOnlyCommand,
    StateAlteringCommand,
    DestructiveCommand,
    UnknownCommand,
    // We will add more specific reasons here soon (e.g., RecursiveFlag, WildcardTarget)
}

#[derive(Debug)]
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub score: u8,
    pub reasons: Vec<RiskReason>,
    pub confidence: f32,
}

pub fn analyze_command(cmd: &Command) -> RiskAssessment {
    match cmd.name.as_str() {
        // Harmless read-only or environment commands
        "echo" | "pwd" | "ls" | "cd" | "type" | "history" | "jobs" | "cat" | "grep" => RiskAssessment {
            level: RiskLevel::Safe,
            score: 0,
            reasons: vec![RiskReason::ReadOnlyCommand],
            confidence: 1.0,
        },

        // Commands that create or alter data, but don't blindly destroy it
        "touch" | "mkdir" | "cp" | "mv" | "declare" | "export" => RiskAssessment {
            level: RiskLevel::StateChanging,
            score: 40,
            reasons: vec![RiskReason::StateAlteringCommand],
            confidence: 0.9,
        },

        // Commands that wipe data from the disk
        "rm" | "rmdir" => RiskAssessment {
            level: RiskLevel::Destructive,
            score: 90,
            reasons: vec![RiskReason::DestructiveCommand],
            confidence: 0.9,
        },

        // Default to Unknown (which we treat carefully!)
        _ => RiskAssessment {
            level: RiskLevel::Unknown,
            score: 50,
            reasons: vec![RiskReason::UnknownCommand],
            confidence: 0.1,
        },
    }
}
