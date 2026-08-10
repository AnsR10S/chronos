use std::fs;
use std::path::{Path, PathBuf};
use crate::chronos::state::tracker::FsTarget;

pub fn get_snapshot_dir(tx_id: &str) -> Option<PathBuf> {
    // Cross-platform home directory resolution
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()?;
    let mut path = PathBuf::from(home);
    path.push(".chronos");
    path.push("snapshots");
    path.push(tx_id);
    Some(path)
}

pub fn create_snapshot(tx_id: &str, targets: &[FsTarget]) -> Result<(), std::io::Error> {
    let snap_dir = match get_snapshot_dir(tx_id) {
        Some(dir) => dir,
        None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find home directory")),
    };

    // Filter for targets that actually exist and are files
    let valid_targets: Vec<&FsTarget> = targets.iter().filter(|t| t.exists && !t.is_dir).collect();

    if valid_targets.is_empty() {
        return Ok(());
    }

    // Create the snapshot directory securely
    fs::create_dir_all(&snap_dir)?;

    for target in valid_targets {
        let source_path = Path::new(&target.path);

        // Create a safe, flat filename (e.g., "folder_file.txt" instead of "folder/file.txt")
        let safe_name = target.path.replace("/", "_").replace("\\", "_");
        let dest_path = snap_dir.join(&safe_name);

        fs::copy(source_path, dest_path)?;
    }

    Ok(())
}

pub fn restore_snapshot(tx_id: &str, targets: &[FsTarget]) -> Result<(), std::io::Error> {
    let snap_dir = match get_snapshot_dir(tx_id) {
        Some(dir) => dir,
        None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find home directory")),
    };

    if !snap_dir.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Snapshot directory does not exist or was already deleted"));
    }

    // Only try to restore targets that were actually files and existed prior to the command
    let valid_targets: Vec<&FsTarget> = targets.iter().filter(|t| t.exists && !t.is_dir).collect();

    for target in valid_targets {
        let safe_name = target.path.replace("/", "_").replace("\\", "_");
        let source_path = snap_dir.join(&safe_name);
        let dest_path = Path::new(&target.path);

        if source_path.exists() {
            fs::copy(source_path, dest_path)?;
        }
    }

    Ok(())
}
