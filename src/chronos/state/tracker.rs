use std::path::Path;
use std::fs;
use crate::parser::ast::{Command, Redirect};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsTarget {
    pub path: String,
    pub exists: bool,
    pub is_dir: bool,
    pub readonly: bool,
}

pub fn track_targets(cmd: &Command) -> Vec<FsTarget> {
    let mut targets = Vec::new();

    for arg in &cmd.args {
        if arg.starts_with('-') { continue; }

        if arg.contains('*') {
            if let Ok(entries) = fs::read_dir(".") {
                let parts: Vec<&str> = arg.split('*').collect();
                let prefix = parts.first().unwrap_or(&"");
                let suffix = parts.last().unwrap_or(&"");

                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.starts_with(prefix) && name.ends_with(suffix) {
                            targets.push(inspect_path(&name));
                        }
                    }
                }
            }
        } else {
            targets.push(inspect_path(arg));
        }
    }

    match &cmd.stdout {
        Redirect::Overwrite(path) | Redirect::Append(path) => {
            targets.push(inspect_path(path));
        }
        _ => {}
    }
    match &cmd.stderr {
        Redirect::Overwrite(path) | Redirect::Append(path) => {
            targets.push(inspect_path(path));
        }
        _ => {}
    }

    targets
}

fn inspect_path(path_str: &str) -> FsTarget {
    let path = Path::new(path_str);
    let exists = path.exists();
    let is_dir = path.is_dir();

    let readonly = if exists {
        if let Ok(metadata) = path.metadata() {
            metadata.permissions().readonly()
        } else {
            false
        }
    } else {
        false
    };

    FsTarget {
        path: path_str.to_string(),
        exists,
        is_dir,
        readonly,
    }
}
