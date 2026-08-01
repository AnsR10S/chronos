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

    loop {
        crate::shell::state::jobs::reap_jobs();

        let readline = rl.readline("$ ");

        match readline {
            Ok(line) => {
                let trimmed_input = line.trim();

                if trimmed_input.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(trimmed_input);

                // Let the orchestrator handle tokenization and routing to pipeline or single
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
}
