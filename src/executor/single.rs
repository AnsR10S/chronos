use crate::parser::parser;
use crate::parser::ast::Redirect;
use crate::shell::builtins::{self, BuiltinStatus, BUILTINS};
use crate::executor::process;
use crate::executor::expand::expand_args;
use crate::chronos::risk::analyzer::{analyze_command, RiskLevel};
use crate::chronos::state::tracker::track_targets;
use crate::chronos::transaction::manager::{Transaction, TransactionStatus, record_transaction};
use std::fs::{File, OpenOptions};
use std::io::Write;

pub fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

pub fn capture_builtin(command: &str, args: &[String]) -> String {
    match command {
        "echo" => format!("{}\n", args.join(" ")),
        "pwd" => std::env::current_dir().ok().map(|d| format!("{}\n", d.display())).unwrap_or_default(),
        "type" => {
            if let Some(target) = args.get(0) {
                if is_builtin(target) {
                    format!("{} is a shell builtin\n", target)
                } else if let Some(path) = builtins::find_executable(target) {
                    format!("{} is {}\n", target, path)
                } else {
                    format!("{}: not found\n", target)
                }
            } else {
                String::new()
            }
        }
        "history" => {
            let registry = crate::shell::state::history::history_registry().lock().unwrap();
            let total = registry.len();
            let mut limit = total;

            if let Some(arg) = args.get(0) {
                if let Ok(n) = arg.parse::<usize>() {
                    limit = n;
                }
            }

            let start_idx = total.saturating_sub(limit);
            let mut out = String::new();
            for (i, cmd) in registry.iter().enumerate().skip(start_idx) {
                out.push_str(&format!("{:>5}  {}\n", i + 1, cmd));
            }
            out
        }
        _ => String::new()
    }
}

pub fn execute(chunk: Vec<String>) -> bool {
    let command_line = chunk.join(" ");

    if let Some(mut parsed_cmd) = parser::parse(chunk) {

        let mut is_background = false;

        if parsed_cmd.args.last().map(|s| s.as_str()) == Some("&") {
            parsed_cmd.args.pop();
            is_background = true;
        }

        parsed_cmd.args = expand_args(&parsed_cmd.args);

        let assessment = analyze_command(&parsed_cmd);

        let risk_label = match assessment.level {
            RiskLevel::Safe => "\x1b[32mSAFE\x1b[0m",
            RiskLevel::ShellStateChange => "\x1b[36mSHELL-STATE\x1b[0m",
            RiskLevel::StateChanging => "\x1b[33mSTATE-CHANGING\x1b[0m",
            RiskLevel::Destructive => "\x1b[31mDANGEROUS\x1b[0m",
            RiskLevel::VeryHigh => "\x1b[1;31mVERY HIGH\x1b[0m",
            RiskLevel::Unknown => "\x1b[35mUNKNOWN\x1b[0m",
        };

        // Check if this is a meta-command that shouldn't enter the ledger
        let is_meta_command = matches!(parsed_cmd.name.as_str(), "undo" | "transactions" | "exit" | "history");

        // We only print the assessment block for normal commands (keeps output clean when viewing history)
        if !is_meta_command {
            println!("[CHRONOS] Assessed Risk: {} (Score: {}, Confidence: {}%)",
                     risk_label,
                     assessment.score,
                     assessment.confidence * 100.0);
        }

        let mut targets = Vec::new();
        if assessment.level == RiskLevel::StateChanging
            || assessment.level == RiskLevel::Destructive
            || assessment.level == RiskLevel::VeryHigh
        {
            targets = track_targets(&parsed_cmd);
            if !targets.is_empty() && !is_meta_command {
                println!("[CHRONOS] Tracking Filesystem Targets:");
                for target in &targets {
                    let status = if target.exists {
                        let kind = if target.is_dir { "Directory" } else { "File" };
                        let perms = if target.readonly { "Read-Only" } else { "Writable" };
                        format!("Exists ({} - {})", kind, perms)
                    } else {
                        "Does Not Exist".to_string()
                    };
                    println!("  -> {}: {}", target.path, status);
                }
            }
        }

        // Only wrap in a transaction if it's not a meta-command
        let active_tx = if !is_meta_command {
            let mut tx = Transaction::new(command_line, assessment.clone(), targets);
            println!("[CHRONOS] Transaction Created: {}", tx.id);

            if assessment.level == RiskLevel::StateChanging
                || assessment.level == RiskLevel::Destructive
                || assessment.level == RiskLevel::VeryHigh
            {
                println!("[CHRONOS] Securing targets...");
                if let Err(e) = crate::chronos::transaction::snapshot::create_snapshot(&tx.id, &tx.targets) {
                    eprintln!("[CHRONOS] ⚠ WARNING: Failed to create snapshot: {}", e);
                    tx.transition_to(TransactionStatus::Failed);
                } else {
                    tx.transition_to(TransactionStatus::Prepared);
                    println!("[CHRONOS] Snapshot secured successfully.");
                }
            }

            tx.transition_to(TransactionStatus::Executing);
            Some(tx)
        } else {
            None
        };

        match &parsed_cmd.stdout {
            Redirect::None => {}
            Redirect::Overwrite(path) => { let _ = File::create(path); }
            Redirect::Append(path) => { let _ = OpenOptions::new().create(true).append(true).open(path); }
        }

        match &parsed_cmd.stderr {
            Redirect::None => {}
            Redirect::Overwrite(path) => { let _ = File::create(path); }
            Redirect::Append(path) => { let _ = OpenOptions::new().create(true).append(true).open(path); }
        }

        let status = builtins::execute(&parsed_cmd.name, &parsed_cmd.args, &parsed_cmd.stdout);

        if status == BuiltinStatus::NotHandled {
            if builtins::find_executable(&parsed_cmd.name).is_some() {
                if is_background {
                    process::run_background(&parsed_cmd.name, &parsed_cmd.args, &parsed_cmd.stdout, &parsed_cmd.stderr);
                } else {
                    process::run_external(&parsed_cmd.name, &parsed_cmd.args, &parsed_cmd.stdout, &parsed_cmd.stderr);
                }
            } else {
                let error_msg = format!("{}: command not found\n", parsed_cmd.name);
                match &parsed_cmd.stderr {
                    Redirect::None => eprint!("{}", error_msg),
                    Redirect::Overwrite(path) => {
                        if let Ok(mut f) = File::create(path) { let _ = f.write_all(error_msg.as_bytes()); }
                    }
                    Redirect::Append(path) => {
                        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) { let _ = f.write_all(error_msg.as_bytes()); }
                    }
                }
            }
        }

        // Only commit the transaction if one was actually created
        if let Some(mut tx) = active_tx {
            tx.transition_to(TransactionStatus::Committed);
            println!("[CHRONOS] Transaction Committed.");
            record_transaction(tx);
        }

        match status {
            BuiltinStatus::Exit => return true,
            _ => return false,
        }
    }
    false
}
