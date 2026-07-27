use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::env;
use std::fs;
use std::collections::HashSet;

pub fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let mut prefix = strings[0].clone();
    for s in strings.iter().skip(1) {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return String::new();
            }
        }
    }
    prefix
}

fn get_command_completions(prefix: &str) -> Vec<String> {
    let mut matches = HashSet::new();
    let builtins = ["echo", "exit", "type", "pwd", "cd"];

    for b in builtins {
        if b.starts_with(prefix) {
            matches.insert(b.to_string());
        }
    }

    if let Ok(path_var) = env::var("PATH") {
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.starts_with(prefix) {
                            matches.insert(file_name);
                        }
                    }
                }
            }
        }
    }

    let mut sorted_matches: Vec<String> = matches.into_iter().collect();
    sorted_matches.sort();
    sorted_matches
}

// Simplified filename completion matching your final architecture style
pub fn autocomplete_filename(search_word: &str) -> Vec<String> {
    let mut matches = Vec::new();

    // For this stage, we only read the current directory (".")
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with(search_word) {
                    matches.push(file_name);
                }
            }
        }
    }

    matches.sort();
    matches
}

#[derive(Default)]
pub struct ChronosHelper;

impl Completer for ChronosHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix = &line[..pos];
        let mut pairs = Vec::new();

        // Find where the current word starts (either after the last space, or index 0)
        let start_idx = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let search_word = &prefix[start_idx..];

        // Decide which completion list to fetch based on if we've typed a space
        if !prefix.contains(' ') {
            // We are completing the command itself
            let completions = get_command_completions(search_word);

            if completions.len() == 1 {
                let comp = &completions[0];
                pairs.push(Pair {
                    display: comp.clone(),
                    replacement: format!("{} ", comp),
                });
            } else if completions.len() > 1 {
                let lcp = longest_common_prefix(&completions);
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
        } else {
            // We have a space, so we are completing arguments (filenames)
            let completions = autocomplete_filename(search_word);

            // For this specific stage, we only need to handle single matches
            if completions.len() == 1 {
                let comp = &completions[0];
                pairs.push(Pair {
                    display: comp.clone(),
                    // Add the trailing space exactly as requested
                    replacement: format!("{} ", comp),
                });
            }
        }

        // Return start_idx instead of 0 so rustyline only replaces the current word,
        // preserving the command and previous arguments!
        Ok((start_idx, pairs))
    }
}

impl Helper for ChronosHelper {}
impl Hinter for ChronosHelper { type Hint = String; }
impl Highlighter for ChronosHelper {}
impl Validator for ChronosHelper {}
