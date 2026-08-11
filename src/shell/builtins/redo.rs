use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, TransactionStatus, parse_transaction_range};
use crate::executor::single::execute as execute_command;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let mut commands_to_run = Vec::new();

    {
        let registry = transaction_registry().lock().unwrap();

        if registry.is_empty() {
            print_output("No transactions recorded in this session.\n", stdout);
            return BuiltinStatus::Handled;
        }

        match parse_transaction_range(args, &registry) {
            Ok(None) => {
                // Default: Find the most recently rolled back transaction
                let mut target = None;
                for (i, tx) in registry.iter().enumerate().rev() {
                    if tx.status == TransactionStatus::RolledBack {
                        target = Some(i);
                        break;
                    }
                }
                if let Some(idx) = target {
                    commands_to_run.push((registry[idx].id.clone(), registry[idx].command_line.clone()));
                } else {
                    print_output("No rolled back transactions found to redo.\n", stdout);
                    return BuiltinStatus::Handled;
                }
            },
            Ok(Some((start, end))) => {
                // Always redo forwards (oldest to newest)
                for i in start..=end {
                    let tx = &registry[i];
                    if tx.status == TransactionStatus::RolledBack {
                        commands_to_run.push((tx.id.clone(), tx.command_line.clone()));
                    }
                }
                if commands_to_run.is_empty() {
                    print_output("No rolled back transactions found in that range.\n", stdout);
                    return BuiltinStatus::Handled;
                }
            },
            Err(e) => {
                print_output(&format!("[CHRONOS] ⚠ {}\n", e), stdout);
                return BuiltinStatus::Handled;
            }
        }
    }

    // Execute the gathered commands outside the lock
    for (id, cmd_line) in commands_to_run {
        print_output(&format!("\n[CHRONOS] Redoing transaction: {}\n", id), stdout);

        let chunk: Vec<String> = cmd_line.split_whitespace().map(|s| s.to_string()).collect();
        execute_command(chunk);
    }

    BuiltinStatus::Handled
}
