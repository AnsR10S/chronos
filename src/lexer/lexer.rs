pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

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

            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current_token.is_empty() {
                    tokens.push(std::mem::take(&mut current_token));
                }
            }

            '>' | '<' | '|' | ';' | '&' if !in_single_quote && !in_double_quote => {
                if !current_token.is_empty() {
                    if current_token == "2" && c == '>' {
                        current_token.push('>');
                        tokens.push(std::mem::take(&mut current_token));
                        continue;
                    } else {
                        tokens.push(std::mem::take(&mut current_token));
                    }
                }

                let mut op = c.to_string();

                if c == '>' && chars.peek() == Some(&'>') {
                    op.push(chars.next().unwrap());
                } else if c == '&' && chars.peek() == Some(&'&') {
                    op.push(chars.next().unwrap());
                } else if c == '|' && chars.peek() == Some(&'|') {
                    op.push(chars.next().unwrap());
                }

                tokens.push(op);
            }

            _ => current_token.push(c),
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}
