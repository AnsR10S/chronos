pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut double_quote_escaped = false;
    let mut has_chars = false;

    for c in input.chars() {
        if escaped {
            current_token.push(c);
            has_chars = true;
            escaped = false;
            continue;
        }

        if double_quote_escaped {
            match c {
                '"' | '\\' | '$' | '\n' => {
                    current_token.push(c);
                }
                _ => {
                    current_token.push('\\');
                    current_token.push(c);
                }
            }
            has_chars = true;
            double_quote_escaped = false;
            continue;
        }

        match c {
            '\\' if !in_single_quote && !in_double_quote => {
                escaped = true;
            }
            '\\' if in_double_quote => {
                double_quote_escaped = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                has_chars = true;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                has_chars = true;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if has_chars {
                    tokens.push(current_token.clone());
                    current_token.clear();
                    has_chars = false;
                }
            }
            _ => {
                current_token.push(c);
                has_chars = true;
            }
        }
    }

    if has_chars {
        tokens.push(current_token);
    }

    tokens
}
