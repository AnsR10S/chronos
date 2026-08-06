use std::fs;

pub fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let mut prefix = strings[0].clone();
    for s in strings.iter().skip(1) {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return String::new();
            }
        }
    }
    prefix
}

pub fn autocomplete_filename(search_word: &str) -> Vec<String> {
    let mut matches = Vec::new();

    let (dir_path, file_prefix, display_dir) = if let Some(last_slash) = search_word.rfind('/') {
        let dir = &search_word[..=last_slash];
        let prefix = &search_word[last_slash + 1..];
        (dir, prefix, dir)
    } else {
        (".", search_word, "")
    };

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with(file_prefix) {
                    let mut full_match = format!("{}{}", display_dir, file_name);

                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            full_match.push('/');
                        }
                    }
                    matches.push(full_match);
                }
            }
        }
    }

    matches.sort();
    matches
}
