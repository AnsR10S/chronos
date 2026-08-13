pub mod chronos;
pub mod executor;
pub mod lexer;
pub mod parser;
pub mod shell;

#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    dotenv::dotenv().ok();
    crate::chronos::ai::client::warm_up_connection();

    // Scans for and recover from crashes instantly on boot
    crate::chronos::transaction::manager::recover_crashed_transactions();

    shell::repl::start();
}
