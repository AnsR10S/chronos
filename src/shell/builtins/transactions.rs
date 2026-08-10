use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};
use crate::chronos::transaction::manager::transaction_registry;

pub fn execute(_args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let registry = transaction_registry().lock().unwrap();

    if registry.is_empty() {
        print_output("No transactions recorded in this session.\n", stdout);
        return BuiltinStatus::Handled;
    }

    let mut output = String::new();

    output.push_str(&format!("{:<18} | {:<12} | {:<15} | {}\n", "TRANSACTION ID", "STATUS", "RISK", "COMMAND"));
    output.push_str(&"-".repeat(75));
    output.push('\n');

    for tx in registry.iter() {
        let risk_str = format!("{:?}", tx.assessment.level);
        let status_str = format!("{:?}", tx.status);

        output.push_str(&format!("{:<18} | {:<12} | {:<15} | {}\n",
            tx.id,
            status_str,
            risk_str,
            tx.command_line
        ));
    }

    print_output(&output, stdout);
    BuiltinStatus::Handled
}
