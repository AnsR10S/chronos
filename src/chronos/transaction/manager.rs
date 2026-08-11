use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Mutex, OnceLock};
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::chronos::risk::analyzer::RiskAssessment;
use crate::chronos::state::tracker::FsTarget;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Prepared,
    Executing,
    Committed,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub timestamp: u128, // Changed from u64 to u128 to hold milliseconds
    pub command_line: String,
    pub assessment: RiskAssessment,
    pub targets: Vec<FsTarget>,
    pub status: TransactionStatus,
}

impl Transaction {
    pub fn new(command_line: String, assessment: RiskAssessment, targets: Vec<FsTarget>) -> Self {
        // Generates timestamp in milliseconds to prevent rapid-fire ID collisions
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let id = format!("tx_{}", timestamp);

        Transaction {
            id,
            timestamp,
            command_line,
            assessment,
            targets,
            status: TransactionStatus::Pending,
        }
    }

    pub fn transition_to(&mut self, new_status: TransactionStatus) {
        self.status = new_status;
    }
}

fn get_history_file() -> Option<PathBuf> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()?;
    let mut path = PathBuf::from(home);
    path.push(".chronos");
    let _ = fs::create_dir_all(&path); // Ensure the hidden directory exists
    path.push("history.json");
    Some(path)
}

pub fn save_registry() {
    if let Some(path) = get_history_file() {
        if let Ok(registry) = transaction_registry().lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*registry) {
                let _ = fs::write(path, json);
            }
        }
    }
}

fn load_registry() -> Vec<Transaction> {
    if let Some(path) = get_history_file() {
        if let Ok(json) = fs::read_to_string(path) {
            if let Ok(registry) = serde_json::from_str(&json) {
                return registry;
            }
        }
    }
    Vec::new() // Return an empty ledger if the file doesn't exist yet
}

// The OnceLock now calls load_registry() the very first time it is accessed on boot
pub fn transaction_registry() -> &'static Mutex<Vec<Transaction>> {
    static REGISTRY: OnceLock<Mutex<Vec<Transaction>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(load_registry()))
}

pub fn record_transaction(tx: Transaction) {
    {
        if let Ok(mut registry) = transaction_registry().lock() {
            registry.push(tx);
        }
    } // Lock is dropped here so saving doesn't deadlock!

    save_registry(); // Auto-save to disk instantly
}
