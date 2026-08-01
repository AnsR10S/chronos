use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// Our global memory store for shell variables
pub fn variables_registry() -> &'static Mutex<HashMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn set_variable(name: String, value: String) {
    let mut registry = variables_registry().lock().unwrap();
    registry.insert(name, value);
}

pub fn get_variable(name: &str) -> Option<String> {
    let registry = variables_registry().lock().unwrap();
    registry.get(name).cloned()
}
