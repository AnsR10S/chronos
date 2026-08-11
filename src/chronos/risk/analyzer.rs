use crate::parser::ast::{Command, Redirect};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    ShellStateChange,
    StateChanging,
    Destructive,
    VeryHigh,
    Unknown,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RiskReason {
    ReadOnlyCommand,
    ShellStateCommand,
    StateAlteringCommand,
    DestructiveCommand,
    UnknownCommand,
    RecursiveFlag,
    ForceFlag,
    WildcardTarget,
    RootDirectoryTarget,
    FileOverwrite,
    FileAppend,
    StderrRedirect,
    NetworkCommand,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Effect {
    ReadOnly,
    ShellStateChange,
    FilesystemCreate,
    FilesystemModify,
    FilesystemDelete,
    ProcessSpawn,
    NetworkActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub score: u8,
    pub reasons: Vec<RiskReason>,
    pub effects: Vec<Effect>,
    pub confidence: f32,
}

pub fn analyze_command(cmd: &Command) -> RiskAssessment {
    let mut assessment = match cmd.name.as_str() {
        "ls" | "pwd" | "echo" | "cat" | "grep" | "type" | "history" | "jobs" => RiskAssessment {
            level: RiskLevel::Safe,
            score: 0,
            reasons: vec![RiskReason::ReadOnlyCommand],
            effects: vec![Effect::ReadOnly],
            confidence: 1.0,
        },
        "cd" | "declare" | "export" | "exit" | "transactions" | "undo" | "redo" => RiskAssessment {
            level: RiskLevel::ShellStateChange,
            score: 10,
            reasons: vec![RiskReason::ShellStateCommand],
            effects: vec![Effect::ShellStateChange],
            confidence: 1.0,
        },
        "touch" | "mkdir" => RiskAssessment {
            level: RiskLevel::StateChanging,
            score: 20,
            reasons: vec![RiskReason::StateAlteringCommand],
            effects: vec![Effect::FilesystemCreate],
            confidence: 0.9,
        },
        "cp" | "mv" => RiskAssessment {
            level: RiskLevel::StateChanging,
            score: 30,
            reasons: vec![RiskReason::StateAlteringCommand],
            effects: vec![Effect::FilesystemCreate, Effect::FilesystemModify],
            confidence: 0.9,
        },
        "rm" | "rmdir" => RiskAssessment {
            level: RiskLevel::Destructive,
            score: 70,
            reasons: vec![RiskReason::DestructiveCommand],
            effects: vec![Effect::FilesystemDelete],
            confidence: 0.9,
        },
        "ping" | "curl" | "wget" => RiskAssessment {
            level: RiskLevel::Unknown,
            score: 50,
            reasons: vec![RiskReason::NetworkCommand],
            effects: vec![Effect::NetworkActivity, Effect::ProcessSpawn],
            confidence: 0.8,
        },
        _ => RiskAssessment {
            level: RiskLevel::Unknown,
            score: 50,
            reasons: vec![RiskReason::UnknownCommand],
            effects: vec![Effect::ProcessSpawn],
            confidence: 0.1,
        },
    };

    match &cmd.stdout {
        Redirect::Overwrite(_) | Redirect::Append(_) => {
            if assessment.level == RiskLevel::Safe || assessment.level == RiskLevel::Unknown || assessment.level == RiskLevel::ShellStateChange {
                assessment.level = RiskLevel::StateChanging;
                assessment.score = assessment.score.max(40);
            }
            if let Redirect::Overwrite(_) = &cmd.stdout {
                assessment.reasons.push(RiskReason::FileOverwrite);
            } else {
                assessment.reasons.push(RiskReason::FileAppend);
            }
            if !assessment.effects.contains(&Effect::FilesystemModify) {
                assessment.effects.push(Effect::FilesystemModify);
            }
        },
        _ => {}
    }

    match &cmd.stderr {
        Redirect::Overwrite(_) | Redirect::Append(_) => {
            if assessment.level == RiskLevel::Safe || assessment.level == RiskLevel::Unknown || assessment.level == RiskLevel::ShellStateChange {
                assessment.level = RiskLevel::StateChanging;
                assessment.score = assessment.score.max(30);
            }
            assessment.reasons.push(RiskReason::StderrRedirect);
            if !assessment.effects.contains(&Effect::FilesystemModify) {
                assessment.effects.push(Effect::FilesystemModify);
            }
        },
        _ => {}
    }

    for arg in &cmd.args {
        if arg == "-r" || arg == "-R" || arg == "-rf" || arg == "-fr" {
            if assessment.level == RiskLevel::Destructive {
                assessment.level = RiskLevel::VeryHigh;
                assessment.score = assessment.score.saturating_add(15);
            } else {
                assessment.score = assessment.score.saturating_add(5);
            }
            if !assessment.reasons.contains(&RiskReason::RecursiveFlag) {
                assessment.reasons.push(RiskReason::RecursiveFlag);
            }
        }

        if arg == "-f" || arg == "-rf" || arg == "-fr" {
            if assessment.level == RiskLevel::Destructive || assessment.level == RiskLevel::VeryHigh {
                assessment.score = assessment.score.saturating_add(10);
            }
            if !assessment.reasons.contains(&RiskReason::ForceFlag) {
                assessment.reasons.push(RiskReason::ForceFlag);
            }
        }

        if arg.contains('*') {
            assessment.score = assessment.score.saturating_add(10);
            assessment.confidence *= 0.8;
            if !assessment.reasons.contains(&RiskReason::WildcardTarget) {
                assessment.reasons.push(RiskReason::WildcardTarget);
            }
        }

        if arg == "/" || arg == "/*" {
            if assessment.level == RiskLevel::Destructive || assessment.level == RiskLevel::VeryHigh {
                assessment.level = RiskLevel::VeryHigh;
                assessment.score = 100;
                if !assessment.reasons.contains(&RiskReason::RootDirectoryTarget) {
                    assessment.reasons.push(RiskReason::RootDirectoryTarget);
                }
            }
        }
    }

    assessment.score = assessment.score.min(100);
    assessment
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Redirect;

    fn make_cmd(name: &str, args: Vec<&str>, stdout: Redirect) -> Command {
        Command {
            name: name.to_string(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
            stdout,
            stderr: Redirect::None,
        }
    }

    #[test]
    fn test_risk_matrix() {
        assert_eq!(analyze_command(&make_cmd("ls", vec![], Redirect::None)).level, RiskLevel::Safe);
        assert_eq!(analyze_command(&make_cmd("pwd", vec![], Redirect::None)).level, RiskLevel::Safe);
        assert_eq!(analyze_command(&make_cmd("cd", vec!["project"], Redirect::None)).level, RiskLevel::ShellStateChange);
        assert_eq!(analyze_command(&make_cmd("touch", vec!["a"], Redirect::None)).level, RiskLevel::StateChanging);
        assert_eq!(analyze_command(&make_cmd("mkdir", vec!["project"], Redirect::None)).level, RiskLevel::StateChanging);
        assert_eq!(analyze_command(&make_cmd("cp", vec!["a", "b"], Redirect::None)).level, RiskLevel::StateChanging);
        assert_eq!(analyze_command(&make_cmd("mv", vec!["a", "b"], Redirect::None)).level, RiskLevel::StateChanging);
        assert_eq!(analyze_command(&make_cmd("rm", vec!["a"], Redirect::None)).level, RiskLevel::Destructive);
        assert_eq!(analyze_command(&make_cmd("rm", vec!["-r", "dir"], Redirect::None)).level, RiskLevel::VeryHigh);
        assert_eq!(analyze_command(&make_cmd("rm", vec!["-rf", "dir/*"], Redirect::None)).level, RiskLevel::VeryHigh);
        assert_eq!(analyze_command(&make_cmd("echo", vec!["x"], Redirect::Overwrite("a".to_string()))).level, RiskLevel::StateChanging);
        assert_eq!(analyze_command(&make_cmd("unknown", vec![], Redirect::None)).level, RiskLevel::Unknown);
        assert_eq!(analyze_command(&make_cmd("./script.sh", vec![], Redirect::None)).level, RiskLevel::Unknown);
    }
}
