use std::process::Child;
use std::sync::{Mutex, OnceLock};

pub struct Job {
    pub id: usize,
    pub child: Child,
    pub command: String,
    pub status: String,
}

// Global registry to hold background jobs
pub fn jobs_registry() -> &'static Mutex<Vec<Job>> {
    static REGISTRY: OnceLock<Mutex<Vec<Job>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

// Spawns the job into the registry and returns its ID
pub fn add_job(child: Child, command: String) -> usize {
    let mut registry = jobs_registry().lock().unwrap();
    let id = registry.iter().map(|j| j.id).max().unwrap_or(0) + 1;

    registry.push(Job {
        id,
        child,
        command,
        status: "Running".to_string(),
    });

    id
}

// Reaps finished jobs right before the prompt is displayed
pub fn reap_jobs() {
    let mut registry = jobs_registry().lock().unwrap();
    let total_jobs = registry.len();
    let mut needs_cleanup = false;

    for (index, job) in registry.iter_mut().enumerate() {
        // Non-blocking check if the process exited
        if let Ok(Some(_status)) = job.child.try_wait() {
            job.status = "Done".to_string();
            needs_cleanup = true;

            // Determine the correct marker (+, -, or space)
            let marker = if index == total_jobs - 1 {
                "+"
            } else if total_jobs > 1 && index == total_jobs - 2 {
                "-"
            } else {
                " "
            };

            let status_padded = format!("{:<24}", job.status);
            let display_cmd = job.command.trim_end_matches(" &");

            println!("[{}]{} {}{}", job.id, marker, status_padded, display_cmd);
        }
    }

    if needs_cleanup {
        registry.retain(|job| job.status != "Done");
    }
}
