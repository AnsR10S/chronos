pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    // Our state trackers
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut double_quote_escaped = false;
    let mut has_chars = false;

    for c in input.chars() {
        // HIGHEST PRIORITY: The "outside quotes" shield
        if escaped {
            current_token.push(c);
            has_chars = true;
            escaped = false;
            continue;
        }

        // SECOND PRIORITY: The "inside double quotes" shield
        if double_quote_escaped {
            match c {
                // Characters that CAN be escaped inside double quotes
                '"' | '\\' | '$' | '\n' => {
                    current_token.push(c);
                }
                // If it's anything else, push the literal backslash AND the character
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
            // Backslash OUTSIDE quotes
            '\\' if !in_single_quote && !in_double_quote => {
                escaped = true;
            }
            // Backslash INSIDE double quotes
            '\\' if in_double_quote => {
                double_quote_escaped = true;
            }
            // Toggle single quotes ONLY if we aren't inside double quotes
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                has_chars = true;
            }
            // Toggle double quotes ONLY if we aren't inside single quotes
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                has_chars = true;
            }
            // Spaces split tokens only if we are completely unquoted
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if has_chars {
                    tokens.push(current_token.clone());
                    current_token.clear();
                    has_chars = false;
                }
            }
            // Everything else accumulates literally
            _ => {
                current_token.push(c);
                has_chars = true;
            }
        }
    }

    // Push the final token if there is one left over
    if has_chars {
        tokens.push(current_token);
    }

    tokens
}
