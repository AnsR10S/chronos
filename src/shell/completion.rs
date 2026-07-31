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
    let builtins = ["echo", "exit", "type", "pwd", "cd", "complete"];

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

pub fn autocomplete_filename(search_word: &str) -> Vec<String> {
    let mut matches = Vec::new();

    let (dir_path, file_prefix, display_dir) = if let Some(last_slash) = search_word.rfind('/') {
        let dir = &search_word[..=last_slash];
        let prefix = &search_word[last_slash + 1..];
        (dir, prefix, dir)
    } else {
        (".", search_word, "")
    };

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with(file_prefix) {
                    let mut full_match = format!("{}{}", display_dir, file_name);

                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            full_match.push('/');
                        }
                    }

                    matches.push(full_match);
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

        let start_idx = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let search_word = &prefix[start_idx..];

        let completions = if !prefix.contains(' ') {
            get_command_completions(search_word)
        } else {
            autocomplete_filename(search_word)
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

        Ok((start_idx, pairs))
    }
}

impl Helper for ChronosHelper {}
impl Hinter for ChronosHelper { type Hint = String; }
impl Highlighter for ChronosHelper {}
impl Validator for ChronosHelper {}
