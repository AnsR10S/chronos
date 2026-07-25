pub enum Redirect {
    None,
    Overwrite(String),
    Append(String),
}

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub stdout: Redirect,
    pub stderr: Redirect,
}
