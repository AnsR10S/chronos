use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::chronos::risk::analyzer::RiskAssessment;
use crate::chronos::state::tracker::FsTarget;
use crate::chronos::transaction::snapshot::restore_snapshot;

static TX_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
    pub timestamp: u128,
    pub command_line: String,
    #[serde(default)]
    pub chunk: Vec<String>,
    pub assessment: RiskAssessment,
    pub targets: Vec<FsTarget>,
    pub status: TransactionStatus,
}

impl Transaction {
    pub fn new(command_line: String, chunk: Vec<String>, assessment: RiskAssessment, targets: Vec<FsTarget>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let count = TX_COUNTER.fetch_add(1, Ordering::SeqCst);
        let id = format!("tx_{}_{}", timestamp, count);

        Transaction {
            id,
            timestamp,
            command_line,
            chunk,
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
    let _ = fs::create_dir_all(&path);
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
    Vec::new()
}

pub fn transaction_registry() -> &'static Mutex<Vec<Transaction>> {
    static REGISTRY: OnceLock<Mutex<Vec<Transaction>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(load_registry()))
}

pub fn record_transaction(tx: Transaction) {
    {
        if let Ok(mut registry) = transaction_registry().lock() {
            registry.push(tx);
        }
    }
    save_registry();
}

pub fn parse_transaction_targets(args: &[String], registry: &[Transaction]) -> Result<Vec<usize>, String> {
    let mut ids = Vec::new();
    let mut is_cascade = false;
    let mut is_range = false;

    for arg in args {
        if arg == "--cascade" {
            is_cascade = true;
        } else if arg == "--range" {
            is_range = true;
        } else if arg.starts_with("tx_") {
            ids.push(arg.clone());
        }
    }

    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut indices = Vec::new();
    for id in &ids {
        if let Some(pos) = registry.iter().position(|tx| &tx.id == id) {
            indices.push(pos);
        } else {
            return Err(format!("Transaction ID {} not found.", id));
        }
    }

    if is_cascade {
        if indices.len() != 1 { return Err("--cascade requires exactly one transaction ID.".to_string()); }
        let start = indices[0];
        let end = registry.len().saturating_sub(1);
        return Ok((start..=end).collect());
    }

    if is_range {
        if indices.len() != 2 { return Err("--range requires exactly two transaction IDs.".to_string()); }
        let min_idx = *indices.iter().min().unwrap();
        let max_idx = *indices.iter().max().unwrap();
        return Ok((min_idx..=max_idx).collect());
    }

    Ok(indices)
}

pub fn recover_crashed_transactions() {
    let mut registry = transaction_registry().lock().unwrap();
    let mut needs_save = false;

    for tx in registry.iter_mut() {
        if tx.status == TransactionStatus::Executing || tx.status == TransactionStatus::Prepared {
            println!("[CHRONOS] ⚠ CRASH RECOVERY: Found incomplete transaction {} (was {:?})", tx.id, tx.status);

            if !tx.targets.is_empty() {
                match restore_snapshot(&tx.id, &tx.targets) {
                    Ok(_) => println!("[CHRONOS] Successfully rolled back partial state for {}.", tx.id),
                    Err(e) => println!("[CHRONOS] ⚠ Failed to restore partial state for {}: {}", tx.id, e),
                }
            }

            tx.status = TransactionStatus::RolledBack;
            needs_save = true;
        }
    }

    if needs_save {
        drop(registry);
        save_registry();
        println!("[CHRONOS] Crash recovery complete.");
    }
}
