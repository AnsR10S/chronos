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
        "cd" | "declare" | "export" | "exit" | "transactions" | "undo" | "redo" | "purge" => RiskAssessment {
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

    // --- Score value tests ---

    #[test]
    fn test_safe_command_score_zero() {
        let a = analyze_command(&make_cmd("ls", vec![], Redirect::None));
        assert_eq!(a.score, 0);
        assert!((a.confidence - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_shell_state_score() {
        let a = analyze_command(&make_cmd("cd", vec!["dir"], Redirect::None));
        assert_eq!(a.score, 10);
    }

    #[test]
    fn test_creation_score() {
        let a = analyze_command(&make_cmd("touch", vec!["f"], Redirect::None));
        assert_eq!(a.score, 20);
    }

    #[test]
    fn test_mutation_score() {
        let a = analyze_command(&make_cmd("cp", vec!["a", "b"], Redirect::None));
        assert_eq!(a.score, 30);
    }

    #[test]
    fn test_destructive_base_score() {
        let a = analyze_command(&make_cmd("rm", vec!["file"], Redirect::None));
        assert_eq!(a.score, 70);
    }

    #[test]
    fn test_unknown_command_score() {
        let a = analyze_command(&make_cmd("./unknown", vec![], Redirect::None));
        assert_eq!(a.score, 50);
    }

    // --- Confidence tests ---

    #[test]
    fn test_unknown_command_low_confidence() {
        let a = analyze_command(&make_cmd("./random_script", vec![], Redirect::None));
        assert!(a.confidence < 0.5, "Unknown commands should have low confidence, got {}", a.confidence);
    }

    #[test]
    fn test_safe_command_full_confidence() {
        let a = analyze_command(&make_cmd("echo", vec!["hi"], Redirect::None));
        assert!((a.confidence - 1.0).abs() < f32::EPSILON);
    }

    // --- Redirection escalation tests ---

    #[test]
    fn test_stdout_overwrite_escalates_safe() {
        let a = analyze_command(&make_cmd("echo", vec!["data"], Redirect::Overwrite("out.txt".into())));
        assert_eq!(a.level, RiskLevel::StateChanging);
        assert!(a.score >= 40);
    }

    #[test]
    fn test_stdout_append_escalates() {
        let a = analyze_command(&make_cmd("echo", vec!["data"], Redirect::Append("log.txt".into())));
        assert_eq!(a.level, RiskLevel::StateChanging);
    }

    #[test]
    fn test_stderr_redirect_escalates() {
        let cmd = Command {
            name: "echo".to_string(),
            args: vec!["hi".to_string()],
            stdout: Redirect::None,
            stderr: Redirect::Overwrite("err.txt".into()),
        };
        let a = analyze_command(&cmd);
        assert_eq!(a.level, RiskLevel::StateChanging);
    }

    // --- Flag detection tests ---

    #[test]
    fn test_recursive_flag_escalates_rm() {
        let a = analyze_command(&make_cmd("rm", vec!["-R", "dir"], Redirect::None));
        assert_eq!(a.level, RiskLevel::VeryHigh);
        assert!(a.reasons.contains(&RiskReason::RecursiveFlag));
    }

    #[test]
    fn test_force_flag_detected() {
        let a = analyze_command(&make_cmd("rm", vec!["-f", "file"], Redirect::None));
        assert!(a.reasons.contains(&RiskReason::ForceFlag));
    }

    #[test]
    fn test_combined_rf_flag() {
        let a = analyze_command(&make_cmd("rm", vec!["-rf", "dir"], Redirect::None));
        assert!(a.reasons.contains(&RiskReason::RecursiveFlag));
        assert!(a.reasons.contains(&RiskReason::ForceFlag));
    }

    #[test]
    fn test_fr_flag_order() {
        let a = analyze_command(&make_cmd("rm", vec!["-fr", "dir"], Redirect::None));
        assert_eq!(a.level, RiskLevel::VeryHigh);
    }

    // --- Wildcard tests ---

    #[test]
    fn test_wildcard_increases_score() {
        let base = analyze_command(&make_cmd("rm", vec!["file"], Redirect::None));
        let wild = analyze_command(&make_cmd("rm", vec!["*"], Redirect::None));
        assert!(wild.score > base.score);
        assert!(wild.reasons.contains(&RiskReason::WildcardTarget));
    }

    #[test]
    fn test_wildcard_reduces_confidence() {
        let base = analyze_command(&make_cmd("rm", vec!["file"], Redirect::None));
        let wild = analyze_command(&make_cmd("rm", vec!["*"], Redirect::None));
        assert!(wild.confidence < base.confidence);
    }

    // --- Root directory tests ---

    #[test]
    fn test_root_directory_maxes_score() {
        let a = analyze_command(&make_cmd("rm", vec!["-rf", "/"], Redirect::None));
        assert_eq!(a.level, RiskLevel::VeryHigh);
        assert_eq!(a.score, 100);
        assert!(a.reasons.contains(&RiskReason::RootDirectoryTarget));
    }

    #[test]
    fn test_root_glob_maxes_score() {
        let a = analyze_command(&make_cmd("rm", vec!["-rf", "/*"], Redirect::None));
        assert_eq!(a.score, 100);
    }

    // --- Network command tests ---

    #[test]
    fn test_network_command_classification() {
        for cmd_name in &["ping", "curl", "wget"] {
            let a = analyze_command(&make_cmd(cmd_name, vec!["example.com"], Redirect::None));
            assert_eq!(a.level, RiskLevel::Unknown, "Failed for {}", cmd_name);
            assert!(a.effects.contains(&Effect::NetworkActivity));
        }
    }

    // --- Effects tests ---

    #[test]
    fn test_safe_command_readonly_effect() {
        let a = analyze_command(&make_cmd("cat", vec!["file"], Redirect::None));
        assert!(a.effects.contains(&Effect::ReadOnly));
    }

    #[test]
    fn test_rm_has_delete_effect() {
        let a = analyze_command(&make_cmd("rm", vec!["file"], Redirect::None));
        assert!(a.effects.contains(&Effect::FilesystemDelete));
    }

    #[test]
    fn test_touch_has_create_effect() {
        let a = analyze_command(&make_cmd("touch", vec!["new_file"], Redirect::None));
        assert!(a.effects.contains(&Effect::FilesystemCreate));
    }

    #[test]
    fn test_cp_has_create_and_modify_effects() {
        let a = analyze_command(&make_cmd("cp", vec!["a", "b"], Redirect::None));
        assert!(a.effects.contains(&Effect::FilesystemCreate));
        assert!(a.effects.contains(&Effect::FilesystemModify));
    }

    // --- Score clamping ---

    #[test]
    fn test_score_never_exceeds_100() {
        let a = analyze_command(&make_cmd("rm", vec!["-rf", "/", "*", "-f"], Redirect::Overwrite("x".into())));
        assert!(a.score <= 100, "Score {} exceeded 100", a.score);
    }

    // --- All read-only commands ---

    #[test]
    fn test_all_safe_commands() {
        for name in &["ls", "pwd", "echo", "cat", "grep", "type", "history", "jobs"] {
            let a = analyze_command(&make_cmd(name, vec![], Redirect::None));
            assert_eq!(a.level, RiskLevel::Safe, "Expected Safe for '{}'", name);
        }
    }

    // --- All shell-state commands ---

    #[test]
    fn test_all_shell_state_commands() {
        for name in &["cd", "declare", "export", "exit", "transactions", "undo", "redo", "purge"] {
            let a = analyze_command(&make_cmd(name, vec![], Redirect::None));
            assert_eq!(a.level, RiskLevel::ShellStateChange, "Expected ShellStateChange for '{}'", name);
        }
    }
}
