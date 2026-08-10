use std::fs;
use std::path::{Path, PathBuf};
use crate::chronos::state::tracker::FsTarget;

pub fn get_snapshot_dir(tx_id: &str) -> Option<PathBuf> {
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

    let valid_targets: Vec<&FsTarget> = targets.iter().filter(|t| t.exists && !t.is_dir).collect();

    if valid_targets.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(&snap_dir)?;

    for target in valid_targets {
        let source_path = Path::new(&target.path);
        let safe_name = target.path.replace("/", "_").replace("\\", "_");
        let dest_path = snap_dir.join(&safe_name);

        fs::copy(source_path, dest_path)?;
    }

    Ok(())
}

pub fn restore_snapshot(tx_id: &str, targets: &[FsTarget]) -> Result<(), std::io::Error> {
    let snap_dir_opt = get_snapshot_dir(tx_id);

    for target in targets {
        let dest_path = Path::new(&target.path);

        if !target.exists {
            if dest_path.exists() {
                if dest_path.is_dir() {
                    let _ = fs::remove_dir_all(dest_path);
                } else {
                    let _ = fs::remove_file(dest_path);
                }
            }
        } else if let Some(ref snap_dir) = snap_dir_opt {
            if snap_dir.exists() {
                let safe_name = target.path.replace("/", "_").replace("\\", "_");
                let source_path = snap_dir.join(&safe_name);

                if source_path.exists() && !target.is_dir {
                    fs::copy(source_path, dest_path)?;
                }
            }
        }
    }

    Ok(())
}
