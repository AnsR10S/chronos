use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, TransactionStatus, parse_transaction_targets};
use crate::executor::single::execute as execute_command;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let mut commands_to_run = Vec::new();

    {
        let registry = transaction_registry().lock().unwrap();

        if registry.is_empty() {
            print_output("No transactions recorded in this session.\n", stdout);
            return BuiltinStatus::Handled;
        }

        let mut indices = match parse_transaction_targets(args, &registry) {
            Ok(vec) => vec,
            Err(e) => {
                print_output(&format!("[CHRONOS] ⚠ {}\n", e), stdout);
                return BuiltinStatus::Handled;
            }
        };

        if indices.is_empty() {
            for (i, tx) in registry.iter().enumerate().rev() {
                if tx.status == TransactionStatus::RolledBack {
                    indices.push(i);
                    break;
                }
            }
            if indices.is_empty() {
                print_output("No rolled back transactions found to redo.\n", stdout);
                return BuiltinStatus::Handled;
            }
        }

        // Redo strictly chronologically
        indices.sort();

        for i in indices {
            let tx = &registry[i];
            if tx.status == TransactionStatus::RolledBack {
                // Pull the perfectly preserved token chunk, not the raw command string
                commands_to_run.push((tx.id.clone(), tx.chunk.clone()));
            }
        }

        if commands_to_run.is_empty() {
            print_output("No valid rolled back transactions found in selection.\n", stdout);
            return BuiltinStatus::Handled;
        }
    } // Lock drops

    for (id, chunk) in commands_to_run {
        print_output(&format!("\n[CHRONOS] Redoing transaction: {}\n", id), stdout);
        execute_command(chunk);
    }

    BuiltinStatus::Handled
}
