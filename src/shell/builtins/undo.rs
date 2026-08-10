use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, TransactionStatus};
use crate::chronos::transaction::snapshot::restore_snapshot;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let mut registry = transaction_registry().lock().unwrap();

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
        for (i, tx) in registry.iter().enumerate().rev() {
            if &tx.id == req_id {
                target_tx_index = Some(i);
                break;
            }
        }
    } else {
        for (i, tx) in registry.iter().enumerate().rev() {
            if tx.status == TransactionStatus::Committed && !tx.targets.is_empty() {
                target_tx_index = Some(i);
                break;
            }
        }
    }

    if let Some(target_idx) = target_tx_index {
        let start_idx = if is_cascade { registry.len() - 1 } else { target_idx };

        for i in (target_idx..=start_idx).rev() {
            let tx = registry[i].clone();

            if tx.status != TransactionStatus::Committed || tx.targets.is_empty() {
                continue;
            }

            print_output(&format!("\n[CHRONOS] Undoing transaction: {}\n", tx.id), stdout);
            print_output(&format!("[CHRONOS] Original command: {}\n", tx.command_line), stdout);

            match restore_snapshot(&tx.id, &tx.targets) {
                Ok(_) => {
                    print_output("[CHRONOS] Successfully restored files from snapshot.\n", stdout);
                    registry[i].status = TransactionStatus::RolledBack;
                },
                Err(e) => {
                    print_output(&format!("[CHRONOS] ⚠ Failed to restore snapshot: {}\n", e), stdout);
                }
            }
        }
    } else {
        if requested_id.is_some() {
            print_output("Transaction ID not found.\n", stdout);
        } else {
            print_output("No undoable transactions found.\n", stdout);
        }
    }

    BuiltinStatus::Handled
}
