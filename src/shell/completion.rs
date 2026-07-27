use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::env;
use std::fs;
use std::collections::HashSet;

// Longest Common Prefix function
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

// Fetch all matching builtins and executables in PATH
fn get_command_completions(prefix: &str) -> Vec<String> {
    // Use a HashSet to avoid duplicate entries if a command exists in multiple PATH dirs
    let mut matches = HashSet::new();

    // Add builtins
    let builtins = ["echo", "exit", "type", "pwd", "cd"];
    for b in builtins {
        if b.starts_with(prefix) {
            matches.insert(b.to_string());
        }
    }

    // Add external executables from PATH
    if let Ok(path_var) = env::var("PATH") {
        for path in env::split_paths(&path_var) {
            // Gracefully handle directories that don't exist
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
        let word = &line[..pos];
        let mut pairs = Vec::new();

        // For now, we only complete the first word (the command itself)
        if !line.contains(' ') {
            let completions = get_command_completions(word);

            if completions.len() == 1 {
                // Single completion: add the trailing space
                let comp = &completions[0];
                pairs.push(Pair {
                    display: comp.clone(),
                    replacement: format!("{} ", comp),
                });
            } else if completions.len() > 1 {
                // Multiple completions: Use LCP to fill in the partial match!
                let lcp = longest_common_prefix(&completions);

                if lcp.len() > word.len() {
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
        }

        // Return 0 because we are replacing the word from the start of the line
        Ok((0, pairs))
    }
}

impl Helper for ChronosHelper {}
impl Hinter for ChronosHelper { type Hint = String; }
impl Highlighter for ChronosHelper {}
impl Validator for ChronosHelper {}
