use std::path::Path;
use crate::parser::ast::Command;

#[derive(Debug)]
pub struct FsTarget {
    pub path: String,
    pub exists: bool,
    pub is_dir: bool,
    pub readonly: bool,
}

pub fn track_targets(cmd: &Command) -> Vec<FsTarget> {
    let mut targets = Vec::new();

    // Iterate through arguments to find potential file paths
    for arg in &cmd.args {
        // Skip obvious flags (e.g., "-r", "--force")
        if !arg.starts_with('-') {
            let path = Path::new(arg);
            let exists = path.exists();
            let is_dir = path.is_dir();

            // Check permissions if the file exists
            let readonly = if exists {
                if let Ok(metadata) = path.metadata() {
                    metadata.permissions().readonly()
                } else {
                    false
                }
            } else {
                false
            };

            targets.push(FsTarget {
                path: arg.clone(),
                exists,
                is_dir,
                readonly,
            });
        }
    }

    targets
}
