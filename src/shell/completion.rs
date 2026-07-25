use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

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

        // The builtins we want to autocomplete for this stage
        let builtins = ["echo", "exit"];
        let mut matches = Vec::new();

        // Only autocomplete if we are typing the first word (no spaces yet)
        if !line.contains(' ') {
            for b in builtins {
                if b.starts_with(word) {
                    matches.push(Pair {
                        display: b.to_string(),
                        // Add the trailing space exactly as requested
                        replacement: format!("{} ", b),
                    });
                }
            }
        }

        // 0 tells rustyline to replace text starting from the 0th index
        Ok((0, matches))
    }
}

// These empty implementations use Rustyline's default behaviors
impl Helper for ChronosHelper {}

impl Hinter for ChronosHelper {
    type Hint = String;
}

impl Highlighter for ChronosHelper {}
impl Validator for ChronosHelper {}
