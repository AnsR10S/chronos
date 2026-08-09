use crate::parser::ast::Command;

#[derive(Debug, PartialEq)]
pub enum RiskLevel {
    Safe,
    StateChanging,
    Destructive,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum RiskReason {
    ReadOnlyCommand,
    StateAlteringCommand,
    DestructiveCommand,
    UnknownCommand,
    // Dynamic reasons based on argument scanning
    RecursiveFlag,
    ForceFlag,
    WildcardTarget,
    RootDirectoryTarget,
}

#[derive(Debug)]
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub score: u8,
    pub reasons: Vec<RiskReason>,
    pub confidence: f32,
}

pub fn analyze_command(cmd: &Command) -> RiskAssessment {
    // Establishes the baseline risk based on the command name
    let mut assessment = match cmd.name.as_str() {
        "echo" | "pwd" | "ls" | "cd" | "type" | "history" | "jobs" | "cat" | "grep" => RiskAssessment {
            level: RiskLevel::Safe,
            score: 0,
            reasons: vec![RiskReason::ReadOnlyCommand],
            confidence: 1.0,
        },

        "touch" | "mkdir" | "cp" | "mv" | "declare" | "export" => RiskAssessment {
            level: RiskLevel::StateChanging,
            score: 30, // Lowered base score
            reasons: vec![RiskReason::StateAlteringCommand],
            confidence: 0.9,
        },

        "rm" | "rmdir" => RiskAssessment {
            level: RiskLevel::Destructive,
            score: 70, // Lowered base score (a single file deletion isn't the end of the world)
            reasons: vec![RiskReason::DestructiveCommand],
            confidence: 0.9,
        },

        _ => RiskAssessment {
            level: RiskLevel::Unknown,
            score: 50,
            reasons: vec![RiskReason::UnknownCommand],
            confidence: 0.1,
        },
    };

    // Dynamically scans the arguments to adjust the score and confidence
    for arg in &cmd.args {
        // Check for recursive flags
        if arg == "-r" || arg == "-R" || arg == "-rf" || arg == "-fr" {
            if assessment.level == RiskLevel::Destructive {
                assessment.score = assessment.score.saturating_add(15);
            } else {
                assessment.score = assessment.score.saturating_add(5);
            }
            if !assessment.reasons.contains(&RiskReason::RecursiveFlag) {
                assessment.reasons.push(RiskReason::RecursiveFlag);
            }
        }

        // Check for force flags
        if arg == "-f" || arg == "-rf" || arg == "-fr" {
            if assessment.level == RiskLevel::Destructive {
                assessment.score = assessment.score.saturating_add(10);
            }
            if !assessment.reasons.contains(&RiskReason::ForceFlag) {
                assessment.reasons.push(RiskReason::ForceFlag);
            }
        }

        // Check for wildcards (reduces confidence because we don't know the exact targets)
        if arg.contains('*') {
            assessment.score = assessment.score.saturating_add(10);
            assessment.confidence *= 0.8;
            if !assessment.reasons.contains(&RiskReason::WildcardTarget) {
                assessment.reasons.push(RiskReason::WildcardTarget);
            }
        }

        // Check for catastrophic root targets
        if arg == "/" || arg == "/*" {
            if assessment.level == RiskLevel::Destructive {
                assessment.score = 100; // Max out the danger score
                if !assessment.reasons.contains(&RiskReason::RootDirectoryTarget) {
                    assessment.reasons.push(RiskReason::RootDirectoryTarget);
                }
            }
        }
    }

    // Ensures the score never exceeds 100
    assessment.score = assessment.score.min(100);

    assessment
}
