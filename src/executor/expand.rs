use crate::shell::state::variables;

pub fn expand_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| expand_string(arg))
        .filter(|arg| !arg.is_empty())
        .collect()
}

fn expand_string(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&'{') = chars.peek() {
                chars.next();

                let mut var_name = String::new();

                while let Some(&next_c) = chars.peek() {
                    if next_c == '}' {
                        chars.next();
                        break;
                    } else {
                        var_name.push(chars.next().unwrap());
                    }
                }

                if !var_name.is_empty() {
                    if let Some(val) = variables::get_variable(&var_name) {
                        result.push_str(&val);
                    }
                }
            } else {
                let mut var_name = String::new();

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
                    result.push('$');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
