pub mod chronos;
pub mod executor;
pub mod lexer;
pub mod parser;
pub mod shell;

#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    dotenv::dotenv().ok();

    // Establish the connection to Google instantly in the background
    crate::chronos::ai::client::warm_up_connection();

    shell::repl::start();
}
