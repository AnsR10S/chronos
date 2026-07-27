use crate::lexer::lexer::tokenize;
use crate::executor::single;
use crate::shell::completion::ChronosHelper;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
// Import Config and CompletionType
use rustyline::{Config, CompletionType, Editor};

pub fn start() {
    // Build a custom config specifying List completion, just like your final target!
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    // Initialize rustyline with our config
    let mut rl = Editor::<ChronosHelper, DefaultHistory>::with_config(config)
        .expect("Failed to initialize readline");

    rl.set_helper(Some(ChronosHelper::default()));

    loop {
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
