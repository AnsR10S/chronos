use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, TransactionStatus};
use crate::executor::single::execute as execute_command;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let mut commands_to_run = Vec::new();

    // Opens the vault and find what we need to redo
    {
        let registry = transaction_registry().lock().unwrap();

        if registry.is_empty() {
            print_output("No transactions recorded in this session.\n", stdout);
            return BuiltinStatus::Handled;
        }

        let mut is_cascade = false;
        let mut requested_id = None;

        for arg in args {
            if arg == "--cascade" {
                is_cascade = true;
            } else if arg.starts_with("tx_") {
                requested_id = Some(arg.clone());
            }
        }

        let mut target_tx_index = None;

        if let Some(ref req_id) = requested_id {
            // Find specific ID
            for (i, tx) in registry.iter().enumerate() {
                if &tx.id == req_id {
                    target_tx_index = Some(i);
                    break;
                }
            }
        } else {
            // Find most recently rolled back transaction
            for (i, tx) in registry.iter().enumerate().rev() {
                if tx.status == TransactionStatus::RolledBack {
                    target_tx_index = Some(i);
                    break;
                }
            }
        }

        if let Some(target_idx) = target_tx_index {
            // Determine the range to redo
            let end_idx = if is_cascade { registry.len() - 1 } else { target_idx };

            // Redo in chronological order (oldest to newest)
            for i in target_idx..=end_idx {
                let tx = &registry[i];
                if tx.status == TransactionStatus::RolledBack {
                    commands_to_run.push((tx.id.clone(), tx.command_line.clone()));
                }
            }
        } else {
            if requested_id.is_some() {
                print_output("Transaction ID not found.\n", stdout);
            } else {
                print_output("No rolled back transactions found to redo.\n", stdout);
            }
            return BuiltinStatus::Handled;
        }
    } // THE LOCK IS DROPPED HERE! Very important to prevent deadlocks.

    // Executes the extracted commands
    for (id, cmd_line) in commands_to_run {
        print_output(&format!("\n[CHRONOS] Redoing transaction: {}\n", id), stdout);

        // Split the raw string back into a chunk vector for the executor
        let chunk: Vec<String> = cmd_line.split_whitespace().map(|s| s.to_string()).collect();
        execute_command(chunk);
    }

    BuiltinStatus::Handled
}
