pub mod executor;
pub mod lexer;
pub mod parser;
pub mod shell;

fn main() {
    shell::repl::start();
}
