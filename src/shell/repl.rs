use crate::lexer::lexer::tokenize;
use crate::executor::single;
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
        // Check for and reap finished background jobs right before we prompt
        crate::shell::state::jobs::reap_jobs();

        let readline = rl.readline("$ ");

        match readline {
            Ok(line) => {
                let trimmed_input = line.trim();

                if trimmed_input.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(trimmed_input);
                let tokens = tokenize(trimmed_input);

                if tokens.is_empty() {
                    continue;
                }

                if single::execute(tokens) {
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
