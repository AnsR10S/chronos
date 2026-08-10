use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, TransactionStatus};
use crate::chronos::transaction::snapshot::restore_snapshot;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let mut registry = transaction_registry().lock().unwrap();

    // Find the target transaction index
    let mut target_tx_index = None;

    if let Some(requested_id) = args.get(0) {
        // If the user specifies a transaction ID (e.g., `undo tx_12345`)
        for (i, tx) in registry.iter().enumerate().rev() {
            if &tx.id == requested_id {
                target_tx_index = Some(i);
                break;
            }
        }
    } else {
        // Otherwise, find the most recent transaction that actually had filesystem targets
        for (i, tx) in registry.iter().enumerate().rev() {
            if tx.status == TransactionStatus::Committed && !tx.targets.is_empty() {
                target_tx_index = Some(i);
                break;
            }
        }
    }

    if let Some(idx) = target_tx_index {
        let tx = registry[idx].clone();

        if tx.status == TransactionStatus::RolledBack {
            print_output("Transaction is already rolled back.\n", stdout);
            return BuiltinStatus::Handled;
        }

        print_output(&format!("[CHRONOS] Undoing transaction: {}\n", tx.id), stdout);
        print_output(&format!("[CHRONOS] Original command: {}\n", tx.command_line), stdout);

        match restore_snapshot(&tx.id, &tx.targets) {
            Ok(_) => {
                print_output("[CHRONOS] Successfully restored files from snapshot.\n", stdout);
                registry[idx].status = TransactionStatus::RolledBack; // Update the global registry
            },
            Err(e) => {
                print_output(&format!("[CHRONOS] ⚠ Failed to restore snapshot: {}\n", e), stdout);
            }
        }
    } else {
        print_output("No undoable transactions found.\n", stdout);
    }

    BuiltinStatus::Handled
}
