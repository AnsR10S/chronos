use crate::parser::ast::Redirect;
use crate::shell::builtin::{print_output, BuiltinStatus};

pub fn execute(_args: &[&str], stdout: &Redirect) -> BuiltinStatus {
    let mut registry = crate::shell::state::jobs::jobs_registry().lock().unwrap();
    let total_jobs = registry.len();

    // Iterate and update statuses
    for (index, job) in registry.iter_mut().enumerate() {
        if let Ok(Some(_status)) = job.child.try_wait() {
            job.status = "Done".to_string();
        }

        let marker = if index == total_jobs - 1 {
            "+"
        } else if total_jobs > 1 && index == total_jobs - 2 {
            "-"
        } else {
            " "
        };

        let status_padded = format!("{:<24}", job.status);
        let display_cmd = if job.status == "Done" {
            job.command.trim_end_matches(" &")
        } else {
            &job.command
        };

        let output = format!("[{}]{} {}{}\n", job.id, marker, status_padded, display_cmd);
        print_output(&output, stdout);
    }

    // Clean up finished jobs
    registry.retain(|job| job.status != "Done");
    BuiltinStatus::Handled
}
