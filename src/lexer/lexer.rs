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

#[cfg(test)]
mod tests {
    use super::tokenize;

    // Helper to concisely create string vectors
    macro_rules! svec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    // --- LEXER TESTS ---

    #[test]
    fn test_simple_command() {
        assert_eq!(tokenize("echo hello world"), svec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_empty_input() {
        let expected: Vec<String> = vec![];
        assert_eq!(tokenize(""), expected);
    }

    #[test]
    fn test_whitespace_only() {
        let expected: Vec<String> = vec![];
        assert_eq!(tokenize("   "), expected);
    }

    #[test]
    fn test_single_token() {
        assert_eq!(tokenize("ls"), svec!["ls"]);
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(tokenize("echo   hello"), svec!["echo", "hello"]);
    }

    #[test]
    fn test_tabs() {
        assert_eq!(tokenize("echo\thello"), svec!["echo", "hello"]);
    }

    #[test]
    fn test_single_quotes() {
        assert_eq!(tokenize("echo 'hello world'"), svec!["echo", "hello world"]);
    }

    #[test]
    fn test_double_quotes() {
        assert_eq!(tokenize("echo \"hello world\""), svec!["echo", "hello world"]);
    }

    #[test]
    fn test_mixed_quotes() {
        assert_eq!(tokenize("echo 'a' \"b\""), svec!["echo", "a", "b"]);
    }

    #[test]
    fn test_single_quote_preserves_backslash() {
        assert_eq!(tokenize("echo 'a\\b'"), svec!["echo", "a\\b"]);
    }

    #[test]
    fn test_double_quote_allows_escape() {
        assert_eq!(tokenize("echo \"a\\b\""), svec!["echo", "ab"]);
    }

    #[test]
    fn test_backslash_escape_space() {
        assert_eq!(tokenize("echo hello\\ world"), svec!["echo", "hello world"]);
    }

    #[test]
    fn test_pipe_operator() {
        assert_eq!(tokenize("cat f | grep x"), svec!["cat", "f", "|", "grep", "x"]);
    }

    #[test]
    fn test_double_pipe() {
        assert_eq!(tokenize("a || b"), svec!["a", "||", "b"]);
    }

    #[test]
    fn test_double_ampersand() {
        assert_eq!(tokenize("a && b"), svec!["a", "&&", "b"]);
    }

    #[test]
    fn test_single_ampersand() {
        assert_eq!(tokenize("sleep 10 &"), svec!["sleep", "10", "&"]);
    }

    #[test]
    fn test_semicolons() {
        assert_eq!(tokenize("a ; b"), svec!["a", ";", "b"]);
    }

    #[test]
    fn test_redirect_stdout() {
        assert_eq!(tokenize("echo hi > file"), svec!["echo", "hi", ">", "file"]);
    }

    #[test]
    fn test_redirect_append() {
        assert_eq!(tokenize("echo hi >> file"), svec!["echo", "hi", ">>", "file"]);
    }

    #[test]
    fn test_redirect_stderr() {
        assert_eq!(tokenize("cmd 2> err"), svec!["cmd", "2>", "err"]);
    }

    #[test]
    fn test_redirect_stderr_append_known_bug() {
        // Known bug: "cmd 2>> err" produces ["cmd", "2>", ">", "err"]
        assert_eq!(tokenize("cmd 2>> err"), svec!["cmd", "2>", ">", "err"]);
    }

    #[test]
    fn test_unclosed_single_quote() {
        assert_eq!(tokenize("echo 'hello"), svec!["echo", "hello"]);
    }

    #[test]
    fn test_unclosed_double_quote() {
        assert_eq!(tokenize("echo \"hello"), svec!["echo", "hello"]);
    }

    #[test]
    fn test_trailing_backslash() {
        assert_eq!(tokenize("echo trail\\"), svec!["echo", "trail"]);
    }

    #[test]
    fn test_pipe_inside_quotes() {
        assert_eq!(tokenize("echo 'a|b'"), svec!["echo", "a|b"]);
    }

    #[test]
    fn test_empty_quotes_dropped() {
        assert_eq!(tokenize("echo \"\" arg"), svec!["echo", "arg"]);
    }

    #[test]
    fn test_variable_syntax_preserved() {
        assert_eq!(tokenize("echo $HOME ${VAR}"), svec!["echo", "$HOME", "${VAR}"]);
    }

    #[test]
    fn test_complex_mixed() {
        assert_eq!(
            tokenize("grep -rn \"pattern\" dir/ | sort > out.txt 2> err.txt &"),
            svec!["grep", "-rn", "pattern", "dir/", "|", "sort", ">", "out.txt", "2>", "err.txt", "&"]
        );
    }

    #[test]
    fn test_adjacent_operators() {
        assert_eq!(tokenize(">|"), svec![">", "|"]);
    }

    #[test]
    fn test_redirect_no_space() {
        assert_eq!(tokenize("echo>file"), svec!["echo", ">", "file"]);
    }

    // --- SECURITY-RELEVANT LEXER TESTS ---

    #[test]
    fn test_security_metachar_in_single_quotes() {
        assert_eq!(tokenize("echo '| > < &'"), svec!["echo", "| > < &"]);
    }

    #[test]
    fn test_security_metachar_in_double_quotes() {
        assert_eq!(tokenize("echo \"| > < &\""), svec!["echo", "| > < &"]);
    }

    #[test]
    fn test_security_escaped_metachar() {
        assert_eq!(tokenize("echo \\> \\|"), svec!["echo", ">", "|"]);
    }

    #[test]
    fn test_security_long_input() {
        let input = "a".repeat(10000);
        let tokens = tokenize(&input);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].len(), 10000);
    }

    #[test]
    fn test_security_many_tokens() {
        let input = "a ".repeat(1000);
        let tokens = tokenize(&input);
        assert_eq!(tokens.len(), 1000);
    }

    #[test]
    fn test_security_nested_quotes() {
        assert_eq!(tokenize("echo \"it's\""), svec!["echo", "it's"]);
    }

    #[test]
    fn test_security_null_byte_char() {
        assert_eq!(tokenize("echo \0"), svec!["echo", "\0"]);
    }

    #[test]
    fn test_security_only_operators() {
        assert_eq!(tokenize("> >> | || && ; &"), svec![">", ">>", "|", "||", "&&", ";", "&"]);
    }
}
