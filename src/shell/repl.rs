use crate::executor::executor;
use crate::shell::builtins;
use crate::shell::completion;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, Editor, Helper};
use std::process::Command;

struct CommandCompleter;

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut pairs = Vec::new();
        let prefix = &line[..pos];

        let start_idx = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let search_word = &prefix[start_idx..];

        let completions = if !prefix.contains(' ') {
            builtins::autocomplete(search_word)
        } else {
            let cmd = prefix.split_whitespace().next().unwrap_or("");

            let tokens: Vec<&str> = prefix.split_whitespace().collect();
            let prev_word = if prefix.ends_with(' ') {
                tokens.last().copied().unwrap_or("")
            } else if tokens.len() >= 2 {
                tokens[tokens.len() - 2]
            } else {
                ""
            };

            if let Some(script_path) = builtins::get_completer(cmd) {
                if let Ok(output) = Command::new(script_path)
                    .env("COMP_LINE", line)
                    .env("COMP_POINT", pos.to_string())
                    .arg(cmd)
                    .arg(search_word)
                    .arg(prev_word)
                    .output()
                {
                    let generated_completions: Vec<String> = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect();

                    if !generated_completions.is_empty() {
                        generated_completions
                    } else {
                        completion::autocomplete_filename(search_word)
                    }
                } else {
                    completion::autocomplete_filename(search_word)
                }
            } else {
                completion::autocomplete_filename(search_word)
            }
        };

        if completions.len() == 1 {
            let comp = &completions[0];

            let replacement_str = if comp.ends_with('/') {
                comp.clone()
            } else {
                format!("{} ", comp)
            };

            pairs.push(Pair {
                display: comp.clone(),
                replacement: replacement_str,
            });

        } else if completions.len() > 1 {
            let lcp = completion::longest_common_prefix(&completions);

            if lcp.len() > search_word.len() {
                pairs.push(Pair {
                    display: lcp.clone(),
                    replacement: lcp,
                });
            } else {
                for comp in completions {
                    pairs.push(Pair {
                        display: comp.clone(),
                        replacement: comp,
                    });
                }
            }
        }

        Ok((start_idx, pairs))
    }
}

impl Highlighter for CommandCompleter {}
impl Hinter for CommandCompleter { type Hint = String; }
impl Validator for CommandCompleter {}
impl Helper for CommandCompleter {}

pub fn start() {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::<CommandCompleter, DefaultHistory>::with_config(config)
        .expect("Failed to create editor");

    rl.set_helper(Some(CommandCompleter));

    if let Ok(histfile) = std::env::var("HISTFILE") {
        crate::shell::state::history::append_from_file(&histfile);
        let _ = rl.load_history(&histfile);
    }

    loop {
        crate::shell::state::jobs::reap_jobs();

        let readline = rl.readline("$ ");

        match readline {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(input);
                crate::shell::state::history::add_history(input.to_string());

                if executor::execute_pipeline(input) {
                    break;
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    if let Ok(histfile) = std::env::var("HISTFILE") {
        crate::shell::state::history::append_to_file(&histfile);
    }
}
