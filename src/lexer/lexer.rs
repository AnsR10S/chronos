pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    // Use a peekable iterator so we can look ahead for multi-char operators like `>>`
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escaped {
            current_token.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' if !in_single_quote => escaped = true,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,

            // Handle Whitespace
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current_token.is_empty() {
                    tokens.push(std::mem::take(&mut current_token));
                }
            }

            // POSIX Edge Case: Unspaced Operators
            '>' | '<' | '|' | ';' | '&' if !in_single_quote && !in_double_quote => {
                // If we were building a word, flush it first (unless it's the "2" in "2>")
                if !current_token.is_empty() {
                    if current_token == "2" && c == '>' {
                        current_token.push('>'); // It's a stderr redirect!
                        tokens.push(std::mem::take(&mut current_token));
                        continue;
                    } else {
                        tokens.push(std::mem::take(&mut current_token));
                    }
                }

                // Extracts the operator
                let mut op = c.to_string();

                // Look ahead for double operators (>>, &&, ||)
                if c == '>' && chars.peek() == Some(&'>') {
                    op.push(chars.next().unwrap());
                } else if c == '&' && chars.peek() == Some(&'&') {
                    op.push(chars.next().unwrap());
                } else if c == '|' && chars.peek() == Some(&'|') {
                    op.push(chars.next().unwrap());
                }

                tokens.push(op);
            }

            // Normal characters
            _ => current_token.push(c),
        }
    }

    // Flush whatever is left in the buffer
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}
