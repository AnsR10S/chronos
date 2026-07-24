// Defines exactly where a standard stream should be routed
pub enum Redirect {
    None,
    Overwrite(String),
    Append(String),
}

// The core Abstract Syntax Tree representation of a command
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub stdout: Redirect,
    pub stderr: Redirect,
}
