use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::{transaction_registry, TransactionStatus};
use crate::chronos::risk::analyzer::RiskLevel;

pub fn execute(_args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let registry = transaction_registry().lock().unwrap();

    if registry.is_empty() {
        print_output("No transactions recorded in this session.\n", stdout);
        return BuiltinStatus::Handled;
    }

    // Build the Table Header using Unicode Box Drawing characters
    let mut output = String::new();
    output.push_str("\n");
    output.push_str(&format!(" {:<22} │ {:<12} │ {:<15} │ {}\n", "\x1b[1mTRANSACTION ID\x1b[0m", "\x1b[1mSTATUS\x1b[0m", "\x1b[1mRISK\x1b[0m", "\x1b[1mCOMMAND\x1b[0m"));
    output.push_str("────────────────────────┼──────────────┼─────────────────┼────────────────────────────────────\n");

    // Iterate and format each row
    for tx in registry.iter() {
        // Color code the status
        let (status_color, raw_status) = match tx.status {
            TransactionStatus::Committed => ("\x1b[32m", "COMMITTED"),    // Green
            TransactionStatus::RolledBack => ("\x1b[33m", "ROLLED BACK"), // Yellow
            TransactionStatus::Failed => ("\x1b[31m", "FAILED"),          // Red
            _ => ("\x1b[36m", "PENDING"),                                 // Cyan
        };

        // Pad the raw string first, then inject the color so alignment doesn't break
        let padded_status = format!("{:<12}", raw_status);
        let colored_status = padded_status.replace(raw_status, &format!("{}{}\x1b[0m", status_color, raw_status));

        // Color code the risk
        let (risk_color, raw_risk) = match tx.assessment.level {
            RiskLevel::Safe => ("\x1b[32m", "Safe"),
            RiskLevel::ShellStateChange => ("\x1b[36m", "Shell State"),
            RiskLevel::StateChanging => ("\x1b[33m", "State Changing"),
            RiskLevel::Destructive => ("\x1b[31m", "Destructive"),
            RiskLevel::VeryHigh => ("\x1b[1;31m", "VERY HIGH"),
            RiskLevel::Unknown => ("\x1b[35m", "Unknown"),
        };

        let padded_risk = format!("{:<15}", raw_risk);
        let colored_risk = padded_risk.replace(raw_risk, &format!("{}{}\x1b[0m", risk_color, raw_risk));

        // Append the row
        output.push_str(&format!(" {:<22} │ {} │ {} │ {}\n",
            tx.id,
            colored_status,
            colored_risk,
            tx.command_line
        ));
    }

    output.push_str("\n");
    print_output(&output, stdout);

    BuiltinStatus::Handled
}
