use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{Transaction, transaction_registry, TransactionStatus, save_registry, parse_transaction_range};
use crate::chronos::transaction::snapshot::restore_snapshot;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    {
        let mut registry = transaction_registry().lock().unwrap();

        if registry.is_empty() {
            print_output("No transactions recorded in this session.\n", stdout);
            return BuiltinStatus::Handled;
        }

        match parse_transaction_range(args, &registry) {
            Ok(None) => {
                // Default: Find the most recent valid commit
                let mut target = None;
                for (i, tx) in registry.iter().enumerate().rev() {
                    if tx.status == TransactionStatus::Committed && !tx.targets.is_empty() {
                        target = Some(i);
                        break;
                    }
                }
                if let Some(idx) = target {
                    undo_range(&mut registry, idx, idx, stdout);
                } else {
                    print_output("No undoable transactions found.\n", stdout);
                }
            },
            Ok(Some((start, end))) => {
                undo_range(&mut registry, start, end, stdout);
            },
            Err(e) => {
                print_output(&format!("[CHRONOS] ⚠ {}\n", e), stdout);
            }
        }
    }

    save_registry();
    BuiltinStatus::Handled
}

// Helper function to keep the logic clean
fn undo_range(registry: &mut Vec<Transaction>, start: usize, end: usize, stdout: &Redirect) {
    let mut count = 0;

    // Always undo backwards (latest to oldest) to prevent file collision issues
    for i in (start..=end).rev() {
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
                count += 1;
            },
            Err(e) => {
                print_output(&format!("[CHRONOS] ⚠ Failed to restore snapshot: {}\n", e), stdout);
            }
        }
    }

    if count == 0 {
        print_output("No undoable transactions found in that range.\n", stdout);
    }
}
