use crate::parser::ast::Redirect;
use crate::shell::builtins::{print_output, BuiltinStatus};

pub fn execute(_args: &[String], stdout: &Redirect) -> BuiltinStatus {
    let mut registry = crate::shell::state::jobs::jobs_registry().lock().unwrap();
    let total_jobs = registry.len();

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

        let display_cmd = if job.status == "Done" {
            job.command.trim_end_matches(" &")
        } else {
            &job.command
        };

        // We now output `[1]+  Running` instead of `[1] +Running`
        let output = format!("[{}]{}  {:<24}{}\n", job.id, marker, job.status, display_cmd);
        print_output(&output, stdout);
    }

    registry.retain(|job| job.status != "Done");
    BuiltinStatus::Handled
}
