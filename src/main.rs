pub mod chronos;
pub mod executor;
pub mod lexer;
pub mod parser;
pub mod shell;

#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    shell::repl::start();
}
