use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, TransactionStatus, save_registry, parse_transaction_targets};
use crate::chronos::transaction::snapshot::restore_snapshot;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    {
        let mut registry = transaction_registry().lock().unwrap();

        if registry.is_empty() {
            print_output("No transactions recorded in this session.\n", stdout);
            return BuiltinStatus::Handled;
        }

        let is_cascade = args.iter().any(|a| a == "--cascade");
        let mut indices = match parse_transaction_targets(args, &registry) {
            Ok(vec) => vec,
            Err(e) => {
                print_output(&format!("[CHRONOS] ⚠ {}\n", e), stdout);
                return BuiltinStatus::Handled;
            }
        };

        if indices.is_empty() {
            for (i, tx) in registry.iter().enumerate().rev() {
                if tx.status == TransactionStatus::Committed && !tx.targets.is_empty() {
                    indices.push(i);
                    break;
                }
            }
            if indices.is_empty() {
                print_output("No undoable transactions found.\n", stdout);
                return BuiltinStatus::Handled;
            }
        }

        if !is_cascade {
            let mut latest_committed = None;
            for (i, tx) in registry.iter().enumerate().rev() {
                if tx.status == TransactionStatus::Committed {
                    latest_committed = Some(i);
                    break;
                }
            }

            let max_target = *indices.iter().max().unwrap();
            if let Some(latest) = latest_committed {
                if latest > max_target {
                    print_output("[CHRONOS] ⚠ ERROR: Transaction has newer committed descendants.\n", stdout);
                    print_output("[CHRONOS] Undoing it alone may invalidate later state. Use `undo --cascade <id>`.\n", stdout);
                    return BuiltinStatus::Handled;
                }
            }
        }

        indices.sort();
        indices.reverse();

        let mut count = 0;
        for i in indices {
            let tx = registry[i].clone();

            if tx.status != TransactionStatus::Committed || tx.targets.is_empty() {
                continue;
            }

            print_output(&format!("\n[CHRONOS] Undoing transaction: {}\n", tx.id), stdout);

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
            print_output("No undoable transactions found in selection.\n", stdout);
        }
    }

    save_registry();
    BuiltinStatus::Handled
}
