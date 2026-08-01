use crate::shell::completion::ChronosHelper;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, CompletionType, Editor};

pub fn start() {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::<ChronosHelper, DefaultHistory>::with_config(config)
        .expect("Failed to initialize readline");

    rl.set_helper(Some(ChronosHelper::default()));

    // Check for HISTFILE and load it on startup
    if let Ok(histfile) = std::env::var("HISTFILE") {
        crate::shell::state::history::append_from_file(&histfile);
        let _ = rl.load_history(&histfile);
    }

    loop {
        crate::shell::state::jobs::reap_jobs();

        let readline = rl.readline("$ ");

        match readline {
            Ok(line) => {
                let trimmed_input = line.trim();

                if trimmed_input.is_empty() {
                    continue;
                }

                // Add to both Rustyline's internal memory and our custom state
                let _ = rl.add_history_entry(trimmed_input);
                crate::shell::state::history::add_history(trimmed_input.to_string());

                if crate::executor::executor::execute_pipeline(trimmed_input) {
                    break;
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // The shell has broken out of the loop and is about to exit. Flush to HISTFILE
    if let Ok(histfile) = std::env::var("HISTFILE") {
        crate::shell::state::history::append_to_file(&histfile);
    }
}
