use crate::parser::parser;
use crate::parser::ast::Redirect;
use crate::shell::builtins::{self, BuiltinStatus, BUILTINS};
use crate::executor::process;
use crate::executor::expand::expand_args;
use crate::chronos::risk::analyzer::{analyze_command, RiskLevel};
use crate::chronos::state::tracker::track_targets;
use crate::chronos::transaction::manager::{Transaction, TransactionStatus, record_transaction};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};


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

    if let Some(mut parsed_cmd) = parser::parse(chunk.clone()) {

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

        let is_meta_command = matches!(parsed_cmd.name.as_str(), "undo" | "redo" | "transactions" | "history" | "exit" | "purge");

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

        if !is_meta_command && (assessment.level == RiskLevel::Unknown || assessment.level == RiskLevel::VeryHigh) {
            let is_loading = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
            let loading_clone = is_loading.clone();

            let spinner = std::thread::spawn(move || {
                let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let mut i = 0;
                while loading_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    print!("\r[CHRONOS] \x1b[35mConsulting AI Semantic Layer... {}\x1b[0m", frames[i]);
                    let _ = io::stdout().flush();
                    i = (i + 1) % frames.len();
                    std::thread::sleep(std::time::Duration::from_millis(80));
                }
                print!("\r\x1b[2K");
                let _ = io::stdout().flush();
            });

            // Use the global runtime instead of creating a new one
            let rt = crate::chronos::ai::client::ASYNC_RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().unwrap());
            let ai_result = rt.block_on(crate::chronos::ai::client::analyze_command(&command_line, &assessment, &targets));

            is_loading.store(false, std::sync::atomic::Ordering::Relaxed);
            let _ = spinner.join();

            match ai_result {
                Ok(ai_response) => {
                    println!("\n┌──────────────── \x1b[1mAI SEMANTIC ANALYSIS\x1b[0m ────────────────┐");
                    println!("│ \x1b[36mIntent:\x1b[0m {}", ai_response.intent);
                    println!("│ \x1b[36mExplanation:\x1b[0m {}", ai_response.explanation);
                    println!("│ \x1b[36mRecommendation:\x1b[0m {:?}", ai_response.recommendation);
                    println!("└──────────────────────────────────────────────────────┘\n");

                    // Structured decision handling based on the Enum
                    if ai_response.recommendation == crate::chronos::ai::client::AIRecommendation::Block {
                        println!("[CHRONOS] \x1b[31mCommand blocked by AI Semantic Layer recommendation.\x1b[0m");
                        return false;
                    }

                    if ai_response.recommendation == crate::chronos::ai::client::AIRecommendation::Escalate
                        || assessment.level == RiskLevel::VeryHigh
                    {
                        print!("[CHRONOS] \x1b[33mAI Escalate / Very High Risk. Proceed with execution? [y/N]: \x1b[0m");
                        let _ = io::stdout().flush();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();

                        if !input.trim().eq_ignore_ascii_case("y") {
                            println!("[CHRONOS] Command blocked by user.");
                            return false;
                        }
                    }
                }
                Err(e) => {
                    println!("[CHRONOS] \x1b[31m⚠ AI Analysis Failed (Timeout/Network): {}\x1b[0m", e);

                    // AI Failure Policy Enforcement
                    if assessment.level == RiskLevel::VeryHigh {
                        println!("[CHRONOS] \x1b[31mPOLICY: Fail-Closed. Very High risk commands are blocked when AI is unavailable.\x1b[0m");
                        return false;
                    } else {
                        print!("[CHRONOS] \x1b[33mPOLICY: Escalate. Unknown command with no AI guidance. Proceed? [y/N]: \x1b[0m");
                        let _ = io::stdout().flush();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        if !input.trim().eq_ignore_ascii_case("y") {
                            println!("[CHRONOS] Command blocked by user.");
                            return false;
                        }
                    }
                }
            }
        }

        let active_tx = if !is_meta_command {
            let mut tx = Transaction::new(command_line, chunk.clone(), assessment.clone(), targets);
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
