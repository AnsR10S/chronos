use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Mutex, OnceLock};

pub fn history_registry() -> &'static Mutex<Vec<String>> {
    static REGISTRY: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn last_appended_idx() -> &'static Mutex<usize> {
    static LAST_IDX: OnceLock<Mutex<usize>> = OnceLock::new();
    LAST_IDX.get_or_init(|| Mutex::new(0))
}

pub fn add_history(cmd: String) {
    let mut registry = history_registry().lock().unwrap();
    registry.push(cmd);
}

pub fn append_from_file(path: &str) {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let mut registry = history_registry().lock().unwrap();
        let mut last_idx = last_appended_idx().lock().unwrap();

        for line in reader.lines() {
            if let Ok(cmd) = line {
                if !cmd.trim().is_empty() {
                    registry.push(cmd);
                }
            }
        }
        *last_idx = registry.len();
    }
}

pub fn write_to_file(path: &str) {
    let registry = history_registry().lock().unwrap();
    let mut last_idx = last_appended_idx().lock().unwrap();

    if let Ok(mut file) = File::create(path) {
        for cmd in registry.iter() {
            let _ = writeln!(file, "{}", cmd);
        }
        *last_idx = registry.len();
    }
}

pub fn append_to_file(path: &str) {
    let registry = history_registry().lock().unwrap();
    let mut last_idx = last_appended_idx().lock().unwrap();

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        for cmd in registry.iter().skip(*last_idx) {
            let _ = writeln!(file, "{}", cmd);
        }
        *last_idx = registry.len();
    }
}
