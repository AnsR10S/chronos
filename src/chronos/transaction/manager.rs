use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Mutex, OnceLock}; 
use crate::chronos::risk::analyzer::RiskAssessment;
use crate::chronos::state::tracker::FsTarget;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Prepared,
    Executing,
    Committed,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub timestamp: u64,
    pub command_line: String,
    pub assessment: RiskAssessment,
    pub targets: Vec<FsTarget>,
    pub status: TransactionStatus,
}

impl Transaction {
    pub fn new(command_line: String, assessment: RiskAssessment, targets: Vec<FsTarget>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

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

pub fn transaction_registry() -> &'static Mutex<Vec<Transaction>> {
    static REGISTRY: OnceLock<Mutex<Vec<Transaction>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn record_transaction(tx: Transaction) {
    if let Ok(mut registry) = transaction_registry().lock() {
        registry.push(tx);
    }
}
