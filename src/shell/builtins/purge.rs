use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, save_registry};
use std::fs;
use std::path::PathBuf;

pub fn execute(_args: &[String], stdout: &Redirect) -> BuiltinStatus {
    // Wipes the in-memory ledger
    {
        let mut registry = transaction_registry().lock().unwrap();
        registry.clear();
    }

    // Overwrites history.json with the empty ledger
    save_registry();

    // Locates and deletes the physical snapshots folder
    let mut snap_dir = PathBuf::new();
    if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok() {
        snap_dir.push(home);
        snap_dir.push(".chronos");
        snap_dir.push("snapshots");

        if snap_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&snap_dir) {
                print_output(&format!("[CHRONOS] ⚠ Failed to delete physical snapshots: {}\n", e), stdout);
                return BuiltinStatus::Handled;
            }
        }
    }

    print_output("[CHRONOS] Transaction history and physical snapshots successfully purged.\n", stdout);
    BuiltinStatus::Handled
}
