pub mod lexer;
pub mod shell;

fn main() {
    shell::repl::start();
}
