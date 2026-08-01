use crate::shell::state::variables;

pub fn expand_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| expand_string(arg))
        .filter(|arg| !arg.is_empty()) // Drop arguments that became completely empty
        .collect()
}

fn expand_string(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            // Check if the next character is an opening brace
            if let Some(&'{') = chars.peek() {
                chars.next(); // Consume the '{'

                let mut var_name = String::new();

                // Read everything until the closing '}'
                while let Some(&next_c) = chars.peek() {
                    if next_c == '}' {
                        chars.next(); // Consume the '}'
                        break;
                    } else {
                        var_name.push(chars.next().unwrap());
                    }
                }

                // Look up the extracted variable name
                if !var_name.is_empty() {
                    if let Some(val) = variables::get_variable(&var_name) {
                        result.push_str(&val);
                    }
                }
            } else {
                // FALLBACK: The original logic for standard $VAR expansion
                let mut var_name = String::new();

                // A variable name stops at the first non-alphanumeric/underscore character
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_alphanumeric() || next_c == '_' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if !var_name.is_empty() {
                    if let Some(val) = variables::get_variable(&var_name) {
                        result.push_str(&val);
                    }
                } else {
                    // If it was just a lone '$' with no name after it
                    result.push('$');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
