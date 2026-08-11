use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, save_registry, parse_transaction_range};
use std::fs;
use std::path::PathBuf;

pub fn execute(args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let mut home_dir = PathBuf::new();
    if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok() {
        home_dir.push(home);
        home_dir.push(".chronos");
        home_dir.push("snapshots");
    }

    {
        let mut registry = transaction_registry().lock().unwrap();

        match parse_transaction_range(args, &registry) {
            Ok(None) => {
                // Default: Nuclear Purge
                registry.clear();
                if home_dir.exists() {
                    let _ = fs::remove_dir_all(&home_dir);
                }
                print_output("[CHRONOS] All transaction history and snapshots successfully purged.\n", stdout);
            },
            Ok(Some((start, end))) => {
                // Precision Purge: Loop backwards so removing items doesn't shift the indices
                let mut count = 0;
                for i in (start..=end).rev() {
                    let tx_id = registry[i].id.clone();

                    // Delete specific physical folder
                    if home_dir.exists() {
                        let tx_dir = home_dir.join(&tx_id);
                        if tx_dir.exists() {
                            let _ = fs::remove_dir_all(&tx_dir);
                        }
                    }
                    // Remove from ledger
                    registry.remove(i);
                    count += 1;
                }
                print_output(&format!("[CHRONOS] Purged {} specific transaction(s) from history.\n", count), stdout);
            },
            Err(e) => {
                print_output(&format!("[CHRONOS] ⚠ {}\n", e), stdout);
            }
        }
    }

    save_registry();
    BuiltinStatus::Handled
}
