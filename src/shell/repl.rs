use crate::lexer::lexer::tokenize;
use crate::executor::single;
use crate::shell::completion::ChronosHelper;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

pub fn start() {
    // Initialize rustyline with our custom autocompletion helper
    let mut rl = Editor::<ChronosHelper, DefaultHistory>::new()
        .expect("Failed to initialize readline");

    rl.set_helper(Some(ChronosHelper::default()));

    loop {
        // rl.readline replaces our old print!("$ ") and read_line combo
        let readline = rl.readline("$ ");

        match readline {
            Ok(line) => {
                let trimmed_input = line.trim();

                if trimmed_input.is_empty() {
                    continue;
                }

                // Free bonus feature: Rustyline natively handles terminal history!
                let _ = rl.add_history_entry(trimmed_input);

                // Pass raw input through your lexical analyzer
                let tokens = tokenize(trimmed_input);

                if tokens.is_empty() {
                    continue;
                }

                // Hand over the parsed tokens to the executor
                if single::execute(tokens) {
                    break;
                }
            },
            Err(ReadlineError::Interrupted) => {
                // Handles Ctrl-C gracefully
                break;
            },
            Err(ReadlineError::Eof) => {
                // Handles Ctrl-D (EOF) gracefully
                break;
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
